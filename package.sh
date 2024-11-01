#!/usr/bin/env bash

# Modified version of https://github.com/casey/just/blob/master/bin/package
# See that project for the license.

set -euxo pipefail

VERSION=${REF#"refs/tags/"}
DIST=`pwd`/dist

echo "Packaging json-patch-cli $VERSION for $TARGET..."

test -f Cargo.lock || cargo generate-lockfile

echo "Installing rust toolchain for $TARGET..."
rustup target add $TARGET

if [[ $TARGET == aarch64-unknown-linux-musl ]]; then
  export CC=aarch64-linux-gnu-gcc
fi

echo "Building json-patch-cli..."
RUSTFLAGS="--deny warnings --codegen target-feature=+crt-static $TARGET_RUSTFLAGS" \
  cargo build --bin json-patch --target $TARGET --release
EXECUTABLE=target/$TARGET/release/json-patch

if [[ $OS == windows-latest ]]; then
  EXECUTABLE=$EXECUTABLE.exe
fi

echo "Copying release files..."
mkdir dist
cp -r \
  $EXECUTABLE \
  Cargo.lock \
  Cargo.toml \
  LICENSE \
  README.md \
  $DIST

cd $DIST
echo "Creating release archive..."
case $OS in
  ubuntu-latest | macos-latest)
    ARCHIVE=json-patch-cli-$VERSION-$TARGET.tar.gz
    tar czf $ARCHIVE *
    echo "archive=$DIST/$ARCHIVE" >> $GITHUB_OUTPUT
    ;;
  windows-latest)
    ARCHIVE=json-patch-cli-$VERSION-$TARGET.zip
    7z a $ARCHIVE *
    echo "archive=`pwd -W`/$ARCHIVE" >> $GITHUB_OUTPUT
    ;;
esac