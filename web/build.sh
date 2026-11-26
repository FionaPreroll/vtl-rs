#!/bin/sh
# Builds the web version into the directory given (default: _site), as the
# workflow pages.yml publishes it: the page, the machine as WebAssembly and
# the example programs. Serve it with any static server, e.g.
#   web/build.sh && python3 -m http.server -d _site
set -eu
cd "$(dirname "$0")/.."
out=${1:-_site}
cargo build -p vtl-web --release --target wasm32-unknown-unknown --locked
rm -rf "$out"
mkdir -p "$out/examples"
cp -R web/www/. "$out/"
cp target/wasm32-unknown-unknown/release/vtl_web.wasm "$out/"
cp examples/*.vtl "$out/examples/"
