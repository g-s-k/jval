#!/bin/sh

wasm-pack build --target web

rm -v pkg/package.json pkg/*.ts

cp -v src/static/* pkg
