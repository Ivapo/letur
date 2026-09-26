---
title: desktop-compile
sources:
  - app/src/preview.rs
  - app/src/watch.rs
  - app/src/document.rs
  - app/src/remote.rs
  - project/src/preview.rs
  - project/src/document.rs
  - project/src/remote.rs
covers: >
  the desktop app's compile: the watch loop and its two debounces, the compile's
  three steps and the two guards on its answer, the state the loop writes and the
  four states it reports, the rule an external change runs, and the images
  fetched by URL — the sites allowed and where they are kept, the one place a
  fetch is claimed, the settle, the worker, and the line that says what is on its
  way
max_lines: 300
# **Split out of `rules/desktop.md` on 2026-09-22**, at `mpdf-003` Phase 25.
# That file was at 730/730 and its own frontmatter named this seam — the watch
# loop and the compile — as where the next phase needing room would cut. This is
# that cut, and the fetch is what needed the room.
generated: 2026-09-22
---

# Desktop compile

**What is true right now**, for the loop that keeps the page current: what is
watched, what a change does, what one compile is, what the state holds, and what
an image named by a URL costs. The window, the file I/O and the session are
`rules/desktop.md`; the two panes that draw this are `rules/desktop-panes.md`.

## The watch loop

`app/src/watch.rs:root` is the document's own directory; **the watch set is the
project root above it**, watched recursively, and the two differ exactly when the
climb found a master above the opened file. `core/src/emit.rs:written_shape` refuses a URI scheme, a
leading `/` and a backslash, `core/src/emit.rs:landed_path` refuses a path that
leaves the document's folder, and `core/src/frontmatter.rs` puts the bibliography
under that same rule, so every path a document can legally name resolves under
there — one watch covers the document, every asset it names, every asset it will
name, and every directory not yet created. **The premise is where a path lands
and not the segments it is spelled with**: `../figures/plot.svg` written inside
`sections/method.md` is legal and lands on `figures/plot.svg`, still under the
root. What `classify` compares against is the resolved path, which
`core/src/sections.rs:Sources::resolve` has already normalised — a stored
`sections/../figures/plot.svg` would never equal the event path
`root(document).join(asset)` builds. It is computable from the document's path
alone, so a document the dialect refuses is watched too. The limit: an asset that
is a symlink out of the directory is not watched, because a watch follows the
tree and not the targets.

`app/src/watch.rs:classify` is the filter, and the one list `section_paths`,
`image_paths` and `bibliography_path` fill is what it filters against — the list
is not the set, and it follows the buffer, because the buffer is the document
now. **A section changes nothing about the watch set**: `root` is already
recursive and `sections/` sits under it, so a section is one more string in that
one list, and it arrives as `Change::Asset` and never `Change::Document` —
`Preview::reload`'s rule is about the buffer the pane holds, which is never a
section. **It sorts rather
than admits**: an event is `Change::Document`, `Change::Asset`, `Change::Tree` or
nothing. **`Tree` is the events this dropped** — any path under the root the
document does not name — and the panel needs exactly them. **The root is a
parameter, no longer re-derived from the document**, since the two can differ:
the document and its assets still resolve against the document's *own* directory,
where a path the markdown writes resolves, and only `Tree` is measured against
the root. **A bibliography is one more string in that one list**
rather than an arm of its own: the split is the open document against everything
the disk supplies, and a bibliography sits on the second side for the reason a
figure does. It also makes a *dropped* `bibliography:` key free, since
`Preview::compile` replaces the whole list on every compile. The document **stays** in the filter though its events no
longer mean "compile" — a path dropped from it would never reach the rule that
decides what its events do mean. **Both sides are
canonicalized**, and the loop depends on it: `notify` canonicalizes a path as it
registers it, because FSEvents reports the resolved path, so an event names
`/private/var/…` where the Open dialog handed the app `/var/…`. On macOS `/tmp`
and `/var` are both symlinks into `/private`, so comparing the paths as they
arrive would leave the watcher running, every event dropped, and the page never
redrawn. The document's *directory* is resolved rather than each file, because a
figure named before it exists has no real path to resolve.

**There are two intervals, and each is measured against a different thing.**
`app/src/watch.rs:DEBOUNCE` is 100 ms, margin over FSEvents' own batching —
twenty saves under each of three write strategies, tabulated in its own doc
comment. `app/src/watch.rs:TYPING_DEBOUNCE` is 300 ms, and it is not protecting
the compile: twenty compiles of each sample through the pane's path put the warm
median at 1.5 ms and 0.6 ms, against 24.6 ms and 12.5 ms for the first compile of
a process, which the app pays at the open. **Both samples are single files**, and
a project of 800 sections is 23.9 ms warm against `md2pdf-core` 0.1.3 — an order
of magnitude under the interval where an article is two, the crossover past it at
some 2.5 MB of markdown, and `mpdf-003`'s OQ-14 holds the tables. It is set to
the pause between phrases rather than the gap between keystrokes, because a
redraw moves the reader — narrowed but not removed by the anchor, which returns
them to the top of the section they are in. `app/src/watch.rs:Debounce` takes the time as a parameter, so both tests
need no clock and cannot flake.

`app/src/watch.rs:settle` is the thread both intervals run on: it folds a stream
into one call per quiet interval, accumulating what arrived. `app/src/watch.rs:start`
registers the watch and settles `notify` events into a `Changed` — which of the
document and the assets moved, since one window can hold both.
`app/src/watch.rs:debounced` settles bare nudges, and is what the keyboard uses.

**Dropping the `Watch` stops the loop**: it unregisters the directory and drops
the sender its handler holds, which disconnects the channel, which ends the
thread. Dropping the typing channel ends its thread the same way. That is the
whole mechanism by which opening a second document moves both.

**The compile runs with the state lock released, and two guards decide whether its
answer is still wanted.** `project/src/preview.rs:Preview::compile` is three steps:
`Preview::plan` stamps a serial onto a `Compile` owning the three inputs,
`app/src/preview.rs:Compile::run` borrows no `Preview` — it carries the plan and a clone
of the `Disk`, renders, and times it with an `Instant` — and so *cannot* hold the lock,
and `Preview::absorb` takes it back with the duration. **The crate reads no clock**: the
compile it runs whole times itself through the `Timer` the session handed it.
`Session::recompile_with` and `Session::on_change_with`'s bare-recompile branch — the two
that fire while a hand is on the keys — drop the lock across the middle step, so a
keystroke arriving mid-compile waits on nothing. `Preview::load` and `Session::save_as`
keep `compile` whole, each one user action already behind `main.rs`'s own
`Mutex<Session>`; **`Preview::reload` is the exception**, on the watch thread behind no
such lock, kept whole because prising it apart inverts `on_change`'s `!taken`. The guards
are `Preview::current` over the three inputs — derived, not counted, they being written in
six places — and `Preview::started`/`landed`, on which **the newest-*started* render
wins**: every change a compile reads is followed by an event scheduling a render that
starts after it, and a dropped answer always has a fresher one coming, each of those six
writers being followed by a compile. **`Preview::open` carries `started` and levels
`landed` to it** — *an Open discards every answer in flight* — where the zeroes `revision`
and `reloaded` take would let an orphan land and freeze the window. Two renders can now
overlap, doubling peak cost for that span; OQ-14 holds the measurements.

## The state

`project/src/preview.rs:Preview` is what the loop writes and the pane shows. **Three
values where it held one**: `files`, the project's `Files` — on the desktop a
`Disk` whose root the panel lists and the watch covers; `main`, which file under
it compiles; `edited`, what the pane holds and `⌘S` writes, both root-relative,
equal at every open and free to differ from the first row click on. **The root moves
only on an explicit Open** — a click that re-rooted would strand the author below
their own project with no way back up. Beside them: **the text the pane holds and
the text as it stood at the last open or save**, the last good PDF bytes, how
long they took, the asset list the filter reads, the disk walk the panel is drawn
from, four counters, a stale flag, the error and the divergence. The two strings are what
`project/src/preview.rs:external_change` compares, and holding them here rather than
in the page is what keeps that rule testable at all.

`Preview::compile` compiles **`main`, with the buffer standing in for `edited`**,
through the one closure `document::render_project` builds: it answers the edited
path from the buffer and every other from the disk, main's own text included, so
the buffer is what compiles exactly when the pane holds main and one rule covers
both. A `main` this app cannot read leaves `read_document`'s own sentence and the
*failed* state. `Preview::load` is the only thing that reads a file into the
buffer, and `Preview::edit` takes what the author typed without compiling,
because one keystroke is not a document. `Preview::save` writes the buffer to
`edited`'s path and moves the last-saved text with it — which is what makes that
save's own event mean nothing a moment later. **`Session::save` and
`Session::save_as` answer a receipt rather than `()`**, `mpdf-003` Phase 19: the bare
word `saved` for the first, and `saved as <name> in <folder>` for the second with the
folder absolute and spelled as landed. **Composed in `Session` and not in the
command**, which is forced — `main.rs` has no test module — and **riding the return
and not `Status`**, a receipt being an event rather than state, so no field was added
and the page's typedef block did not move.

`Preview::compile` replaces the bytes on success and
clears both marks; **on failure it keeps the bytes**, records the message and
sets the flag. **The duration travels with the bytes**, replaced and kept
exactly as they are, so the time the window shows describes the page on screen
rather than the last attempt at one.

`project/src/preview.rs:State` is the four the window reports: *empty*, the state the
app launches into; *current*, a compile that succeeded; *stale*, one that failed
over a page still drawn; and *failed*, one that failed with no page to keep.
**What separates *stale* from *failed* is whether there are bytes**, not any
branch in `current_pdf` — `compile` sets the flag on every failure, so both take
that command's first branch. **Empty is exactly "no document has been opened"**:
`compile` returns early with no document and `Session::open` sets the document
and compiles inside one lock scope, so nothing observable sits between
`Preview::default()` and the first outcome. The serialized name is lowercase, and
the page uses it as a word and as a class.

`project/src/preview.rs:Status` is that state, the compile time worded as `"28 ms"`,
the error, whether a page is drawn, the divergence, two counters and the anchors —
one value the page places rather than composes. **Thirteen fields**, the
appearance and the web line beside the eleven the last compile fills, and
`preview.rs`'s own test holds the page's typedef to exactly them.
**`revision` counts compiles that produced bytes** and `reloaded` counts the times the buffer was replaced from
disk; both exist so the page can tell a signal apart from work it has already
taken. **The anchors ride the status because the status is already fetched on the
path that draws**, so following the caret needs no command of its own. Like the
compile time, they are replaced on a success and kept on a failure, so they always
describe the page on screen rather than the last attempt at one.

**`entries`, `main` and `edited` ride with them and for their reason**, the
status being already fetched on the path that draws, so the panel costs no command
of its own. `entries` is put together here and **reads nothing off the disk**:
`Preview::tree` is the walk, refreshed at an open and a `Tree` event and never in
`status()`, which the page calls on every render, and the marked-missing rows come
off `Preview::sections`, which every compile assigns from the master's text. Both
paths are root-relative so the page can match them to a row, and **`edited` rides
because the page cannot derive it from `main`** — they are equal at every open and
differ from the first click. It is spelled with `document::spell` and not
`document::relative`, a `canonicalize` here being two syscalls in front of every
render. **It is root-relative and can be nothing else**, which is a property of
where the pane may go rather than of this function: `Preview::edited_relative` held
an absolute fallback for the one release `Save as…` could put the pane outside the
root, and `mpdf-003` Phase 19 removed the state instead — every path that moves the
pane keeps it under the root, so `spell` cannot decline and a fallback nothing could
test is not kept as insurance.

## The rule an external change runs

`project/src/preview.rs:external_change` is three strings and two comparisons, and it
needs no dirty flag. **It is asked about the *edited* file and nothing else**: the
master moving is a bare recompile, because with the pane elsewhere the master is
one more file the compile reads off the disk. The file equal to the buffer is
`Unchanged` — the app's own
save arriving back, and **nothing happens**. The file differing under a clean
buffer is `Taken` and recompiled, which is the loop the app has shipped since
Phase 2 and the case an unconditional refusal would have broken. The file
differing under a dirty one is `Diverged`: the work is kept and the divergence is
named. **The app does not merge**, and it makes neither choice for the author —
saving overwrites the disk, reopening takes it.

`Preview::reload` reads the file and carries that answer out; a document that
will not read at that instant counts as `Unchanged`. **A divergence is not
staleness**: nothing failed to compile, and the page belongs to the text in the
pane.

Two limits it accepts. An author typing between a save and that save's event
lands in `Diverged`, so the app can name a divergence that was really its own
write — it loses nothing, and the next save clears it. And an external writer
that writes exactly the author's unsaved text takes `Unchanged`, leaving the
last-saved text unrefreshed. Both err toward keeping work. **Nothing suppresses a
self-write**: the first outcome *is* the self-write case, decided by comparing
content rather than by winning a race against an event that arrives 12 ms after
the write.

## The images fetched by URL

`md2pdf-core` 0.3 made a URL a name the caller fills: `core` fetches nothing, and
a caller that fetches supplies the bytes under the URL itself. The CLI does it
under `--fetch`; **this app does it when the author presses the button beside the
refusal**, and not before. `mpdf-003` Phase 25, which is the one thing in this
app that reaches the network at all.

`app/src/remote.rs:fetch` is `cli/src/main.rs:fetch_image` copied value for
value — a 30 s global timeout, a 20 MB cap read with `limit(FETCH_LIMIT + 1)`,
ten redirects, 2xx only, no gzip, no cookies, no cache, `Content-Type` ignored —
and its agent is built once for the process where the CLI builds one per run. The
duplication is `read_assets_with`'s own: the two wrappers report their errors
differently. A failed fetch is refused in the CLI's sentence, `cannot fetch <url>
for the image at line N: <reason>`. **A proxy named in the environment is
honoured and one set in System Settings is not**, a window launched from Finder
carrying neither — a recorded limit.

**Consent is per project folder and per site, and it is remembered across
launches.** `app/src/document.rs:sites_file` is the third file beside the store
and the settings, `sites.json`, a `BTreeMap` from canonical root to a sorted list
of sites; `read_sites` forgives a malformed one as nothing allowed and
`write_sites` reports a failed write, and `document.rs:writing_the_sites_does_not_touch_the_store`
holds the other two files byte for byte. The site is the URL's host, lower-cased,
read with `ureq::http::Uri`; a URL with no host is its own site. **The fetched
bytes are held in memory, for the process, and never on disk** — so after a
relaunch offline the page says `cannot fetch …` for images it drew yesterday,
which is the stated cost of having no second store with its own staleness.
`project/src/remote.rs:Web` holds both, and **they are held apart**: `Session::open_at`
replaces the allowed sites from `sites.json` *before* `load` compiles, and carries
the fetches across, so a project that never allowed a site draws none of that
site's images out of memory.

**`Preview::absorb` is the one place a fetch is claimed.** Every compile path
reaches it — the typing loop, the watch loop, an open, a reload, a save-as, the
worker's own compile — so none of them can strand a URL, where claiming at the
call sites would be eight places to remember. A URL newly named on an allowed
site is marked `Waiting` and **waits out `remote::SETTLE`, one second**, before a
request goes out; it is dropped if the text stopped naming it meanwhile, which is
what makes a URL edited in place one request instead of one per compile. **Only
the press skips the settle.**

`Session::new` starts **the fetch worker**: one thread holding a `Weak` to the
state, so a dropped `Session` takes its `Preview` — which holds the channel's only
sender — and the worker's `recv` ends. Each claim gets a thread of its own that
waits out the settle, moves `Waiting` to `Fetching` under the lock if the text
still names the URL and the open project still allows its site, fetches off the
lock, records what came back, and then runs the typing loop's own three steps.
**A keystroke never waits on the network**, which is Phase 22's property over a
new source of work. The fetch and the render are seams `Session::new` takes:
`main.rs` passes `remote::fetch` and `Compile::run`, and every test passes fakes,
so no case in the suite touches the network.

**A compile reads only the fetches on sites the open project allows**, and
records the generation of each; `absorb` promotes those from `Arrived` to `Done`
**only at the generation it read**, so an older plan landing after a retry cannot
clear the line for bytes it never compiled. `Render::refused` carries the URL a
compile was refused on — read off `Error::UnfetchedImage` before it becomes a
string, or off the `cannot fetch` sentence's own `Unread` — and `Preview` keeps
it, clearing it on every error written that did not come from a render.
**`Preview::status` leaves `error` out exactly while `refused` names a URL that is
waiting, being fetched or back and not yet compiled, on an allowed site**, so
`core`'s *"no image fetched"* never stands beside *"Fetching"*. Every other URL
shows its error as it always has.

`Web::line` words what the page draws, taking the first of four cases that
applies: a URL on a site not allowed — *"1 image on cdn.example.com is not
fetched."* — with **Fetch images from the web**; a URL on its way — *"Fetching 1
image from cdn.example.com…"* — with no button; a fetch that failed — *"1 image
could not be fetched."* — with **Try again**; and otherwise nothing, a URL only
waiting included. **The order is what keeps consent narrow**: both buttons run
`Session::fetch_images`, which allows every site the document names, so **Try
again** can only appear once no site is waiting to be allowed. That command writes
`sites.json` before it wears the sites, claims without the settle, and **always
sends the worker a compile** — the bytes may already be in memory from a project
that allowed the site first.

**There is no placeholder while an image is on its way**: a box of a guessed size
draws a page that is not the document and moves when the real image lands. The
last good page stays under the line, marked stale. **A URL is not watched** —
`classify` never sees one — so an image that changes at its URL arrives at the
next launch. Taking consent back is `specs/desktop_app_spec.md` OQ-19, and the
licence notice the TLS stack asks for is OQ-20.
