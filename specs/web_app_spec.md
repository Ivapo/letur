---
id: ltr-001
title: web-app
note: >
  The published page splits in two: a landing page that shows the dialect and loads no
  module, and Letur itself in a browser tab — the desktop front end unchanged, answered by
  a browser host over a project held in the browser's own storage, or, in Chromium, a
  folder on the reader's disk.
status: accepted
last_updated: 2026-09-25

phases:
  - name: "Phase 1 — the rules the window answers through leave the desktop crate"
    reviewed: 2026-09-25
    shipped: null
    cut: null
    by: null
  - name: "Phase 2 — two URLs, and the app is Letur's own window over a project in memory"
    reviewed: null
    shipped: null
    cut: null
    by: null
  - name: "Phase 3 — the project persists in the browser's own storage"
    reviewed: null
    shipped: null
    cut: null
    by: null
  - name: "Phase 4 — a folder on the reader's disk, in the browsers that can open one"
    reviewed: null
    shipped: null
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: [mpdf-003, mpdf-009, mpdf-010]
reference: >
  Typst's web app is still the two-pane shape, and still nothing more: it is a hosted
  editor with accounts and a server, and `mpdf-001` §1.1 refuses servers permanently.
  The browser APIs this builds on are the WHATWG File System standard's
  `navigator.storage.getDirectory()` (the origin-private file system, every current
  engine) and the File System Access API's `showDirectoryPicker()` (Chromium only). Both
  hand back a `FileSystemDirectoryHandle`, which is the fact §2 is built on. Nothing of
  either is adopted beyond the calls.
---

# web-app

## 1. Goal

**Put Letur in a browser tab, and give the dialect a page of its own in front of it.**
Today `https://ivapo.github.io/letur/` is one page doing two jobs: it argues for the
dialect with twelve examples, and it carries a textarea and a PDF pane beneath them so a
click can prove each one. Both jobs are done well, and the page is still not what a
visitor who wants to *write* is looking for. The textarea is not Letur — it has no file
panel, no pages drawn by `mpdf-009`'s renderer, no caret-follows-page, no project — and
`mpdf-006` §1.2 says it never will be.

The end state, as a visitor meets it:

```
https://ivapo.github.io/letur/            the landing page
  twelve rows: source │ the same parse, as HTML │ [open in Letur →]
  no module, no wasm — it is text and it renders at once

https://ivapo.github.io/letur/app/#example=caption-table
  Letur's own window: the file panel, the text pane, the drawn pages
  the example is document.md in a project beside pipeline.svg and refs.yml
  typing recompiles; the panel creates and deletes; Save writes

  Open… → ┌ Your projects (kept in this browser) ┐
          │  thesis            notes              │
          │  + New project     ⤓ Import a folder  │
          │  ─────────────────────────────────────│
          │  Open a folder on your disk…          │  ← Chromium only
          └───────────────────────────────────────┘
```

**The observable is the one this project exists to produce: the page the author sees
beside their text, redrawn as they write it** — now in a tab, over files of their own.
Phase 1 produces none and argues for itself; Phase 2 produces it over a project that
lives as long as the tab; Phase 3 makes the project survive the tab; Phase 4 makes it the
reader's own folder.

**"Looks just like Letur" is met by construction, not by imitation.** The app page *is*
`app/dist/index.html`, byte for byte, with one script injected ahead of it. §2 argues
why that is possible — the harness already does it — and why a lookalike would be the
wrong thing to build.

### 1.1 Why this is a new spec, and why it supersedes a phase of `mpdf-006`

The methodology's §6.1, worked in order.

- **Step 0 — a decision, not only code.** `mpdf-006` recorded "One page, and its text
  does not wait on the module (decision, recorded)" and non-goals of no persistence, no
  editor, and the user's own files parked. This reverses all three.
- **Step 1 — it removes shipped work of another spec.** `mpdf-006` Phase 2 — "every
  example is one click from a PDF" — shipped the panes beneath the list, the `load it`
  buttons that compile in place, the click-empties-the-pane rule and the `#status` line.
  Phase 2 here removes every one of them from the landing page. **The promise survives
  and the mechanism does not**: each example is still one click from a PDF, but the
  click is a link into the app. So the edge is phase-scoped, and `mpdf-006` keeps
  `status: accepted`.
- **What goes with the panes, stated rather than implied.** The rest of `mpdf-006`
  stays live on the landing page — the twelve examples, the byte rule and the test that
  holds them, the page-owned image and bibliography (Phase 3), the generated HTML column
  (Phase 4), the ordered-task-list refusal (Phase 5). **Not everything those phases
  shipped survives**: Phase 1's height model for the panes, Phase 3's sentence above the
  textarea and its `render` asset channel, Phase 5's typed-into-the-box check, and the
  `anchors` export `mpdf-006` Phase 2 kept as `mpdf-003` Phase 6's browser answer all
  go, because each belonged to the panes. They are consequences of cutting Phase 2, not
  cuts of their own phases, whose subjects stay live.
- **Why not a phase of `mpdf-003` instead.** `mpdf-003` owns the window, and this reuses
  the window — but `mpdf-003` §1.1 disowns the browser by name: "a different front end
  with a different file story, so it is a later spec". The file story *is* this spec's
  subject. And no subject spec has standing to cut another's shipped phase, which settles
  it: a new spec.

**The edge and the cut are written when Phase 2 ships, not now.** A `cut` date records
work removed, and nothing is removed while this is a draft — and the linter checks the
edge against its inverse, so the edge cannot precede the cut. Phase 2's close-out sets
`supersedes: [{id: mpdf-006, phases: ["Phase 2 — every example is one click from a
PDF"]}]` here and `cut` and `by: ltr-001` on that phase, and writes dated `CORRECTED`
notes beside `mpdf-006`'s "One page" decision and its §1.2 "No editor" non-goal. The
§1.2 persistence non-goal is corrected by Phase 3's close-out, the day persistence
arrives, and the "user's own files stay parked" non-goal by Phase 2's, since a file
input already brings one in.

### 1.2 Non-goals

- **No server, ever.** `mpdf-001` §1.1 stands. Pages serves static files; every byte of
  a project stays in the reader's browser or on the reader's disk. No accounts, no sync,
  no share link.
- **No second front end.** Nothing in `app/dist/index.html` changes for the browser's
  sake — no `if (web)`, no feature flag, no fork. A behaviour the browser cannot offer is
  the host's refusal, in the host's words. **The cost is stated**: the window's delete
  button reads "Move … to the Trash" and `TRASHING`'s sentence says the same, on a host
  that has no Trash. The window is not changed to fix a word; OQ-7 holds it.
- **No images by URL.** `mpdf-003` Phase 25 fetches them from Rust through `ureq`, and a
  browser fetch is bound by CORS, which most image hosts do not grant. The web session
  answers the `Status` field `web` with `null`, so the window's line never appears, and a
  URL image is refused in `md2pdf-core`'s own `Error::UnfetchedImage` sentence — the one
  the landing page has always printed for it. OQ-4 holds the question.
- **No watching.** The desktop app notices a file changed by another program; a tab over
  a disk folder does not (§2, "What a folder on disk costs"). No polling is added to
  pretend otherwise.
- **The landing page's prose is not rewritten here** beyond what the split forces — the
  button's words and a sentence pointing at the app. The stale construct count
  `rules/web-demo.md` logs is `mpdf-006`'s to fix.

## 2. Design

### Two URLs, and the landing page carries no module (decision, recorded)

`mpdf-006` chose one page because "two URLs would turn the click that settles the
argument into a navigation that re-pays the download". That was right while the landing
page needed the module; **it no longer does**. `mpdf-006` Phase 4 inlined the HTML
column as bytes, so every row renders with scripting off; the module was on the page
only to serve the panes. Take the panes away and the landing page is text: **9,900,913
bytes** of brotli'd wasm (`rules/web-demo.md`, 2026-09-25) stop being the price of
reading about the dialect, and become the price of choosing to write in it.

- `web/index.html` stays the landing page, at the site root. Its `<main>`, textarea,
  iframe, `#status` and module script go. Each row's `load it` button becomes an
  `<a href="app/#example=NAME">open in Letur</a>`, where `NAME` is the row's own
  `data-example` value — the name the test already holds unique.
- The app lives at `app/` under the site. It downloads the module once, when a visitor
  asks for it.

### The app is the desktop front end, answered by a browser host (decision, recorded)

`app/dist/index.html` talks to its backend through exactly one object:
`window['__TAURI__']`, read at module top level — `core.invoke` for the nineteen
commands `app/src/main.rs`'s `generate_handler!` registers, `dialog.open` and
`dialog.save`, and `event.listen` for eight events. **`app/harness/serve.mjs` already
serves a copy of that file with `app/harness/stub.mjs` injected into `<head>`**, and the
window runs in headless Chromium and WebKit under it; that is how every harness gate since
`mpdf-003` Phase 12 has been driven. So "can Letur's front end run in a browser" was
answered long ago. What was missing is a stub that is real.

That is the **host**: `web/host/host.mjs`, a module injected into `<head>` of a copy of
`app/dist/index.html` by the rule `serve.mjs` follows — **copy, never edit**, since
`app/typecheck.mjs` dies unless the file holds exactly one module script. It supplies
`window.__TAURI__` and proxies each command to the worker below.

**A lookalike was refused, and the reason is the harness's own finding.** A second page
styled after Letur would drift from the first on the next phase of `mpdf-003`, and
nothing would notice. One file with two hosts cannot drift; a command the host has not
learnt is a rejection the window already prints through `fail`.

### Every command and event, and who answers it (decision, recorded)

**A path in the browser is virtual**: `/project/<root-relative path>`, where `/project`
is the one project Phase 2 holds (Phase 3 makes it `/<project name>`). **The root is the
project directory, always** — the desktop climb, `app/src/document.rs:project_root`,
exists to guess a root from one opened file, and a browser project names its own root.

**Downloads are the host's; everything with a rule is the session's.** The window's
save-as and export flows are fixed — `edit`, then `save_as_path`, then `dialog.save`,
then `save_as(path)`, then the receipt; `export_path`, `dialog.save`, `export(path)`.
The host's `dialog.save` returns a **download token**, `download:<file name>`, taken
from the `defaultPath` the window passed; the command that receives it produces the
bytes and the host downloads them. So the refusals stay Rust's and only the delivery is
the browser's.

| The window asks | Answered by | What it does in the browser |
|---|---|---|
| `open_document(path)` | session | opens the project at `/project` with `path` as the opened file; `discover_main` picks the main |
| `pending_open` | host | the seed's path once, if a seed is waiting (below), then `null` — never `undefined` |
| `document_text`, `edit`, `status`, `current_pdf`, `asset_bytes` | session | as desktop; `current_pdf` crosses as an `ArrayBuffer` |
| `set_main`, `set_edited`, `discard`, `save` | session | as desktop, `SWITCHING` and `SAVED` included; Phase 2's `save` writes the buffer into the map |
| `create_file`, `trash_file` | session | the map gains or loses the file; `TRASHING` as desktop; removal is permanent |
| `save_as_path`, `export_path` | session | desktop's suggested name, and `exportable`'s refusals, unchanged |
| `save_as(download:…)` | session, then host | the session returns the buffer's bytes and marks it saved; the host downloads them and answers the receipt `downloaded <name>`. The pane does not move — a download is not in the project |
| `export(download:…)` | session, then host | `exportable`'s bytes, downloaded |
| `set_appearance` | host | stored in `localStorage`, handed to the session so `Status.appearance` carries it, then `rendered` |
| `fetch_images` | host | rejects: `images by URL are not fetched in a browser` |
| `dialog.open` | host | Phase 2: a file input taking one `.md`, read into a fresh memory project; Phase 3: the project sheet |
| `dialog.save` | host | the download token above; `null` if the host's name prompt is cancelled |
| event `rendered` | host | emitted after every compile lands, and after `create_file`, `trash_file`, `set_main`, `set_edited`, `discard`, `save`, `set_appearance` — the desktop's watch loop announced these, and there is no watch loop here |
| event `opened` | host | emitted when a `hashchange` puts a new seed in the slot `pending_open` reads |
| events `open`, `save`, `save-as`, `view-files` | host | ⌘/Ctrl+O, ⌘/Ctrl+S, ⇧⌘/Ctrl+S, ⌘/Ctrl+B — the desktop menu's own accelerators; the window keeps "no `keydown` of its own", as its comment requires |
| events `export`, `view-lines` | — | not bound: `export` has no desktop accelerator, and ⌘L is the browser's address bar. Both have buttons |

**The debounce is the host's**, and the session gains the command the desktop keeps
implicit: `edit` stores the text and compiles nothing, as `app/src/preview.rs:Preview::edit`
does, and the host sends `compile` 300 ms after the last `edit` — or at once before
`save`, since `saveDocument` sends `edit` and `save` back to back. **The window's title**,
which the desktop sets from Rust, the host sets as `document.title` from the `main` and
`edited` fields of each status.

**The seed** runs in `host.mjs` on the main thread — `DOMParser` exists nowhere else. On
`#example=NAME` it fetches `new URL('../index.html', location)`, selects
`script[data-example="${CSS.escape(NAME)}"]` and both `script[data-asset]` elements, and
opens a project of `document.md` and the two files under their `data-asset` names,
`pipeline.svg` and `refs.yml`. The seed's path waits for `pending_open`, which the window
calls at startup (`takePendingOpen`). With no hash, the seed is the first `ok` example,
`caption-table`. **The landing page stays the one copy of every example** — a fourth
consumer of the element, beside the reader, the test and the HTML column.

### The rules are Rust, shared, and compiled twice (decision, recorded)

The host could reimplement the backend in JavaScript, and the web crate could
reimplement it in Rust. **Neither**: the panel's order, the main a project opens on, the
files a compile reads, the state the pane is in, the refusals `SWITCHING`, `TRASHING` and
`DIVERGED`, the `SAVED` receipt, `exportable`'s two sentences and the counters `revision`
and `reloaded` are `app/src/document.rs`'s and `app/src/preview.rs:Preview`'s, tested
there, and a copy would be a second implementation of every rule
`rules/desktop-project.md` and `rules/desktop-compile.md` describe, checked against none.

**The seam is I/O, time and threads.** `app/src/document.rs:render_with` already takes
the file read as a closure, which `mpdf-010` Phase 2 opened so the pane's buffer could
stand in for one file. Phase 1 widens that seam to the whole state machine:

- **A new workspace crate, `project/` (package `letur-project`)**, holding a `Files`
  trait — `read`, `write`, `remove`, `list` over root-relative paths — and an in-memory
  implementation, `MemFiles`, over a `BTreeMap<String, Vec<u8>>`.
- **What moves into it**, with its tests: from `app/src/document.rs`, `render_with`,
  `read_sections_with`, `read_assets_with`, `Unread`, `Pane`, `Render`, `Anchor`,
  `Entry`, `Kind`, `kind_of`, `order`, `merge`, `beside`, `under`, `spell`, `directory`,
  and `masters` and `discover_main` rewritten over `Files`; from `app/src/preview.rs`,
  `Status`, `State`, `Appearance`, the four sentence constants, and **`Preview`'s pure
  core** — the fields that are state (root, main, edited, tree, buffer, saved, the
  compile's outputs, the counters, stale, error, divergence) and the methods that decide
  (`state`, `status`, `entries`, `exportable`, `edit`, `take`, `absorb`), with every file
  operation going through `Files` and the compile's duration **passed in** rather than
  measured, since `std::time::Instant::now` panics on `wasm32-unknown-unknown`; and the
  rules `app/src/preview.rs:Session` holds that touch nothing but that state —
  `refused_while_dirty` and the decisions inside `set_main`, `set_edited`, `trash`,
  `discard`, `save` and `save_as` (its "saved as … in …" receipt included), with the
  sentences `Session` builds inline ("no document is open", "… is not a file in this
  project"); and from `app/src/document.rs`, the rule halves of `confined`, `create_file`
  and `asset_bytes` — what they refuse and in what words — over `Files`. From `app/src/remote.rs`, `Fetched`, `WebLine`, and `Web`'s
  state without `fetch` and `agent`.
- **What stays in `app`**: `Session`'s threads, the watch (`app/src/watch.rs` whole),
  the typing debounce, the claim channel, `Compile::run`'s clock, the disk `Files`
  implementation, the Trash, Application Support, `ureq`.
- **The property is checked, not asserted**: `project/clippy.toml` disallows, as an
  enumerated list since clippy matches paths and not modules, every `std::fs` function
  and `std::fs::File`, `std::time::Instant::now` and `std::time::SystemTime::now` (both panic on
  `wasm32-unknown-unknown`), `std::thread::spawn`, and
  `std::sync::mpsc::{channel, Sender, Receiver}`, and `cargo clippy -p letur-project -- -D clippy::disallowed_methods
  -D clippy::disallowed_types` over non-test code is in Phase 1's gate. A wasm build alone
  would not catch `std::fs`, which compiles for `wasm32-unknown-unknown` and fails at
  run time.

`app/src/preview.rs:Preview`'s exact split is the plan's to make; the spec's constraint
is the one above — **after Phase 1, every sentence the window shows and every counter it
reads is produced by `letur-project`**, except the tail an I/O error's own `Display`
contributes, which is the `Files` implementation's, and Phase 2's web session is `MemFiles`, a
`Preview` and a clock.

**One sentence differs, and it is priced rather than hidden.** `read_assets_with`'s
"cannot read … for the image …" ends with the `io::Error`'s `Display` and the file's
path. On desktop that is an absolute path and "No such file or directory (os error 2)";
`MemFiles` answers with the root-relative path and `io::Error::new(NotFound, "No such
file or directory")`. The frame is the same function's; the tail is the host's.

### Status crosses as JSON, and that is what makes one test hold both hosts (decision, recorded)

`web/src/lib.rs` took the page's two files as `(path, bytes)` twice to avoid a
dependency for a `Vec<Vec<u8>>`. A project has any number of files, so the boundary
changes — and **`serde-wasm-bindgen` was considered and refused**: by default it turns
`Option::None` into `undefined`, and `app/dist/index.html:report` hides the error,
divergence and web bars on `=== null`. Every status would show three empty bars, and the
typedef test — `app/src/preview.rs`'s comparison of the `@typedef Status` field names
against a `serde_json` serialization — could not see it.

So **every structured answer crosses as a JSON string from `serde_json`**, the same
serializer Tauri's IPC uses on the desktop, and the host `JSON.parse`s it. Bytes
(`current_pdf`, `asset_bytes`, file contents) cross as `Uint8Array`. The cost is stated:
`web/Cargo.toml` gains `serde_json`, already in `web/Cargo.lock` through `md2pdf-core`,
and `letur-project` by path. With `Status` shared and the serializer shared, **the desktop test holds
the web host too**, and Phase 2's gate adds the check the typedef test cannot make: on an
`ok` example the three bars are hidden.

### Everything runs in a worker (decision, recorded)

A compile is `md2pdf-core` calling Typst; on the main thread that is a frozen caret every
300 ms of typing. So **the module and the session live in a dedicated worker**,
`web/host/worker.mjs`; `host.mjs` is the `__TAURI__` surface, the debounce, the seed, the
keyboard and the downloads, and proxies the rest. The worker times each compile with
`performance.now()` and hands the duration to the session. It is also where Phase 3's
storage lives, because the origin-private file system's fastest write path,
`createSyncAccessHandle`, exists only in a worker.

**Reads are synchronous because the project is in memory.** `render_with`'s closure is
synchronous and no browser storage API is. So a project is read into `MemFiles` when it
opens, and every write the window makes goes to the map first and the store second
(Phase 3). A document project is markdown, a few images and a bibliography: megabytes,
not gigabytes. **A project above 64 MB refuses to open** with a sentence saying so; the
figure is a guard, not a measurement, and OQ-3 asks for one.

### One directory handle, two places it can come from (decision, recorded)

`navigator.storage.getDirectory()` returns the root of the origin-private file system as
a `FileSystemDirectoryHandle`. `showDirectoryPicker()` returns a folder the reader chose
as a `FileSystemDirectoryHandle`. **Same type, same methods** — `values()`,
`getFileHandle`, `getDirectoryHandle`, `removeEntry`. So the worker's store is one
module over a handle, and where the handle came from is the only branch:

- **Browser storage** (Phase 3, every engine). Each project is a directory under the
  origin root. It needs no warning and no permission, and it is private to this browser
  on this device. The host calls `navigator.storage.persist()` once, so the browser does
  not evict it under storage pressure; a reader who clears site data still loses it, and
  the project list says so in one line.
- **A folder on disk** (Phase 4, Chromium only). Feature-detected — `'showDirectoryPicker'
  in window` — and absent from the sheet elsewhere, rather than present and failing. The
  handle is kept in IndexedDB (the one store a handle survives a reload in), and the
  browser's own permission prompt is re-asked on each visit.

**Getting files in and out, in every engine.** Import is an `<input type="file"
webkitdirectory>` or a folder dropped on the sheet, copied into a new browser-storage
project. Export is a `.zip` of the project, written by a store-only zip writer in the
host — local headers, CRC-32, a central directory, about sixty lines — rather than a
vendored library, since nothing here needs compression and `app/dist/` is embedded in
the desktop binary where a web-only dependency must not land.

### What a folder on disk costs, and the reader is told before it is chosen (decision, recorded)

Opening a disk folder is the one act in this spec that can destroy a reader's work, so
**the host shows a sheet before the browser's picker**, and the reader confirms it:

> **You are about to work on files on your disk.**
> Letur will read every file in the folder you choose. **Save** writes your changes into
> those files. **Deleting a file in the panel deletes it permanently** — there is no
> Trash here, and nothing asks first. If another program changes a file while it is
> open, Letur will not notice until you open the folder again.

Each sentence is a difference from the desktop app, and each is the browser's rather
than a choice:

- **Nothing is written until Save.** The window's buffer-and-save model is the desktop
  one unchanged; the host adds nothing that writes on its own.
- **No Trash, and no confirmation.** The window confirms no delete by design — its
  `trashFile` says so, as Finder does not confirm a move to the Trash — and the desktop
  can afford that because `app/src/document.rs:move_to_trash` is undoable. `removeEntry`
  is not. This sheet is therefore the only warning a disk-folder reader gets, given once
  per folder; OQ-7 asks whether that is enough.
- **No watching.** `FileSystemObserver` exists in Chromium behind an origin trial and in
  no other engine. OQ-5 holds it.
- **Confinement is the browser's.** `app/src/document.rs:confined` canonicalises to stop
  a `../` leaving the root. A directory handle cannot reach its parent at all, so the
  property holds by construction; `MemFiles` still refuses a path with a `..` component,
  in `confined`'s terms, so the window sees the same refusal on every host.

## 3. Open questions

- **OQ-1** — Does Safari's origin-private file system take writes through
  `createSyncAccessHandle` in a worker, and does `createWritable` exist there at all?
  The worker design needs only the first. *(answerable by measurement — blocks Phase 3;
  answer during that phase's review round in WebKit under the existing Playwright rig.)*
- **OQ-2** — Does the landing page link to a desktop download? There is no signed
  release to link to today. *(needs-input — Phase 2; the default is no link, and the
  phase builds with the default.)*
- **OQ-3** — What does a large project cost in a tab? The 64 MB refusal is a guard.
  *(deferred by evidence — measure the showcase and one image-heavy project in Phase 3.)*
- **OQ-4** — Images by URL: the host could fetch through the browser where the site
  grants CORS. *(design call — deferred; no phase here.)*
- **OQ-5** — `FileSystemObserver` for disk folders, once it leaves origin trial.
  *(deferred by evidence — reopen as a phase when it ships in stable Chromium.)*
- ~~**OQ-6** — The module is still named `md2pdf_web_spike`. Rename it?~~ **RESOLVED
  2026-09-25 (review round 1):** renamed to `letur_web` in Phase 2 — `web/Cargo.toml`'s
  package, so `web/pkg/letur_web_bg.wasm` — since the workflow, the page and the module
  are all rewritten around it in that phase anyway.
- **OQ-7** — The window says "Move … to the Trash" on hosts with no Trash, and a disk
  folder's delete is permanent and unconfirmed. Should the host keep a recoverable copy
  (a `.letur-trash/` in browser storage, never in the reader's folder), or is Phase 4's
  sheet enough? *(design call — blocks Phase 4, not Phase 2 or 3, whose files live only
  in the browser.)*

## 4. Implementation phases

Strictly sequential: Phase 2's session is Phase 1's crate, Phase 3 stores what Phase 2's
session holds, and Phase 4 is a second source for Phase 3's store.

### Phase 1 — the rules the window answers through leave the desktop crate
*Produces the observable: no — and that is the argument for it.* The window a desktop
author sees is identical before and after, by gate. What it buys is that Phase 2's web
session is **one implementation of the rules rather than two**: without it, every
refusal, receipt and counter in §2's table is either rewritten in `web/src/lib.rs`,
unshared and checked by nothing, or the web host answers differently from the desktop in
ways no test sees. Bundled into Phase 2 it made a single pass of a refactor across
~9,000 lines of desktop Rust *and* a new front end, with two blast radii in one gate.

- **Scope:** create `project/` (`letur-project`) as a workspace member, with `Files`,
  `MemFiles` and everything §2 "The rules are Rust" lists as moving, their tests beside
  them; `project/clippy.toml` as §2 states. `app` depends on it and implements `Files`
  over the disk. `Preview` is split as §2 constrains, the part holding the watch's claim
  channel and the clock staying in `app`. **Add `MemFiles` tests** for what the web will
  rely on and no desktop test exercises over memory: `SWITCHING` and `TRASHING` refused
  while the buffer differs from the save, `SAVED` after a save, both `exportable`
  refusals, `revision` rising on a compile that produced a page and not on one that did
  not, and a `..` path refused.
- **Exit gate:**
  1. `cargo test --workspace` passes, and reports **no fewer tests** than on the commit
     before the phase (the count read from the same command's summary lines, both runs
     recorded in the review record), and every test name that left `app/src` appears
     under `project/src` — the list of moved names recorded beside the counts, so a
     dropped test cannot hide behind a new one. This is the blast radius: the Rust that moved.
  2. `cargo clippy -p letur-project -- -D clippy::disallowed_methods -D
     clippy::disallowed_types` passes, and `cargo build -p letur-project --target
     wasm32-unknown-unknown` succeeds.
  3. `cargo build -p letur --features driven`, then `bun app/driver/drive.mjs` passes —
     the build first, since the driver launches whatever `target/debug/letur` holds. It
     drives the real binary in WKWebView and covers the chrome, `open_document`,
     `set_appearance` and `fetch_images` end to end; save, switch and trash are guarded
     by the `Session` and `MemFiles` tests under clause 1, not by this. macOS only, as
     the driver is. `bun app/typecheck.mjs` passes. (`app/harness/checks.mjs` drives the page over a
     stub and never runs this Rust, so it is not part of this gate.)
  4. `grep -rn 'const SWITCHING\|const TRASHING\|const DIVERGED\|const SAVED' app/src`
     returns nothing, and the same over `project/src` returns four.
- **The typedef test stays in `app`**, importing `Status` from `letur-project`: it reads
  the page with `include_str!("../dist/index.html")`, a path that only resolves from
  `app/src`.
- **Close-out:** every rule citing a moved symbol is corrected against its sources:
  `rules/desktop.md` (`render_with`, `read_assets_with`, `read_sections_with`,
  `Anchor`), `rules/desktop-panes.md` (`kind_of`), `rules/desktop-panel.md` (the
  entries and kinds), `rules/desktop-project.md` and `rules/desktop-compile.md` — each
  **adding** `project/src/…` to its `sources` rather than swapping it in, since most of
  what `desktop-compile.md` covers stays in `app/`. **Shipped specs are a stated
  exception to §6.1's "leave it"**: `specs/desktop_app_spec.md` and
  `specs/file_panel_spec.md` cite about fifty moved symbols by `app/src/…` path, and
  `spec-lint` makes a symbol absent from a file that still exists an error
  (`CIT_SYMBOL_ABSENT`), so leaving them fails the commit that ships this phase. The
  close-out therefore rewrites **the path half of those backticked `file:symbol`
  citations and nothing else** — no claim, no prose, no decision — and the check is
  that `git diff --word-diff` over the two specs shows only `app/src/…` → `project/src/…`
  substitutions inside backticks. The same path rewrite applies to this spec's own §2,
  which cites `render_with` and `Preview` by their `app/src` paths. `spec-lint` then
  reports zero errors. User-facing documentation: none needed — nothing a user sees
  changes. `CLAUDE.md`: none needed.

### Phase 2 — two URLs, and the app is Letur's own window over a project in memory
*Produces the observable: yes — Letur's window, drawing the page beside the text as it
is typed, in a tab.*

- **Scope:**
  1. **The web session.** `web/Cargo.toml` is renamed to `letur_web` (OQ-6) and depends
     on `letur-project` by path and on `serde_json`. `web/src/lib.rs` becomes a
     `#[wasm_bindgen]` `Session` over `MemFiles` and `Preview`, with one method per
     session row of §2's table plus `compile(elapsed_ms)`, structured answers as JSON
     strings and bytes as `Uint8Array`. `render` and `anchors` go, with `mpdf-006`'s
     panes.
  2. **The host.** `web/host/host.mjs` and `web/host/worker.mjs`, answering every row of
     §2's table: the proxy, the debounce, the `rendered` and `opened` events, the
     keyboard, the downloads and their receipts, the title, the seed.
  3. **The landing page.** `web/index.html` loses its panes, textarea, `#status` and
     module; each row's button becomes the link §2 names. `app/tests/page_examples_test.rs`
     is **unchanged** — it asserts nothing about the buttons, and an `<a href>` carries
     no `data-example="`, so its twelve-element count still holds.
  4. **The site.** `web/assemble.sh` builds `_site/` — `index.html`; `app/index.html` (a
     copy of `app/dist/index.html` with `<script type="module" src="host/host.mjs">`
     inserted as the first child of `<head>`), `app/pdfjs/`, `app/host/`, `app/pkg/` —
     and `.github/workflows/pages.yml` calls it, with `app/dist/**` and `project/**`
     added to its trigger paths and its header comments ("This builds `web/` alone", "The
     workspace … is not touched") corrected, since `web/` now takes `project/` by path.
- **Exit gate:**
  1. `cargo test --workspace` passes; `app/tests/page_examples_test.rs` passes unchanged.
  2. **The reference compiler is pinned**: `cargo install --locked md2pdf-cli --version
     <V>`, where `<V>` is the `md2pdf-core` version `web/Cargo.lock` resolves (0.4.0
     today), and the driver refuses to compare unless `md2pdf --version` reports `<V>`.
     The machine this spec was reviewed on carries 0.1.3, which refuses a task list.
  3. `web/check.mjs` (bun and Playwright, one engine per process, as
     `app/harness/checks.mjs` records a second launch in one process hangs) serves
     `web/assemble.sh`'s `_site` on `127.0.0.1` — redirecting `/app` to `/app/` as Pages
     does, and serving `.wasm` as `application/wasm` — and in **Chromium and WebKit**
     checks: (a) the landing page requests no `.wasm`; (b) for each of the twelve
     examples, the link opens the app with the text pane holding that example's source
     byte for byte; (c) each `ok` example draws a page, and the bytes `invoke('current_pdf')`
     returns equal what the pinned `md2pdf` writes for that source with `pipeline.svg`
     and `refs.yml` beside it under their `data-asset` names; (d) on each `ok` example
     the error, divergence and web bars are hidden; (e) each `error` example shows the
     sentence the landing page prints for it; (f) on `caption-table`, appending the line
     `More text.` bumps `revision` by at least one within two seconds; (g) `create_file`
     through the panel adds its row, and `trash_file` removes it; (h) ⌘/Ctrl+S on an
     edited buffer shows the `saved` receipt.
  4. `_site/app/index.html` equals `app/dist/index.html` except for the one inserted
     line, by `diff`.
  5. `web/pkg/letur_web_bg.wasm`'s raw size and its size under `brotli -q 11` are
     recorded against 33,165,169 and 9,900,913, the method `rules/web-demo.md` uses.
- **Close-out:** `rules/web-demo.md` narrows to the landing page, keeping
  `web/index.html` and `app/tests/page_examples_test.rs` as sources; a new
  `rules/web-app.md` covers the session, the host, the worker and the site, with
  `web/src/lib.rs`, `web/host/host.mjs`, `web/host/worker.mjs`, `web/assemble.sh` and
  `.github/workflows/pages.yml` as sources, and carries gate 5's sizes. The README's web
  section names both URLs. This spec takes the `supersedes` edge, `mpdf-006` Phase 2
  takes `cut` and `by: ltr-001`, and the `CORRECTED` notes §1.1 assigns to this phase are
  written. `CLAUDE.md`: none needed.

### Phase 3 — the project persists in the browser's own storage
*Produces the observable: yes — the same window, over a project that is still there
tomorrow.*

- **Scope:** `web/host/store.mjs` in the worker: a store over one
  `FileSystemDirectoryHandle`, reading a project into `MemFiles` on open and writing each
  Save, create and delete through to it. Browser storage is the handle: each project a
  directory under `navigator.storage.getDirectory()`, and `navigator.storage.persist()`
  asked once. Virtual paths become `/<project name>/…`. `dialog.open` becomes the project
  sheet: the list, New project, Import a folder (`webkitdirectory`, or a drop), and
  per-project Export (the store-only `.zip`). An `#example=` seed becomes a new project
  named after the example, never overwriting one. The 64 MB guard. The main-file
  override and the appearance, which the desktop app keeps in Application Support, live
  in `localStorage`, keyed per project.
- **Exit gate:** `web/check.mjs` gains, in **Chromium, WebKit and Firefox**: a project
  created, edited, saved and reloaded comes back with the same files and the same main;
  an unsaved edit does not survive a reload; import of a three-file folder then export
  produces a `.zip` whose entries equal the imported bytes; a deleted file is gone after
  reload. OQ-1 is resolved before this gate is written.
- **Close-out:** `rules/web-app.md` gains the store; the README says where a web
  project lives and how to get it out; `mpdf-006` §1.2's persistence non-goal takes its
  `CORRECTED` note.

### Phase 4 — a folder on the reader's disk, in the browsers that can open one
*Produces the observable: yes — the window over the reader's own files, the case the
desktop app exists for, with no install.*

- **Scope:** the sheet's third entry, present only where `showDirectoryPicker` exists;
  the warning sheet of §2, confirmed before the picker opens; the handle posted to the
  worker and handed to the same store as Phase 3's; kept in IndexedDB so a reload offers
  the folder again behind the browser's own permission prompt. OQ-7's answer.
- **Exit gate:** in **Chromium**, with the picker answered by Playwright's file-chooser
  interception over a scratch folder: the warning shows before the picker and Cancel
  opens nothing; Save changes the file on disk and nothing else in the folder changes;
  a delete removes the file on disk; a reload re-offers the folder. In **WebKit and
  Firefox**, the entry is absent from the sheet.
- **Close-out:** `rules/web-app.md` gains the disk source and the warning; the README
  states which browsers can open a disk folder and what the reader is told.
