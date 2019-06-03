#!/bin/sh

DIRNAME=$(realpath $(dirname "$0"))
OUTDIR=$(realpath "$DIRNAME/../../target/pkg")

wasm-pack build --target web --no-typescript --out-dir "$OUTDIR" "$DIRNAME"

rm -v "$OUTDIR/package.json"

cp -v "$DIRNAME"/src/static/* "$OUTDIR"
