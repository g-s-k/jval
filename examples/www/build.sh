#!/bin/sh

DIRNAME=$(dirname "$0")

wasm-pack build --target web --no-typescript "$DIRNAME"

rm -v "$DIRNAME/pkg/package.json"

cp -v "$DIRNAME"/src/static/* "$DIRNAME/pkg"
