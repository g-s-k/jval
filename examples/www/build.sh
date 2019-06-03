#!/bin/sh

DIRNAME=$(dirname "$0")
OUTDIR="$DIRNAME/../../target/pkg"

wasm-pack build --target web --no-typescript --out-dir "$OUTDIR" "$DIRNAME"

rm -v "$OUTDIR/package.json"

cp -v "$DIRNAME"/src/static/* "$OUTDIR"
