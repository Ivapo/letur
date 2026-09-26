#!/usr/bin/env bash
# Build the published site in `_site/` at the repository root — `ltr-001` Phase 2.
#
#   _site/index.html        the landing page, `web/index.html`, as it is
#   _site/fonts/            the faces it is set in, `web/fonts/`
#   _site/hero-*.png        the window it shows, `web/hero.mjs`'s capture
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
# **The fonts are Libertinus Serif and Mono**, the faces `md2pdf-core` sets
# every page in, subset from `md2pdf-core-0.4.0/assets/fonts/` — `mpdf-006`
# Phase 6 — by this, once per face, beside that directory's `OFL.txt`:
#
#   uvx --from 'fonttools[woff]' pyftsubset <font>.otf --flavor=woff2 \
#     --unicodes='U+0020-007E,U+00A0-00FF,U+2010-2027,U+2190-2193' \
#     --output-file=web/fonts/<font>.woff2
#
# **The two images are copied when they exist**, so `web/hero.mjs` can make its
# first capture against a site assembled before them.
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
cp -R web/fonts _site/fonts
for scheme in light dark; do
  if [ -f "web/hero-$scheme.png" ]; then cp "web/hero-$scheme.png" _site/; fi
done
awk '/<\/head>/ { print "    <script type=\"module\" src=\"host/host.mjs\"></script>" } { print }' \
  "$page" > _site/app/index.html
cp -R app/dist/pdfjs _site/app/pdfjs
cp -R web/host _site/app/host
cp -R web/pkg _site/app/pkg
# `.gitignore` inside pkg/ would make the Pages upload skip the module.
rm -f _site/app/pkg/.gitignore

echo "--- what is being published ---"
find _site -type f -exec ls -l {} \; | awk '{ print $5, $9 }'
