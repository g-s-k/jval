#!/bin/sh

wasm-pack build --target web --no-typescript

rm -v pkg/package.json

cp -v src/static/* pkg
