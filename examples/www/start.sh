#!/bin/sh

mkdir -p pkg && cd pkg && python3 -m http.server ${1:-8000} &
PYPID=$!

trap 'kill $PYPID' 2

if [ $(uname) = "Linux" ]; then
    inotifywait -rme modify,close_write,move,create,delete ./src ../../src |
    while read events; do ./build.sh; done
else
    kill $PYPID
    exit 1
fi

