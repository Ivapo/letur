#!/usr/bin/env bash
# Build the published site in `_site/` at the repository root — `ltr-001` Phase 2.
#
#   _site/index.html        the landing page, `web/index.html`, as it is
#   _site/app/index.html    `app/dist/index.html` with one line inserted
#   _site/app/pdfjs/        the renderer the window vendors
#   _site/app/host/         the `__TAURI__` the window answers through
#   _site/app/pkg/          the module, built by `wasm-pack` beforehand
#
# **The app page is a copy, never an edit**, by the rule `app/harness/serve.mjs`
# follows: `app/typecheck.mjs` dies unless the file holds exactly one module
# script. The inserted line goes immediately before `</head>`, where the harness
# puts its stub, so the host has set `window.__TAURI__` before the window's own
# module reads it. It is the only difference, and `web/check.mjs` holds it so.
#
# Run from anywhere; it works from the repository root. `web/pkg/` must exist:
# `cd web && wasm-pack build --target web --release` first.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

page=app/dist/index.html
heads=$(grep -c '</head>' "$page" || true)
if [ "$heads" != 1 ]; then
  echo "assemble: $page has $heads </head> lines, not one" >&2
  exit 1
fi
if [ ! -f web/pkg/letur_web_bg.wasm ]; then
  echo "assemble: web/pkg/ has no module — run wasm-pack build in web/ first" >&2
  exit 1
fi

rm -rf _site
mkdir -p _site/app

cp web/index.html _site/
awk '/<\/head>/ { print "    <script type=\"module\" src=\"host/host.mjs\"></script>" } { print }' \
  "$page" > _site/app/index.html
cp -R app/dist/pdfjs _site/app/pdfjs
cp -R web/host _site/app/host
cp -R web/pkg _site/app/pkg
# `.gitignore` inside pkg/ would make the Pages upload skip the module.
rm -f _site/app/pkg/.gitignore

echo "--- what is being published ---"
find _site -type f -exec ls -l {} \; | awk '{ print $5, $9 }'
