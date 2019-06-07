#!/bin/sh

./build.sh

mkdir -p ../../target/pkg && cd ../../target/pkg && python3 -m http.server ${1:-8000} &
PYPID=$!

if [ $(uname) = "Linux" ]; then
    trap 'kill $PYPID' 2
    inotifywait -rme modify,close_write,move,create,delete ./src ../../src |
    while read events; do ./build.sh; done
else
    trap 'pkill Python' 2
    fswatch -o ./src ../../src | xargs -n1 -I{} ./build.sh
fi
