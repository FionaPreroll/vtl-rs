#!/bin/sh
# Builds the web version into the directory given (default: _site), as the
# workflow pages.yml publishes it: the page, the machine as WebAssembly, the
# example programs and build.json, which the About box shows. Serve it with
# any static server, e.g.
#   web/build.sh && python3 -m http.server -d _site
# BUILD_SHA and BUILD_BRANCH name the commit and branch (default: git's), and
# the variables of GitHub Actions the run.
set -eu
cd "$(dirname "$0")/.."
out=${1:-_site}
cargo build -p vtl-web --release --target wasm32-unknown-unknown --locked
rm -rf "$out"
mkdir -p "$out/examples"
cp -R web/www/. "$out/"
cp target/wasm32-unknown-unknown/release/vtl_web.wasm "$out/"
cp examples/*.vtl "$out/examples/"

# A string for JSON: backslashes and quotes escaped.
json() {
  printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g')"
}
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
commit=${BUILD_SHA:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}
branch=${BUILD_BRANCH:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)}
built=$(date -u +%Y-%m-%dT%H:%M:%SZ)
repository=${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-FionaPreroll/vtl-rs}
if [ -n "${GITHUB_RUN_ID:-}" ]; then
  run="${GITHUB_WORKFLOW:-CI} #${GITHUB_RUN_NUMBER:-?}, attempt ${GITHUB_RUN_ATTEMPT:-1}"
  run_url="$repository/actions/runs/$GITHUB_RUN_ID/attempts/${GITHUB_RUN_ATTEMPT:-1}"
else
  run="local build"
  run_url=""
fi
cat > "$out/build.json" <<JSON
{
  "version": $(json "$version"),
  "commit": $(json "$commit"),
  "branch": $(json "$branch"),
  "built": $(json "$built"),
  "repository": $(json "$repository"),
  "run": $(json "$run"),
  "runUrl": $(json "$run_url")
}
JSON

# Each build its own cache in the service worker.
sed "s/^const VERSION = 'dev';/const VERSION = '$(printf '%s' "$commit" | cut -c1-12)-$built';/" web/www/sw.js > "$out/sw.js"
