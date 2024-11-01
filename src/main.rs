use console::{style, Style};
use std::process::Stdio;
use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use clap::{CommandFactory, Parser};

#[derive(clap::Parser)]
#[clap(name = "json-patch")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Calculate the difference between two json files to create a JSON (RFC 6902) patch.
    Diff(DiffCommand),
    /// Apply a JSON (RFC 6902) patch.
    Apply(ApplyCommand),
    /// Edit or create a JSON (RFC 6902) patch, by editing a patched version of the input using a text editor.
    Edit(EditCommand),
    /// Generate command line completions script.
    Completions(CompletionsCommand),
}

#[derive(clap::Args)]
struct DiffCommand {
    from: PathBuf,
    to: PathBuf,
}

fn diff_impl(command: DiffCommand) -> anyhow::Result<String> {
    let from: serde_json::Value =
        serde_json::from_slice(&std::fs::read(command.from).context("Error reading from file")?)
            .context("Error parsing from file as json")?;
    let to: serde_json::Value =
        serde_json::from_slice(&std::fs::read(command.to).context("Error reading to file")?)
            .context("Error parsing to file as json")?;
    let patch = json_patch::diff(&from, &to);
    serde_json::to_string_pretty(&patch).context("Error serializing patch")
}

fn diff(command: DiffCommand) -> anyhow::Result<()> {
    let patch_string = diff_impl(command)?;
    println!("{patch_string}");
    Ok(())
}

#[derive(clap::Args)]
struct ApplyCommand {
    input: PathBuf,
    #[arg(short, long)]
    patch: PathBuf,
}

fn apply_impl(command: ApplyCommand) -> anyhow::Result<String> {
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(command.input).context("Error reading from file")?)
            .context("Error parsing input file as json")?;
    let patch: json_patch::Patch =
        serde_json::from_slice(&std::fs::read(command.patch).context("Error reading patch file")?)
            .context("Error parsing patch file as json")?;
    json_patch::patch(&mut document, &patch).context("Error applying patch")?;
    serde_json::to_string_pretty(&document).context("Error serializing output")
}

fn apply(command: ApplyCommand) -> anyhow::Result<()> {
    let output_string = apply_impl(command)?;
    println!("{output_string}");
    Ok(())
}

#[derive(clap::Args)]
struct EditCommand {
    input: PathBuf,
    /// Enable live editing of the patch file.
    #[arg(short, long)]
    watch: bool,
    /// Path to JSON patch file.
    ///
    /// If the patch file does not yet exist, this command will create a new one.
    #[arg(short, long)]
    patch: PathBuf,
    #[arg(short, long, default_value = "vim")]
    editor: String,
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn edit(command: EditCommand) -> anyhow::Result<()> {
    let command = &command;
    // Create a temporary file
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("patched.json");
    let path = &path;

    let (patched, old_patch) =
        if std::fs::exists(&command.patch).context("Error checking whether patch file exists")? {
            let patched = apply_impl(ApplyCommand {
                input: command.input.clone(),
                patch: command.patch.clone(),
            })?;
            let old_patch =
                std::fs::read_to_string(&command.patch).context("Error reading patch file")?;
            (patched, old_patch)
        } else {
            (
                std::fs::read_to_string(&command.input).context("Error reading input")?,
                String::new(),
            )
        };

    std::fs::write(path, patched)?;

    std::thread::scope(|s| {
        if command.watch {
            s.spawn(move || {
                let mut previous_final = None;
                loop {
                    std::thread::sleep(Duration::from_secs(1));

                    if let Err(e) = (|| {
                        let current_final = std::fs::read_to_string(path)?;

                        if Some(&current_final) == previous_final.as_ref() {
                            return Ok(());
                        }

                        log::trace!("Detected change in {path:?}");

                        let new_patch = diff_impl(DiffCommand {
                            from: command.input.clone(),
                            to: path.clone(),
                        })
                        .context("Error executing diff")?;
                        std::fs::write(command.patch.clone(), new_patch)?;

                        previous_final = Some(current_final);
                        anyhow::Ok(())
                    })() {
                        log::error!("Error occurred while watching: {e}");
                    }
                }
            });
        }

        log::debug!("Editing {path:?} with {}", &command.editor);

        // Spawn Vim as a child process
        std::process::Command::new(&command.editor)
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to open vim");

        let new_patch = diff_impl(DiffCommand {
            from: command.input.clone(),
            to: path.clone(),
        })
        .context("Error executing diff")?;

        let diff = similar::TextDiff::from_lines(&old_patch, &new_patch);
        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                println!("{:-^1$}", "-", 80);
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let (sign, s) = match change.tag() {
                        similar::ChangeTag::Delete => ("-", Style::new().red()),
                        similar::ChangeTag::Insert => ("+", Style::new().green()),
                        similar::ChangeTag::Equal => (" ", Style::new().dim()),
                    };
                    print!(
                        "{}{} |{}",
                        style(Line(change.old_index())).dim(),
                        style(Line(change.new_index())).dim(),
                        s.apply_to(sign).bold(),
                    );
                    for (emphasized, value) in change.iter_strings_lossy() {
                        if emphasized {
                            print!("{}", s.apply_to(value).underlined().on_black());
                        } else {
                            print!("{}", s.apply_to(value));
                        }
                    }
                    if change.missing_newline() {
                        println!();
                    }
                }
            }
        }

        std::fs::write(command.patch.clone(), new_patch)?;

        // Exit and don't join the threads.
        std::process::exit(0)
    })
}

#[derive(clap::Args)]
struct CompletionsCommand {
    shell: clap_complete::Shell,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Command::Diff(command) => diff(command)?,
        Command::Apply(command) => apply(command)?,
        Command::Edit(command) => edit(command)?,
        Command::Completions(command) => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(command.shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }

    Ok(())
}
