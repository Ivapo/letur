---
title: web-app
sources:
  - web/src/lib.rs
  - web/host/host.mjs
  - web/host/worker.mjs
  - web/assemble.sh
  - web/Cargo.toml
  - web/package.json
  - web/check.mjs
  - .github/workflows/pages.yml
covers: >
  Letur in a browser tab: the window copied rather than edited and the one line
  inserted into it, the session the worker holds and the one method per command
  it answers, the three conventions an answer crosses in, the virtual root and
  the staged open, the download token and the method that moves nothing, the
  host's surface and what it answers itself, the debounce and the flush, the
  events no loop sends, the chords the menu carried and the one it did not, the
  seed that opens an example and the landing page it reads, the site the
  assemble step builds, the lockfile seeded from the CLI's and the check that
  refuses to compare without it, the module's size, and the build and deploy
  that publish it
max_lines: 220
generated: 2026-09-26
---

# Web app

Letur's own window in a browser tab, at `https://ivapo.github.io/letur/app/`. `ltr-001`
owns it; Phase 2 built it over a project that lives as long as the tab. The landing
page one level up links every example into it, and `rules/web-demo.md` covers that page.

## The window is a copy, and one line is added

**`_site/app/index.html` is `app/dist/index.html` byte for byte, plus one line**:
`<script type="module" src="host/host.mjs">`, inserted immediately before `</head>` by
`web/assemble.sh` — the place `app/harness/serve.mjs` injects its stub. A module in
`<head>` runs before the window's own module, which reads `window['__TAURI__']` at top
level. **The page is never edited for the browser's sake**: `app/typecheck.mjs` dies
unless the file holds exactly one module script, and a second front end would drift
from the first on the next phase of `mpdf-003` with nothing noticing. A command the
host has not learnt rejects with `<command> is not answered in a browser`, which the
window prints through `fail`. The window reads three members of the object —
`core.invoke`, `dialog.open`/`dialog.save` and `event.listen` — and nothing else.

## The session

**`web/src/lib.rs:Session` is `letur-project`'s `Preview` over a `MemFiles`, and every
rule is the crate's.** What the file adds is what the desktop's commands and `Session`
add around a `Preview`, restated for a host with no disk, no watch, no threads and no
clock: one method per session row of `ltr-001` §2's table, plus `stage`, `unstage`,
`compile` and `set_appearance`. It is built with `Preview::new(timed, None)` — **no claim
channel**, so no URL is ever marked waiting — and `web/src/lib.rs:timed` reads
`performance.now()` through a `#[wasm_bindgen]` extern, so the compiles inside an open,
a row click or a discard are timed as the desktop's `Instant` timer times them.

**Three conventions cross the boundary.** A refusal is thrown as the sentence, a plain
string (`web/src/lib.rs:refused`), since the window's `fail` prints `String(problem)`.
A structured answer is a JSON string from `serde_json`, the serializer Tauri's IPC uses,
so `None` is `null` — `serde-wasm-bindgen` was refused for turning it into `undefined`,
which would have shown the window's three bars on every status. Bytes are `Vec<u8>`,
which JavaScript receives as a `Uint8Array`.

**A path is virtual**: `/project/<root-relative path>`, stripped by
`web/src/lib.rs:relative`. The root is the project, always; the desktop's climb from one
opened file does not exist here. **An open is staged**: the host `stage`s each file,
and `open_document` takes the staged set as a fresh `MemFiles`, refuses a path it does
not hold in `document::not_a_file`'s words, picks the main with `discover_main`, and
calls `Preview::open` and `load`. With nothing staged it reopens the open project, as
the desktop's Open of a file in the folder already open does. `set_main` is
`Preview::ask_main`, then — granted — the same files reopened on that main, which
resets the counters as the window's `setMain` `clear()` expects. `create_file` refreshes
the panel itself, where the desktop leaves that to its watch. `trash_file` hands
`document::remove_file` to `Preview::trash`, and **the removal is permanent**; the
window still says "Move … to the Trash", which `ltr-001` OQ-7 holds.

`status` fills `appearance` from the session's own field and **`web` with `null`,
always**: images by URL are not fetched in a browser, so the line that offers a fetch
never appears and a URL image is refused in `md2pdf-core`'s own sentence.
`current_pdf` restates `app/src/main.rs:current_pdf`'s fallback, "the page is out of
date", word for word, and `save_as_path`'s "no document is open" likewise.

**A download is a save outside the project, and it never reaches the crate's
`save_as`.** The host's `dialog.save` answers a token, `download:<name>`;
`web/src/lib.rs:downloaded` recognises it before any crate call — a `MemFiles` would
take it for a root-relative name, write it and move the pane — and refuses a path
without one in "a browser saves a copy only as a download". `save_as` then answers
`project/src/preview.rs:Preview::download`: the buffer's bytes and `downloaded <name>`,
with `saved`, `divergence` and `edited` left alone, so the buffer is still dirty and the
next row click is refused in `SWITCHING`'s words. `export` answers `exportable`'s bytes.
Both cross as a `web/src/lib.rs:Download`, which `worker.mjs` flattens to
`{bytes, receipt}`.

## The worker and the host

**Everything that compiles runs in `web/host/worker.mjs`**, because a compile on the
main thread is a frozen caret every 300 ms of typing. It instantiates the module once,
answers `{id, value}` or `{id, error}`, and transfers byte buffers. **Messages are
answered in arrival order** — every session call is synchronous behind one awaited
promise — which the host relies on: an `edit` posted before a `compile` is taken first.

**`web/host/host.mjs` is the surface**, and it answers what only a page can:

- **The debounce.** `edit` stores the text and compiles nothing; the host sends
  `compile` 300 ms after the last one — the desktop's `TYPING_DEBOUNCE` — and emits
  `rendered`. A compile still waiting is flushed first by `save`, `save_as`,
  `export_path` and `export`, since `saveDocument` sends `edit` and `save` back to back.
- **The events no loop sends.** `rendered` follows every compile and every session call
  that changes what the window draws — `open_document`, `set_main`, `set_edited`,
  `trash_file`, `discard`, `save`, `create_file`, `save_as`, `export`,
  `set_appearance` — once it succeeds; a rejection reaches `fail` and announces nothing,
  as on desktop. `opened` follows a `hashchange` that seeds a project.
- **The keyboard.** A capturing `keydown` takes `metaKey` on macOS and `ctrlKey`
  elsewhere: ⌘O, ⌘S, ⇧⌘S and ⌘B are the desktop menu's own. **⇧⌘E is the host's own**:
  export is a desktop menu item with no accelerator and no button in the window, and a
  tab has no menu, so without a chord a reader could not export at all. ⌘L is the
  browser's address bar, and `view-lines` keeps its footer button.
- **The dialogs.** `dialog.open` is a hidden `<input type="file" accept=".md">`: its
  one file is `unstage`d-then-staged alone and answered as `/project/<name>`; a cancel
  answers `null`. `dialog.save` is `window.prompt`, seeded with the file name of the
  `defaultPath` the window passed, answering the token or `null`.
- **The downloads.** A `Blob`, an `<a download>` clicked and removed, the object URL
  revoked a minute later.
- **The title.** Each `status` sets `document.title` to the file name of `edited`, as
  the desktop's `app/src/document.rs:title` does from Rust.
- **The appearance.** Kept in `localStorage` under `letur.appearance`, one global key,
  and handed to the session before the window's first `status`; a blocked store costs
  only persistence.

**The seed opens an example as a project.** On `#example=NAME` — or `caption-table`
with no hash — the host fetches `../index.html`, parses it with `DOMParser`, selects
`script[data-example="${CSS.escape(NAME)}"]`, and stages its text as `document.md`
beside every `script[data-asset]` under its own name. The bytes are the elements' text
through `TextEncoder`, the bytes the landing page's test compiles. `pending_open`
answers `/project/document.md` once, after the fetch settles, then `null`; an unknown
name seeds nothing, and the window starts empty with Open… offered.

## The site, the lockfile and the check

`web/assemble.sh` builds `_site/`: the landing page at the root; at `app/` the copied
window, `app/dist/pdfjs/`, `web/host/` and `web/pkg/`, with `pkg/.gitignore` removed so
the Pages upload keeps the module. It dies unless the window has exactly one `</head>`
and the module has been built.

**`web/Cargo.lock` was seeded from the published `md2pdf-cli`'s** before the new
dependencies resolved, so every package the two share is at the CLI's version — 391
names, none differing, on 2026-09-26, where 38 had. The engine's version alone is not
the graph: Typst breaks lines with `icu_segmenter`'s compiled data. `web/Cargo.toml`
takes `letur-project` by path and `serde_json`, and keeps its empty `[workspace]`, so
the root workspace's gates are unchanged.

**`web/check.mjs` is `ltr-001` Phase 2's exit gate**, bun over the `playwright`
`web/package.json` pins at `app/package.json`'s version, one engine per process
(`--webkit` for the second), each clause in a fresh context. It refuses to run unless
`md2pdf --version` reports the `md2pdf-core` version `web/Cargo.lock` resolves and the
two lockfiles agree on every shared package; it checks the copied window is the source
plus the one line; and it serves `_site` on 127.0.0.1 as Pages does — `/app` redirected,
`.wasm` as `application/wasm`. Its clauses: the landing page requests no `.wasm`; every
example opens with its source in the pane, one through a `hashchange`; **every `ok`
example's PDF equals what the pinned `md2pdf` writes, byte for byte**; the three bars
hidden on an `ok` example; each refusal's exact sentence; the debounce bumping
`revision` within 2 s of typing and holding still after; create and trash through the
panel; the `saved` receipt; the download that moves nothing; the export's bytes; Open…;
the default seed and title; the appearance surviving a reload; and no uncaught page
error, counted by listeners installed with `addInitScript`. **Its status reads are
polled from Node**: `waitForFunction` takes an async predicate's promise for truthy and
passes at once. **Run 2026-09-26: every clause passes in Chromium and in WebKit**, with
`md2pdf` 0.4.0.

## The module, and what it costs

`web/pkg/letur_web_bg.wasm` is **33,239,606 bytes** raw and **9,922,721** under
`brotli -q 11`, measured 2026-09-26 on rustc 1.97.1 and wasm-pack 0.14.0, beside the
commit before `ltr-001` Phase 2 rebuilt the same day on the same toolchain —
33,165,169 and 9,900,913, which reproduced `rules/web-demo.md`'s last record exactly:
**+74,437 raw, +21,808 over the wire**, the session, `letur-project` and `serde_json`.
The release profile is `web/Cargo.toml`'s — size over speed. The record is the
requirement, never a ceiling. **The price moved rather than grew**: the landing page no
longer pays it; a visitor pays it when they choose to write.

## The build and the deploy

`.github/workflows/pages.yml` runs `wasm-pack build --target web --release` in `web/` and
then `web/assemble.sh`, on a push to `main` touching `web/**`, `project/**` (taken by
path), `app/dist/**` (the page copied) or the workflow. **The module is never
committed** — ten times this repository, and git history is permanent — so it is built in
the job and handed straight to the Pages artifact. `web/` is not a workspace member, so
nothing here changes what a phase's exit gate runs, and nothing here runs one:
`web/check.mjs` is a local check before a push.
