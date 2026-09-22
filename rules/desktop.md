---
title: desktop
sources:
  - app/Cargo.toml
  - app/tauri.conf.json
  - app/capabilities/default.json
  - app/src/main.rs
  - app/src/document.rs
  - app/src/preview.rs
  - app/src/remote.rs
  - app/dist/index.html
  - app/tsconfig.json
  - app/package.json
  - app/driver/drive.mjs
covers: >
  the desktop app: the crate and its files, the window and its menu, the five
  submenus and the six ids the one filter enumerates, the two items that name a
  view and carry no state, the two
  titles an open sets, the commands and the signal between them, the file I/O the
  app owns and the two read passes one closure serves, the fourth reader of a
  path the author did not name in a dialog, the first write to one and the first
  delete of one, the second writer of a path the author did name, the confinement
  it does not share and the one its caller asks after the fact, the receipt the
  two saves answer with and the field it deliberately is not, the three answers the
  asset list gives and the filter reads, the URL whose bytes come from memory and
  whose path reaches no watch, the project this file hands to a rule of its own
  and the compile it hands to a second, the panel's status fields and the union
  built where no disk is read, the three values the state holds where it held one
  and the two of them that are now two files, the second file beside the store and
  the one thing in it that is not about a folder, the third beside them for the
  sites a folder allows, the nineteenth command and the two halves one of them is,
  the second
  setter called where capabilities do not apply, the field composed in two places
  and the seam that holds them, the buffer that compiles and the closure it rides,
  the export and its two
  refusals and the file it names, the errors it puts on the page, the bundle and the document association that launches it,
  the vendored renderer the front end imports and what embedding it costs, the
  declarations it is type-checked against and where they may not live, the node
  manifest the crate carries for its test rig and the build that reads neither,
  the window that is built hidden and shown a hook later, the configuration
  facts a build enforces, and the server one build carries behind a feature,
  the capability it looks to want and does not, and the setter it offers where
  the page is refused one
# **The split, taken.** 695, then 715 at `mpdf-003` Phase 21, then 730, at which
# point this file's own note said the next phase needing room would cut it at the
# seam between the watch loop and the compile rather than raise it a fourth time.
# `mpdf-003` Phase 25 needed that room and made the cut:
# `rules/desktop-compile.md` took `## The watch loop`, `## The state` and `## The
# rule an external change runs`, and the fetch went in beside them. What is left
# is the crate, the window, the file I/O, the project, the session, the export,
# the bundle and the document association — some 540 lines under a cap that binds
# again.
max_lines: 600
generated: 2026-09-22
---

# Desktop

A macOS window that shows the PDF while you write it. `letur` is the second
wrapper around `md2pdf-core`, beside `md2pdf-cli`, and it calls no other code of
this project's own; **`core` gained one function for it, in Phase 6, and nothing
in any other phase** — `core/src/lib.rs:md_to_pdf_with_anchors`, which is
additive and left every existing signature alone. Today it opens one *project*
at a time — the folder a document sits in, with one file under it set as the one
that compiles — lists that folder's files down the left, holds the compiled
file's text in a pane beside the page, recompiles when the typing stops, opens
the page on the heading the caret is under, says what state the page is in, saves
the text back, and writes the page to a file the user names. It bundles into an
`.app` and a `.dmg`, and a `.md` double-clicked in Finder launches it on that
file's project. **The bundle is unsigned**, a credential this machine does not
hold rather than a step skipped; `specs/desktop_app_spec.md` OQ-8 says what that
costs.

**The fixtures this suite reads are its own, frozen at `mpdf-011` Phase 1**: the
eleven documents and images `app/src/*.rs` names under `tests/fixtures/`, and the
engine's `samples/` tree under `tests/fixtures/samples/` as `article.md`,
`check.svg`, `pipeline.svg` and `showcase/` — that shape because
`document.rs:a_master_in_a_subdirectory_is_not_this_roots_master` reads the
directory and asserts no master sits at its top. Copies on purpose: these tests
assert what the *app* does with a document, so a fixture that stopped tracking
the dialect costs them nothing, where one that kept tracking it would move this
app's measured numbers on every dialect phase.

**The pane's text is what compiles**, and several claims below turn on it. The
file beside it need never have held that text. **The pane holds exactly one file
and it need not be the one that compiles**: the master's text and every other
section come off the disk, and the buffer stands in for the one file the pane has.

## The crate

**A hundred and sixty-four committed files, and 152 of them are third-party
declarations and modules that nothing here builds**: `app/types/pdfjs/` is the
149-file, 824 KB `types/` tree of the same `pdfjs-dist` 6.2.108 tarball as the
two vendored `.mjs` modules under `dist/pdfjs/`, which with their shared
Apache-2.0 licence make three more. Of the twelve that are this project's,
`src/` is `main.rs` and three modules. `Cargo.toml` names `tauri` 2.11.5,
`tauri-plugin-dialog` 2.7.2, `notify` 8.2.0, `serde` 1.0.229 with its `derive`
feature, `serde_json` 1.0.151 and `objc2-foundation` 0.3.2 for `NSError`,
`NSFileManager`, `NSString` and `NSURL`, with `tauri-build` 2.6.3 under
`[build-dependencies]` and **no dev-dependencies at all** — `serde_json` was one
until the store made reading JSON the app's job rather than a test's. **Both of
those last two are already in `Cargo.lock` at their version by way of `tauri`,
so neither adds a crate to the tree**, and that fact is what picked
`objc2-foundation` over the `trash` crate, whose macOS implementation is the
same `NSFileManager` call. There is no `target.'cfg(...)'` table: this binary is
macOS only by construction and `src/main.rs` says so. **`md2pdf-core` is a
registry dependency at `"0.3"`** since `mpdf-011` Phase 3 published it, a `{ git, rev }`
table staying the seam for any publish Letur cannot wait for. A patch lands with no
commit here — that spec's OQ-5, and what its reopening condition watches — and a minor,
`0.x`'s breaking release, takes one. Its diagram renderer `merman` sets a Rust 1.95 floor
and links three of the binary's four MPL-2.0 crates. **This crate owns its
`version`** — `0.1.0` inline rather than `version.workspace`, because a library
versions by API and a product by release; `app/tauri.conf.json` carries **no**
`"version"` key, Tauri falling back to this one, so the two cannot disagree,
while `edition`, `license` and `[profile.release]` are still the root's.
`build.rs` calls `tauri_build::build()`. `tauri.conf.json` sets `app.withGlobalTauri: true`, which
puts the API on `window.__TAURI__` and is what removes the bundler and the node
toolchain entirely — `build.frontendDist` is `dist`, a directory of static files,
so the whole app is one Cargo build. **The vendored `pdf.js` does not spend
that**: `pdfjs-dist` ships browser-ready ES modules, so `app/dist/pdfjs/` is two
more static files the page imports and not a dependency anything builds.
`generate_context!` walks `frontendDist` recursively with no allowlist, so a new
subdirectory is embedded with no configuration at all, and `.mjs` is served as
`text/javascript`. **That is why the declarations are at `app/types/pdfjs/` and
must not be under `dist/`**: placed there they would put 824 KB no runtime reads
into the shipped binary. `app/tsconfig.json` and `app/typecheck.mjs` sit beside
them and are read by nothing the app builds — what they are for is
`rules/desktop-panes.md`. **`app/package.json` and `app/bun.lock` sit there too, and
they are this crate's first node manifest**: they pin `playwright` 1.62.1 for
`app/harness/`, and no build reads either. `withGlobalTauri`'s "one Cargo build, no
node toolchain" is a claim about the *build* and is unchanged by them — a contributor
who never drives the page installs nothing, and the browser binaries are a
`playwright install` run once by one who does. They sit under `app/` rather than at
the workspace root because the workspace is Cargo's and nothing outside this crate
reads them; `node_modules/` and the scratch directory `app/.harness/` are gitignored,
the second for the same reason `app/types/` is not under `dist/`.
`capabilities/default.json` grants
`core:default` to the window labelled `main`, plus **one entry per dialog** —
`dialog:allow-open` and `dialog:allow-save`.

**What that withholds is worth stating beside what it carries.** `core:window:default`
is the window's *getters* — `allow-scale-factor`, `allow-inner-size`, `allow-title`,
the `is-*` family — and **no setter**, so nothing in the page can resize, move or
retitle the window. The two setters this app calls — `set_title` and
`set_appearance`'s `set_theme` — are `main.rs`'s own, from Rust, where capabilities
do not apply, and neither added one. **A setter called from the page is a function
that exists and rejects at the IPC**: `setSize` is on `getCurrentWindow()` whatever the
manifest says, and answers `window.set_size not allowed. Permissions associated with
this command: core:window:allow-set-size`. So which of them are reachable is a question
to settle by calling one, never by `typeof` — which cost `tests/gates/mpdf-003-phase11.js`
two runs.

**One build of this crate carries a WebDriver server, and it is asked for by
name.** `Cargo.toml` declares `tauri-plugin-wdio-webdriver` 1.3.0 **optional**
behind `[features]`' `driven = ["dep:tauri-plugin-wdio-webdriver"]`, and
`main.rs:main` breaks its builder chain at the top — an attribute cannot be
applied to a method call — so a `#[cfg(feature = "driven")]` rebinding can add
the plugin and nothing else moves. `cargo tree -p letur -e normal` names the
crate **zero** times and once under `--features driven`, and
`app/driver/drive.mjs` is the only thing that asks for it. **Not
`[target.'cfg(debug_assertions)'.dependencies]`**, which WebdriverIO's own setup
page prescribes and which selects nothing — cargo says so in as many words — so
under the documented form the crate is compiled and linked into a *release* build
and only `init()` is gated.

**No capability goes with it, and the entry that looks required is a trap.** The
plugin declares `COMMANDS: &[]` and `permissions = []` and exposes no IPC at all,
every automation going over its own HTTP server on 4445; `wdio-webdriver:default`
in `capabilities/default.json` would therefore grant nothing when the plugin is
present and **fail the build when it is absent**, `tauri-build` globbing
`capabilities/**/*`. A `--features driven` binary with no capability entry
anywhere was built and driven. **What the server does offer is the setter the
page is refused**: `setWindowRect` is in the session's capabilities, and
`window/rect` is **physical** pixels where `tauri.conf.json`'s `width` is
logical, so a driver reaches a viewport width by reading `innerWidth` back rather
than by arithmetic.

**A third configuration fact costs a frame rather than a build.** The `main`
window is declared `"visible": false` and `setup` shows it, after `set_theme` and
with `set_focus` beside it. **The runtime does not merely build the configured
window before that hook — it puts it on screen**, so an appearance applied there
arrives a frame late and a stored `dark` on a light system flashes; measured in
the window on 2026-08-29, which is what `specs/desktop_app_spec.md` Phase 13's
own note records. So `show` is now what makes the window appear at all, and
nothing between the config and that line may return early.

Two configuration facts cost a build each. **`icons/icon.png` is required** —
without it `generate_context!` panics with "failed to open icon" — and **it must
be RGBA**, or `tauri-codegen` panics "is not RGBA" instead. The file in the tree
is a generated placeholder, and **no phase has designed artwork to replace it**;
the bundler synthesises the `.icns` from it, so the Dock upscales one 512×512
image. **`app.security.csp` is unset and defaults to `None`, so no policy is
served at all** — which is what lets the page import ES modules and start a
worker from `app/dist/pdfjs/` over the `tauri://localhost` custom scheme, with no
capability added for either. `app/gen/` is generated and is not committed.

## The window

`app/src/main.rs` holds only what needs a window. `main` registers the dialog
plugin, builds the menu, manages one `Mutex<Session>` and one
`app/src/main.rs:Pending`, registers nineteen commands, and — since the bundle —
**builds the app rather than running it**, so that it can hand `App::run` a
callback and see the run events a `.run(generate_context!())` never surfaces.

The menu is built by hand, because macOS draws none of its own: an app submenu,
a `File` submenu carrying `Open…` at `CmdOrCtrl+O`, `Save` at `CmdOrCtrl+S`,
`Save as…` at `Shift+CmdOrCtrl+S` and `Save a Copy…` at no accelerator at all,
an `Edit` submenu, a `View` submenu carrying `Files` at `CmdOrCtrl+B` and `Lines`
at `CmdOrCtrl+L`, and a `Window`
submenu. **`View` sits between `Edit` and `Window`**, which is where macOS puts
it; the builders have no positional insert, so the slice's order is the
placement. **No item acts on its own.** Each emits an event of its own id to the
window, and the page invokes, so a menu item and the button beside it run one
code path and not two — which is why `Save` emits too, though it opens no dialog
and so costs no capability. **`Open…` and `Save` have that button, in the header;
`Save as…` has that button — the header's one floppy, `mpdf-003` Phase 17 — while
`Save` has neither a button nor a rival for `⌘S`, and **`Save a Copy…` is the menu's
alone and now its only accelerator-less item**, its header button withdrawn on the argument
that the other two act on the document being edited where the export writes a
derived artifact. **`Files` and `Lines` have theirs in the footer**, the bar's two
drawn marks. `core:default` already carries
`core:event:allow-listen`, so neither those events nor the `rendered` signal
below needs an entry either.

**Six ids, and the enumeration is the load-bearing half.** `on_menu_event` filters
on `open`, `save`, `save-as`, `export`, `view-files` and `view-lines` by name
before it emits, so an item whose id is missing from that list builds, draws with
its accelerator, and does nothing at all. **The two view items are plain
`MenuItem`s and carry no checkmark**: the state one would place is the page's —
`app/dist/index.html` holds both settings and Rust reads neither — and a
`CheckMenuItem` would be a second copy of a page-held boolean across the IPC
boundary. The bar's two marks stay the only things in the window that wear either
state. **`Files` stays enabled with no document open** and the page declines it,
on `Save`'s precedent: the refusal happens where the state is, and disabling it
here would put a menu write on the status path, which runs on every compile and
every watch event, to grey an item whose press already does nothing.

`app/src/main.rs:open_document` **titles the window twice, and both are
needed**: once from the path the user picked before the compile, so the window
names something whether or not it compiled, and once from the session after,
because the open lands on the file the *project* compiles rather than the one it
was handed. A double-click on a section reads `text.md` for one compile and
`showcase.md` after it, both true in their instant. `set_main` and `set_edited`
are the same shape, and the title is now the *edited* file's. **It returns no
bytes.** The compile and the fetch are two calls, because
the watch loop compiles with nobody asking. The command is `async`, so the
compile runs off the thread that draws the window.

`app/src/main.rs:current_pdf` is the fetch, and `rendered` — a signal carrying
**no payload**, emitted after every compile — is what asks for it. The bytes
cross as a `tauri::ipc::Response`, reaching the page as an `ArrayBuffer`; a
returned `Vec<u8>`, or an event carrying them, would serialize as a JSON array of
numbers, one per byte. Its `Err` is a state and not a fault: a stale pane keeps
its bytes and gets the message instead.

`app/src/main.rs:status` answers the same signal, **a second command rather than
a wider return**, because the bytes cross raw and a status does not. It asks
`Session::status`, not `preview()`, the first knowing a field the second cannot, and
**that one line is outside every test here** by the division this section's last
paragraph records. `app/src/main.rs:set_appearance` is the sixteenth and is two
halves: `Session::set_appearance` writes, moves and announces, then `set_theme`
follows with the native title bar — the half no browser sees, and the reason this is a
command at all. **The loose reading fails silently**: writing the file and never
announcing leaves the bar's own mark stale until the next compile.
`app/src/main.rs:document_text` answers it too, and only when the status says the
buffer was replaced from disk. `app/src/main.rs:export_path` and
`app/src/main.rs:export` are the two halves of Save a Copy.

`app/src/main.rs:edit` takes what the author typed, `app/src/main.rs:save`
writes it to the edited file's own path, and `app/src/main.rs:discard` throws it
away and takes that file again — **the only command in this app that discards
anything**, and the way out both refusals name. **Keystrokes
therefore cross the IPC boundary and the debounce is Rust's**, which is what puts
the interval on the testable side of the window: a debounce in the page would be
logic reachable only by typing at one.

Seventeen of the nineteen commands are wrappers over a plain function.
`app/src/main.rs:pending_open` is the odd one and is not: it reads no session
and **takes** a slot the run event filled, which the section on the bundle below
explains. `app/src/main.rs:fetch_images` is the newest of the nineteen and the
split is load-bearing rather than tidy: this file has no test module, the crate
is bin-only and `tauri::State` has a private field and no public constructor, so
a rule written into a command is a rule nothing in this repository can reach.

## The file I/O

`app/src/document.rs:render_with` **takes the markdown as a parameter and not a
path**, and hands that one string to both the asset list and the compile; the read
left it for `app/src/document.rs:read_document` one call out, so the string the
pane holds is what compiles, and `md2pdf_core::md_to_pdf` already took a `&str`.
`app/src/document.rs:render_project` is the caller that puts the project back
together — `main`'s text and directory, with the buffer standing in for `edited`
through the read closure below. Both return a `Render`, which carries the asset
paths **even when the compile failed**: emission reads the text and not the disk,
so a document whose figures are all missing still names them, which is what keeps
the watch filter alive while nothing compiles.

**`Render::assets` has three answers, and the section paths go in first and
unconditionally**: `Some(sections ++ bibliography ++ images)` when the two walks
answer, `Some(sections)` when they do not and the master names any, `None`
otherwise — and on `None` the caller keeps the list it had, which is what stops a
transient out-of-dialect edit from dropping the images the app knows about.
**The middle branch is why the sections are unconditional**: `section_paths`
cannot fail where both shopping lists now fail with `Error::MissingSection` for a
section that does not exist yet, and `Preview::compile` replaces the list only
when it is `Some` — so without it the list would stay empty, `classify` would
drop the section's creation event, and the window would never recover. It
*replaces* the list with a shorter one, so such a document stops watching its
figures until the section returns: a deliberate trade, since recovering the
section beats watching figures through a window in which nothing compiles anyway.

**`Render::sections` is that same list kept on its own, and its type is the
claim.** A plain `Vec` where `assets` is an `Option`: `assets` is `None` exactly
when the caller must keep the list it has, which is a sentence about a watch
filter and not a thing the panel that reads this can draw. `section_paths`
cannot fail, so an empty list is the answer rather than a failure to answer —
and it is taken off the *text*, never off the read, so a master whose sections
are missing from the disk still names them.

**A file the document names makes two journeys through this app, and they are
independent.** The bytes travel `read_sections_with` → `read_assets_with` →
`md_to_pdf_with_anchors` and reach nothing else; the paths are built separately
in `app/src/document.rs:render_with`, which calls all three exports for itself,
and travel `Render::assets` → the watch filter. **The bytes are what make the
document compile; the paths are what make a change to one of them redraw.** The
image and bibliography exports come off one walk, so they answer or fail
together, and the paths arrive even when a read failed — which is what an asset
named before it exists depends on. **An image named by an http or https URL makes
neither journey**: `ImageRef::is_url` drops it from both, because a URL is a name
and not a file. Since `mpdf-003` Phase 25 **its bytes travel from memory once
fetched** — supplied to the compile under the URL itself, from what
`app/src/remote.rs` holds for the process — and its path still reaches no watch,
there being no file to watch. A URL nobody has allowed is refused by `core`'s
`Error::UnfetchedImage`, as it always was; `rules/desktop-compile.md` has the
consent, the fetch and the line.

The compile itself is `md2pdf_core::md_to_pdf_with_anchors`, and `Render` carries
its `anchors` beside the bytes. **They go the other way from `assets`**: a failed
compile has none, because they describe the *page* where the asset list describes
the text. `app/src/document.rs:Anchor` is `md2pdf_core::Anchor` again — the
duplication `read_assets_with` already makes — and is a line and a page where
`core`'s carries a `md2pdf_core::Location`, because **`document::Pane` keeps only
the anchors written in the file the pane holds** and drops the rest.
`rules/desktop-panes.md` has its three arms and why the third is not an absence.

**The reading is two passes and it mirrors the CLI's.**
`app/src/document.rs:read_sections_with` runs first, as
`cli/src/main.rs:read_sections` does and for the reason that function records: the
markers are in the master's own text, so the sections can be read with no join,
where neither shopping list can answer about a document that has not been
assembled. `app/src/document.rs:read_assets_with` then mirrors
`cli/src/main.rs:read_assets` — it takes the sections, seeds `seen` with their
paths, carries them out on the same array, resolves each remaining path against
`app/src/document.rs:directory`, reads each once, and keeps the path the markdown
wrote. **The bibliography is read first of the two remaining channels**, the line
it names being the frontmatter's and therefore earlier than every image's. A
section's own images are found beside it with nothing added here: `core` wrote the
folder into the destination before the list arrived, so an image drawn in
`sections/method.md` arrives as `sections/figure.png` and still joins the master's
directory. The duplication with the CLI is deliberate — the two wrappers report
their errors differently, and **the app owes the terminal three hand-built
sentences**, one per channel, a file that will not read being no
`md2pdf_core::Error` at all.

**One closure serves both passes**, borrowed by the first and handed to the second
in `render_with`. The read is a parameter so a caller counting its own reads can
check that a path named twice is read once, and a second closure would leave half
of them outside that counter. **It is also the seam the pane's buffer rides**:
`render_project` builds one that answers `edited` from the buffer, `main`'s own
text included, so the buffer compiles exactly when the pane holds main. Both
classes of failure reach the page in the terminal's words — an `Error` through its
`Display`, a file that will not read through the sentence this builds — and a
`main` that is not UTF-8 fails there too, wrapped so the two spell alike.

**`app/src/document.rs:asset_bytes` is a fourth reader of a path the author did
not name in a dialog**, beside the walk, the compile's own closure and
`preview::Session::set_edited`, and it obeys the rule all four do —
`app/src/document.rs:confined`, which resolves the path and requires its target
under the resolved root, so a `..`, an absolute path and a symlink leaving the
root are refused by one comparison and in one sentence. It reads one of the project's figures for
the window to draw and nothing about the document changes for it — no compile, no
buffer, no `Status` field. The bytes cross as a `tauri::ipc::Response` for
`current_pdf`'s reason and reach the page as an `ArrayBuffer`; the page makes a
blob of them, which wants neither Tauri's asset protocol, nor a scope in
`app/tauri.conf.json`, nor a capability. **`app/src/document.rs:create_file` is
the first write to such a path**, `write_override`'s being into Application
Support: it makes an empty file and stops, `rules/desktop-project.md` has the
three rules it obeys, and the row arrives by the watch rather than by a return.

**`app/src/document.rs:save_file` is the second writer of a path the author *did*
name in a dialog**, `export` being the first, and **it is the one write in this file
that confines nothing**: `mpdf-003` Phase 18 dropped its `landing` call, so a
`Save as…` goes wherever it is pointed. Its contract is therefore the path **as
given** — a relative one would resolve against the process working directory and not
the project — and the confinement question moved to its caller, where it is asked
*after* the write: `preview::Preview::save_as` requires both
`app/src/document.rs:confined` and `app/src/document.rs:spell` before it moves the
pane, `confined` opening on `is_file` and so having nothing to answer before the file
exists. `kind_of` stays, so a `.txt` is refused wherever it is aimed.

**`app/src/document.rs:trash_file` is the first *destructive* one, and it moves
to the Trash rather than unlinking.** There is no undo anywhere in this app —
not for an edit, not for a save, not for an export — and the Trash is the
platform's own undo for exactly this operation. **That is also why nothing asks
twice**: a confirmation stands in for an undo where there is none, and Finder
does not confirm a move to the Trash for the same reason. `rules/desktop-project.md`
has the rule it obeys, which is neither of the other two.

**The OS call is a parameter and `app/src/document.rs:move_to_trash` is what
`app/src/main.rs` hands in** — the one function in `document.rs` no test in this
repository calls. It is injected where `create_file`'s write deliberately is
not, and the difference is where the effect lands: a `std::fs::write` goes into
a `scratch_dir` the suite owns, where this call's whole effect is **outside the
repository**, in a `~/.Trash` nothing cleans.

`app/src/document.rs:default_output` is where an export lands unless the user says
otherwise: **`main`'s** path with a `.pdf` extension, duplicating
`cli/src/main.rs:default_output` because sharing it would make one crate's binary
reachable from the other. `document::spell` is the textual root-relative spelling
and `document::relative` the canonicalizing one; `document::under` is
`document::beside` run backwards, which is how a root-relative `edited` reaches
the spelling `md2pdf_core::Location` carries.

## The project

`rules/desktop-project.md` has it: the root the opened file climbs to, the
masters under it, the listing the panel draws, and the two facts remembered.

## The compile

`rules/desktop-compile.md` has it: the watch loop and its two debounces, the
three steps one compile is taken apart into and the two guards on its answer, the
state that loop writes and the four states it reports, the rule an external
change runs, and the images fetched by URL.

## The session

`app/src/preview.rs:Session` is that state plus the two loops, the fetch worker
it starts, the paths of the **three**
files this app writes outside the author's own folders — all three out of one
`app_data_dir()`, and `rules/desktop-project.md` has why they are three — and the
appearance one of them holds. Each path is **a parameter and not a call to the
platform**, so a test hands in a scratch file and `main.rs` resolves the real ones
where the `AppHandle` is. **The appearance is here and not on `Preview`**, which
`open_at` rebuilds whole and would reset it on every open; so `Status` is composed in
two places, `Preview::status` filling `System` where `Session::status` corrects it, and
`preview.rs:the_session_carries_the_appearance_and_a_bare_preview_does_not` holds that
seam — a mis-wired composition being otherwise silent.
`Session::open` finds the project the opened file sits in and **reads the store
first**, or it is a thing written and never used; an override naming a file the
disk no longer holds falls through to discovery rather than opening nothing. It
then hands root and main to `open_at`, which `Session::set_main` also takes, so
the store and the window cannot disagree. It clears the previous document's page
and text, reads and compiles once, and only then calls `Session::arm` — old loops
dropped before new started, so no two of them ever hold a document. Both callbacks
check `edited` before **planning** and again before absorbing, dropping a `Watch` or a
typing channel not joining its thread.

**`Session::set_edited` borrows `arm` and not the rest of the open**, and
`rules/desktop-project.md` has why. `arm` exists at all because both loop guards
key on a path captured when they started, so a command that moved the pane and
stopped there would compile nothing and drop every event.

`Session::on_change` is where the four kinds of event reach different code: the
**edited** file runs the rule, the document and an asset are a bare recompile
each, and **a `Tree` event walks the disk and stops** — no compile, so `revision`
stands still, which is what "does not compile" means as an assertion. It is
announced all the same, because the panel is drawn off the status the announcement
fetches. A window that took the disk copy compiled inside the rule and read the new
assets on the way, so one window never compiles twice, and **nothing is announced
when nothing happened**.

`Session::set_main` and `set_edited` **confine rather than merely checking
existence**: each asks `document::relative` for the path's root-relative spelling
back, which `root.join("../../secrets.md")` cannot answer where `is_file` would
have said yes. Both then **refuse while the buffer diverges from the last-saved
text**, in `SWITCHING`'s own sentence rather than `DIVERGED`'s, which opens *"this
file changed on disk"* and would be a lie here. **The refusal rides `divergence`
and not an `Err`**, so one refusal does not arrive in the window two ways, and it
announces, because no compile ran and nothing else would fetch the status carrying
it. **One field, one occasion at a time**: a switch refused over a standing
divergence overwrites that sentence and is overwritten by the next, which costs
nothing — the two name the same two exits, and `Preview::save` and
`Preview::take` clear both. `Session::discard` is that second exit, and is
`Preview::load` behind a command.

## The export

`app/src/preview.rs:Preview::export` writes the page's own bytes where the user
asked. **Nothing in it compiles**: the file is what the pane is already showing,
which is what keeps the two from disagreeing.

`Preview::exportable` is the one refusal rule, and **it words two sentences
rather than one**, because an *empty* pane holding no bytes and a *stale* or
*failed* one holding bytes known to be old are two problems. `export_path` runs
it before returning the default path, so a pane that cannot be exported never
opens a dialog whose answer it would throw away. **That path is `main`'s and not
the pane's**: the bytes on offer are the master's, so a section in the pane must
not lend the file its name.

**The file it writes is the file `md2pdf` writes**, byte for byte, for the same
document — **while the pane and the file say the same thing**. The PDF is a pure
function of the text and the asset bytes, and the text the app compiles is the
pane's; a buffer with unsaved edits is a different document from the one on disk,
and the two front ends then agree about nothing except their own inputs. No
single test holds even the qualified claim: `CARGO_BIN_EXE_md2pdf` reaches only
integration tests of the package defining that binary, and `letur` declares
a `[[bin]]` and no `[lib]`. It is gated in two halves, in `app/src/preview.rs`
and `cli/tests/cli_test.rs`, meeting at an in-test `md_to_pdf` call.

The limit the flag accepts: **"stale" answers "did the last compile fail", not
"does the page match the file on disk"**. It never answered the second question,
and the window in which the two can differ is now unbounded rather than the
debounce plus the compile: an author who types and does not save leaves the file
behind the page for as long as they like, and the pane reads *current* throughout,
because it is current — for the text in the pane.

Nothing in `preview.rs`, `document.rs` or `watch.rs` needs a window: a GUI whose
logic is reachable only by clicking has no exit gate but a screenshot. Two claims
are read by a person: whether the right pixels reached the glass, and whether the
bundle below runs at all away from `cargo`.

## The bundle

`cargo tauri build` is the one command, and **`bundle.active` is what it turns
on** — while that key is false the two obvious commands disagree, `cargo tauri
build` skipping the bundle and the standalone `cargo tauri bundle` making one
anyway. `bundle.targets` is `app` and `dmg`, leaving
`target/release/bundle/macos/Letur.app` and a `.dmg` beside it named for the
version and the architecture. **`productName` renames the `.app` and not what is
inside it**: the bundle is `Letur.app` and `CFBundleExecutable` is
`letur`. `tauri-cli` pins at 2.10.1, as everything here pins.

`Contents/Resources/` holds `Letur.icns` and nothing else, and **no font ships
there**: `core/src/lib.rs` embeds all five faces with `include_bytes!` and the
Typst world exposes those alone, so one added there would be dead weight.

**The bundle is unsigned**, measured rather than assumed: `codesign -dv` reports
`flags=0x20002(adhoc,linker-signed)` with `Sealed Resources=none`, and `spctl -a
-t exec` rejects it. How it travels therefore decides whether it launches
elsewhere — a copy over USB or `scp` sets no `com.apple.quarantine` attribute and
runs, a download or an AirDrop sets one and Gatekeeper refuses until a person
overrides by hand. `bundle.macOS.signingIdentity` and `hardenedRuntime` are where
the fix goes, and both need an Apple Developer credential.

**A bundle gets its own privacy identity, and the watch loop depends on one.**
Under `cargo tauri dev` the process inherits the terminal's grants; as
`dev.letur.desktop` it does not, and `watch.rs:start` watches a whole directory
recursively. A document under `~/Documents`, `~/Desktop` or `~/Downloads` can
compile once through the open panel and then stop redrawing — the silent-failure
class the canonicalization note above records, by another route.

## The document association

`bundle.fileAssociations` emits one `CFBundleDocumentTypes` entry: `ext` becomes
`CFBundleTypeExtensions`, `name` `CFBundleTypeName`, `role` `CFBundleTypeRole`,
`rank` `LSHandlerRank`, and `contentTypes` `LSItemContentTypes` — the last only
when asked for, carrying `net.daringfireball.markdown` because modern
LaunchServices prefers a UTI. **Spell it `contentTypes`, not `content-types`**:
`tauri-utils` declares that alias, but the CLI validates against the generated
JSON Schema before serde runs, an alias does not appear in a schema, and
`deny_unknown_fields` becomes `additionalProperties: false` there. The rank is
`Default`, so **a machine with an editor already registered for `.md` keeps that
editor** — the ranking working, not a broken association.

**The association only launches the app; it hands the process nothing**, and
nothing here reads `std::env::args`. The path arrives as
`tauri::RunEvent::Opened`, which is why `main` builds and then runs. Its URLs are
`file://`, and the path comes from `Url::to_file_path` and **not** `Url::path`,
which leaves a space percent-encoded — `my doc.md` would arrive as `my%20doc.md`
and open as nothing; `tauri` re-exports `Url`, so this costs no dependency.
`urls` is a `Vec`, because Finder delivers a multiple selection as one event, and
the app takes the first. A cold launch survives by ordering rather than by
queueing: tao's `AppState::open_urls` **drops** an event when no callback is set,
and tao installs the callback before `NSApp.run()`.

**The open goes through the page rather than around it**, and `clear()` is why.
`Session::open` rebuilds from `Preview::default()`, so `revision` and `reloaded`
restart at 0 per document, while the page resets `drawnRevision` and
`takenReload` only inside `clear()` — which the dialog path calls and a path
straight into Rust never would. A second document opened that way would return
Rust to `revision 1, reloaded 1`, which the page already holds, and both panes
would keep the old document under a new title.

So the run event **stores and emits**, in the menu items' own idiom: the path
goes into `Pending`, a managed slot, and a payload-less `opened` signal goes to
the window. The page takes that slot through `pending_open` at startup and again
on every signal, and **the take clears it**, so a cold open is collected by the
first, a warm one by the second, and whichever runs second finds nothing. The
page then calls `clear()` and invokes `open_document` as the dialog does.
`core:event:allow-listen` already covers the signal, and `open_document` keeps
its signature and its `async`.

The limit that accepts: `listen` completes over IPC, so a document landing
between the startup take and the listener's registration would sit in the slot
until the next signal. `app/dist/index.html` registers its eight `listen` calls
before it calls `refresh()` and `takePendingOpen()`, which makes that window
practically unreachable, and the cost if it is reached is a document that opens
late rather than one that opens wrong.
