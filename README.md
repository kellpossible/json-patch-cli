# json-patch-cli

A CLI command to perform JSON RFC 6902 patching, merging and editing operations.

This project provides a CLI command `json-patch` which uses the brilliant work from the [`json-patch`](https://crates.io/crates/json-patch) crate.

## Usage

```txt
Usage: json-patch <COMMAND>

Commands:
  diff         Calculate the difference between two json files to create a JSON (RFC 6902) patch
  apply        Apply a JSON (RFC 6902) patch
  edit         Edit a JSON (RFC 6902) patch, by editing a patched version of the input using a text editor
  completions  Generate command line completions script
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `diff` Command

```txt
Calculate the difference between two json files to create a JSON (RFC 6902) patch

Usage: json-patch diff <FROM> <TO>

Arguments:
  <FROM>
  <TO>

Options:
  -h, --help           Print help
```

### `apply` Command

```txt
Apply a JSON (RFC 6902) patch

Usage: json-patch apply --patch <PATCH> <INPUT>

Arguments:
  <INPUT>

Options:
  -p, --patch <PATCH>
  -h, --help           Print help
```

### `edit` Command

```txt
Edit a JSON (RFC 6902) patch, by editing a patched version of the input using a text editor

Usage: json-patch edit [OPTIONS] --patch <PATCH> <INPUT>

Arguments:
  <INPUT>

Options:
  -w, --watch            Enable live editing of the patch file
  -p, --patch <PATCH>
  -e, --editor <EDITOR>  [default: vim]
  -h, --help             Print help
```