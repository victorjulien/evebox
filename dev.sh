#! /bin/sh

set -e
set -x

trap 'echo "Killing background jobs..."; kill $(jobs -p)' EXIT

args="$@"

if ! test -d webapp/node_modules; then
    (cd webapp && npm ci --prefer-offline)
fi

(cd webapp && npm start) &
cargo watch -w src -i webapp -x "run server --no-tls --no-auth ${args}"
