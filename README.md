# Letur

A macOS window that shows the PDF while you write it. Open a markdown file, type in the
left pane, and the typeset page redraws on the right — the whole project, not just the
file you are in.

Letur is the desktop front end for [md2pdf](https://github.com/Ivapo/md2pdf), the engine
that turns one markdown file — or a master and the sections it names — into one typeset
PDF. It wraps that same crate, so it converts exactly what the command converts and
refuses exactly what the command refuses, in the same words. Everything happens on your
machine: no server, no SaaS, and no LaTeX toolchain.

**What the markdown may contain** is the engine's to say, and its README says it.

## Install

```console
$ cargo tauri build
```

That writes `target/release/bundle/macos/Letur.app`, and a `.dmg` beside it under
`target/release/bundle/dmg/`. Drag the `.app` into `/Applications` and launch it from
there; double-clicking a `.md` file opens it too.

**The bundle is not signed.** Copy it over — a USB stick, `scp`, a shared folder — and
it runs. Download it or send it by AirDrop and macOS marks it quarantined, and
Gatekeeper refuses it until you allow it by hand in System Settings → Privacy &
Security. Signing and notarising it needs an Apple Developer account, which this build
does not have.

## Use

```console
$ cargo tauri dev
```

That opens a window. Press `⌘O`, or the Open button, and pick a markdown file: the
window finds the document that file belongs to, puts its text in the left pane, lists the
project's files beside it and draws the page on the right. `cargo tauri dev`
needs the Tauri CLI (`cargo install tauri-cli`); without it,
`cargo run --release -p letur` opens the same window and skips the
rebuild-on-change.

**Type in the left pane and the page follows.** It redraws when you stop typing, and
**the PDF is what the pane says, not what the file says** — so the page shows your
unsaved work. `⌘S` writes the pane back to whichever file it is
holding, and the foot of the window says `saved` for a moment. **The floppy in the
header, or `⇧⌘S`, is `Save as…`**: it saves **wherever you point it**, and it takes the
same kinds the panel lists — `.md`, `.bib`, `.yml` and `.yaml`. **Where it lands decides
whether the pane goes with it.** Inside the project, the pane holds the new file and the
next `⌘S` goes there. Outside it, the file is written and **the pane stays where it
was** — the save is a copy, and the bar says `saved as <name> in <folder>` so you can
see which of the two just happened. **The window keeps compiling the project it had**
either way. If you save a file your document names, the page redraws with it; if you
save the file you were editing under a new name — inside the project — the page goes
back to what the old one says on disk, because that is the file the document still
names. **The bar along the foot of the window names whatever the pane is holding** — the
file you are typing in, which once you have clicked another file in the list is not
the file the page beside it comes from, or the picture, if you have clicked an image
row to look at one. **That bar also carries the view and fit controls**: the panel and
the line numbers as two small marks on the left, and the fit beside the appearance
button. **The header is Open and Save as…** — two marks and the status line, and nothing
that decides what the window shows.
**The button in that bar sets the window light or dark**: it starts on whichever your
system is set to, and clicking gives you the other one. It remembers your choice the
next time you open the app, and it changes only the app's own chrome — the page stays
the white it will print as. Until you press it the window follows your system, so a
machine that switches at sunset takes the window with it; pressing it once settles
that, and there is no way back short of clearing the app's own settings. Drag the divider to give either side more room. **The `Lines` mark in the bar, or `⌘L`,
numbers the pane and marks the line your cursor is on**, so an error that names a line names
somewhere you can see.

**The pane colours what it is holding**, and it does so whether the numbers are on or not:
markdown, a `.bib` and a `.yml` or `.yaml` each get their own reading, so a heading, a key,
a fenced block and the marker that reads a section in are all told apart at a glance — and
an include marker naming a file the project does not have wears the same red the errors do.
**The colours are this dialect's and not markdown's in general.** A citation, a
cross-reference to a figure, a footnote and a section marker are four different things
wearing four different marks, and a caption, a `:::` group and a `{#name}` are told from the
prose around them — so the constructs that are *not* ordinary markdown are the ones the pane
is most careful about. **Headings do not grow**: they are bolder and a different colour, and
never a larger size, because the letters have to stay exactly where the text is or the
colour slides off the words. The colour is a hint and never a verdict — the page beside it is what says whether
your document compiles.

**Save the file in another program and the page redraws too** — with one exception. The
window watches the whole project, so editing a section, a figure or the bibliography
elsewhere redraws it as well, and adding or removing a file shows up in the list. If the
pane holds unsaved edits when the file changes underneath, the app keeps your text and
says so rather than choosing for you: save to write the pane over the file, or press
`Discard` in the bar it says it in to take the file instead. It never merges the two.

A document that will not compile leaves the last good page on screen, dimmed, with the
error above it — the same sentence the command prints — and the page comes back when you
fix it.

**A redraw opens the page on the heading you are writing under.** The app draws the page
itself, so it knows where you are: it follows your cursor to the nearest heading above it,
which is as close as it can get without one. Opening a file, and taking one that changed
underneath, still start you at page 1. In a document written across several files it
follows the headings in the file the pane is showing — so editing a section takes you to
that section's own pages in the whole document, which is the thing the file list below is
for. A master that is only a list of sections has no headings of its own and opens at
page 1.

**Dragging the divider or resizing the window leaves you where you were.** Nothing about
the document moved, so nothing about your place in it does either — the page just refits
to the width you gave it and comes back sharp when you let go.

**The control beside `Lines` says how big the page is drawn.** `Fit width` is where it
starts and is what the pane has always done; `Fit page` puts a whole page in the window;
and the percentages below them pin a size of your own, up to 400%, where a page wider
than the pane scrolls sideways. Changing it keeps your place in the document, and the
page is drawn again at the new size rather than stretched to it. Opening a file goes
back to `Fit width`.

**The page is text, not a picture of text.** Select and copy from it as you would in any
PDF reader — on a long document, from the pages around the one you are reading, which are
the ones the app keeps drawn — and click a cross-reference to jump to the figure, table,
equation, footnote or reference it names. A link out to the web does not open: the app fetches nothing and opens
nothing, so those links are inert on the page.

The header says where the page stands — `current` with the time the compile took, or
`stale` when the last one failed and the page you are looking at is the older one.

**`File → Save a Copy…` writes the PDF where you ask**, offering the path of
the file that compiles with a `.pdf` extension — the page is that document's, whichever
file you happen to be editing. It writes the page on screen and compiles nothing, so
the file and the page cannot disagree, and it is byte for byte the file `md2pdf` writes
for the same document — while the pane and the file say the same thing, which they do
until you type. A page that is stale, or no page at all, is refused rather than written.

**A `.md` file double-clicked in Finder opens in the app**, once it is the handler for
that extension. macOS gives an installed editor the first claim on `.md`, so if
double-clicking still opens your editor, pick a markdown file, press `⌘I`, and set
*Open With* to Letur followed by *Change All*. Double-clicking a section opens its whole
document, per the project rule below. Opening a second file this way switches the window
to it, and **unsaved edits in the pane are lost** — the same as reopening from the Open
dialog. Save first if you want to keep them.

**The window opens a project, not just a file.** Open any markdown file and the app
looks for the document it belongs to: if a file in the folder above names it as a
section, that folder is the project. So double-clicking `sections/method.md` gives you
the whole book, not one chapter of it. A file nobody names is its own project, which is
every single-file document and is what the app has always done. The one limit: a section
more than one folder below its master roots at its own folder — open the master instead.

**Every document lists its folder down the left**: the markdown, the bibliographies and
the images under the project's root, folders and all, with the file that compiles marked
`◀ main`. A file the document names that is not on disk is listed too, struck through,
because that is the file the next compile will refuse on. Hover a markdown row and a
`main` button appears: click it and that file becomes the one that compiles, and the app
remembers your choice the next time you open that folder. It remembers it in its own
Application Support folder and writes nothing into yours. The `Files` mark in the bar, or
`⌘B`, folds the whole list away and brings it back; a folder heading is a button of its
own, and pressing it folds just what is under it — the mark beside the name says which
way it is set, and the panel gives the width it was using back to the pages.

**The `+` beside `Files` makes a new one.** Type the path you want it at, relative to
the project — `sections/discussion.md` — and the app creates it empty and lists it. It
takes markdown and bibliographies, `.md`, `.bib`, `.yml` and `.yaml`, and it will not
write outside the project or over a file that is already there; it says so beside the
field if you ask for either. It does **not** add the include marker to your master:
where a new section belongs in a document is your decision, so write `[](…)` where you
want it. Folders are not created — make the folder first, in Finder.

**Hover any other row and a small basket appears beside it: it moves that file
to the Trash.** It goes red under the pointer and is quiet the rest of the time.
Not a delete — the Trash, so the file is in there under Finder's own `Put Back`
if you did not mean it. **Nothing asks you twice**, and that is
deliberate: a confirmation is what stands in for an undo where there is none,
and this has one. The file that compiles has no such button — set another file
as `main` first if you want that one gone — and neither does a struck-through
row, which names a file that is not there to move. If you delete the file the
pane is holding, the pane goes back to the one that compiles; if it holds
unsaved edits the app will not delete it, and says so with `Discard` beside the
sentence. Delete a section your document names and the row stays, struck
through, and the next compile refuses by that file's name — which is the app
telling you the document is broken, not the app breaking it.

**Click a file in that list and the pane opens it, while the page keeps showing the whole
document.** So you can write chapter three and watch the book: the file that compiles and
the file you are editing are two different things, and the list marks each. `⌘S` writes
the one in the pane. If it holds unsaved edits the app will not switch — it says so and
offers `Discard`, the same way it does when a file changes underneath you.

**A bibliography opens the same way.** Click `refs.bib` and it is in the pane; change a
title and the reference list in the page beside it redraws with the new one, before you
have saved anything — the same rule as everywhere else here, that the page is what the
pane says rather than what the file says. What it cannot become is the file that
compiles: `main` stays markdown, so no `main` button appears on a bibliography row. The
caret's own page does nothing while you are in one, because a bibliography has no
headings for it to follow.

**Click a figure and it opens over the pane, which keeps the file it was holding.** So you
can check that `emit.svg` is the diagram you meant without leaving for Preview: nothing
compiles, the page goes on showing the whole document, and `⌘S` still writes the markdown
you were editing. `Escape`, or `Back to the text`, puts it away, and so does opening any
other file. A PDF is a legal figure here, so the list holds one — it says so rather than
drawing it. Bibliographies are listed but not opened.

## Developing

The Rust suite needs nothing beyond a toolchain:

```console
$ cargo test --workspace
```

**The browser harness needs three things on your machine.** `bun`, Playwright's
Chromium, and the engine's own CLI — every document the harness serves is compiled by a
`md2pdf` binary on `PATH`, because this workspace holds the window and not the engine:

```console
$ cargo install --locked md2pdf-cli
$ cd app && bun install
$ bun harness/checks.mjs
```

`app/harness/serve.mjs` serves a copy of the front end with a stub in its head, so the
real page can be driven outside a window; `bun harness/checks.mjs --falsify` breaks the
page twenty-three ways and checks that each break fails the one clause that owns it.

## Licence

The code is MIT; see `LICENSE`.

The app draws its page with Mozilla's `pdf.js`, vendored as two modules under
`app/dist/pdfjs/` and licensed Apache-2.0; see `app/dist/pdfjs/LICENSE`.

The fonts are the engine's and ship inside `md2pdf-core`; its README carries their
licences.
