---
title: desktop-panel
sources:
  - app/dist/index.html
  - app/src/document.rs
  - app/src/preview.rs
covers: >
  the desktop app's file panel: the column every open document draws and the two
  states it keeps apart, the flat entries it is drawn from and the folders
  derived rather than sent, the two folds the page holds and the one of them
  that has to reach the rows, the three gestures on a file row and the two marks
  it may carry, the fourth that is a folder row's own and the invariant it does
  not break, the mark that says which way a disclosure is set and where it is
  drawn, the two opens that empty the folds and the third caller that is why
  they are not emptied lower, the create that expands what it would otherwise
  land behind, the two rows the delete is not on and the edge two buttons share,
  the one refusal here that reaches `fail` and the two that keep their own
  surfaces, the two kinds a row's body puts in the pane and the second kind test
  that has to move with the first, the panel's own gesture that is on no row and
  the two things the page holds, the surface over the text pane and the three boxes that place it,
  the sentence it shows where a figure cannot be drawn, the sequence its read
  carries and the three ways back, and the disk half that is walked twice beside
  the missing half that follows the text
max_lines: 235
generated: 2026-09-03
---

# The desktop app's file panel

The left column beside the text pane: what it lists, what a row does, what a
folder row now does, and the figure surface an image row raises over the pane.
`rules/desktop-panes.md` has the two panes this sits beside and what checks the
page; `rules/desktop-project.md` has the Rust that decides what is in the list.

**Every open document draws a panel**, a left column beside the text pane at
`max-width: 40%` listing the project: every file under the root that this
dialect can read, with the one that compiles marked. A lone `.md` naming nothing
draws one too, which is what lets an author build a first section without
leaving the window, and is the visible reversal of the panel `mpdf-008` Phase 4
shipped.

`Status::entries` is a flat `Vec` of `document::Entry` — `{ path, kind,
missing }`, `path` root-relative with `/` separators, `kind` one of `markdown`,
`bibliography` or `image`. **A directory is never an entry**: `parts` derives the
headings and the indent from the path's own segments, a thing a page can do and a
thing a nested node type would make `Status` carry twice. **A derived heading
still carries its whole root-relative path** — `parts/ch1`, not `ch1` — because
that is what the folds below are keyed by and what tells two `ch1`s apart; the
segment is all it draws. `Status::main` rides
with it, spelled the same way — root-relative and not the bare file *name* it
carried while the panel listed one document's parts, or the page could not match
it to a row. Both cross for the anchors' reason: the status is already fetched on
the path that draws, so the panel costs no command.

**Absent and folded are two states.** `hidden` is exactly *no document is open*,
and it takes the toggle with it; `.collapsed` is a reader who folded the panel,
and the toggle stays so they can get it back. `clear()` sets the first, because
a panel drawn for every document would otherwise hold the previous project's
files across the open that replaces them. **The fold is the page's own**, a
variable reapplied on every status rather than a field in `Preview`: §2's rule is
about state that decides behaviour, and a fold decides nothing but its own
drawing. The store this app now keeps is not a precedent for it — a main is a
decision about the document, a fold is where a scrollbar was.

**There are two folds now and only one of them redraws anything.** The panel's
is one class on one element, so its press writes `.collapsed` and stops. A
folder's changes which rows *exist*, so it has to reach `panelRows` — and
`parts` runs off a status the page does not cache, so the press goes through
`refresh()`, an `await invoke('status')` round trip. That is the one shape that
adds no state: caching the last entries would put a second thing in the page,
and hiding rows in place would move the skip out of `panelRows`. `status` runs
no compile, and the panel is already rebuilt on every one of them. **Two
consequences follow from routing a page-only gesture through a status fetch**,
neither a new rule: `report` clears `#error` and `refusing` on every status, so
a fold press dismisses a standing refusal that a keystroke would otherwise have
cleared — that is `report`'s documented ownership of the bar reaching one more
caller; and a press landing between a compile's announcement and the page's own
`refresh` fetches `current_pdf` and follows the caret's page, a redraw that was
arriving anyway and is superseded correctly by `renderSeq`.

**The fold set is a `Set` of root-relative folder paths, and the test on it is a
*proper* prefix.** An entry is skipped when a collapsed folder is a path prefix
of it, headings included, so `parts` folded takes `parts/ch1/deep.md` **and the
`ch1` heading**; a parent test would leave that heading standing above nothing.
The collapsed folder's own row is still drawn, or there is no way back, and
`panelRows`' shared-segment bookkeeping advances on the entries Rust sent rather
than on the rows it drew. **The two opens empty the set and `clear()`
deliberately does not**: `clear()` has a third caller, `setMain`, which goes
through it because Rust rebuilds the preview and restarts its counters, so
emptying there would unfold the whole tree on a `main` press against a tree that
did not change. **A create expands every ancestor of its new file** — the field
takes a whole root-relative path, and a create whose row does not appear is
indistinguishable from one that failed; every ancestor and not the immediate
parent, `parts/ch2` removed alone still being hidden under a folded `parts`.
Nothing else re-expands.

**A fold may hide the row carrying `◀ main` or the one the pane is holding**,
and that is the reader's own action on their own panel: `#edited` in the footer
still names the pane's file whatever the panel is doing. Whether a collapsed
folder should *say* it holds one of them is `mpdf-010` OQ-9, left open rather
than built because it would make a folder row carry a mark.

**A file row carries three gestures and can carry two marks.** The body of a markdown
row is a `button.name` that puts that file in the pane; the `main` button appears
beside it on hover and on focus and sets which file compiles; the basket
beside *that* moves the file to the Trash. **One is a word and one is a mark**,
by the rule above: `main` names a state, the delete names an action no glyph
names, and the two are one glance apart. The first two were kept apart before
there were two of them rather than after. **`here` is the file that
compiles and `.holding` is the file the pane shows** — one row at every open, two
from the first click — and `.holding` wears the text pane's own `--ground` rather
than `--band`, which sits a point from the panel's `--chrome` and would be
invisible in both themes. **An image row's body is a button too, and it does
something else**: it shows the figure over the text pane, leaving `edited` where
it was. **A bibliography's body opens like a markdown row's**, where OQ-2
resolved: the compile is `main`'s, so a `.bib` in the pane feeds its unsaved
bytes to the citation pass while the page draws the whole document, and the
`main` control still appears on markdown alone. A marked-missing row is the one
that opens nothing and says so in its `title`, naming a file the disk does not
hold. **The `title` is a second chain with its own kind test and it moves with
`opens`'**, or `— not edited here` lands on exactly the row the pane is
holding; with both moved that branch is unreachable for all three kinds and is
left standing, an unreachable branch putting no sentence in the window. So the
panel is still rebuilt whole on every status, and that is still
right: **the rows hold no selection.** Both files live in Rust and arrive in the
status, and each control reads its path off the DOM at the moment it is clicked.

**A folder row is the fourth gesture and it does not break that**, which has to
be written down because a careless reading has the invariant falling here. Its
`<span class="name">` became a `<button>`, the move a file row already makes for
a row that does something, and it carries `aria-expanded` — a disclosure that
does not say which way it is set is an asymmetry with the panel's own control,
and the attribute is also what the mark is keyed to, so one declaration serves
both. What it holds is a *fold*, which is not a selection: a **file** row still
holds nothing, and a fold decides nothing but which rows are drawn. **The mark
is a glyph and it is drawn in `::before`.** `▾` and `▸` name a disclosure
exactly, are text-presentation and take `color`, so the glyph-or-drawn rule
decides it in one step and this draws nothing; `::before` and not the button's
text because the harness reads `.name`'s `textContent` against the bare path
segment. The turning chevron `#views` refused is a different question — that
refusal is about 10px in a bar whose entire ink is one colour, beside a second
toggle marked another way, and a panel row is `12px/1.5 ui-monospace` beside no
such neighbour.

**The delete is on every row but two, and nothing asks first.** The `main` row
has none — its file is the one `Session::trash` refuses — and neither does a
marked-missing row, which names a file the disk does not hold. An image and a
bibliography both get one: the panel lists them, and a figure the document
stopped naming is a thing to be rid of. **No confirmation**, because the Trash
is the platform's own undo and a confirmation is what stands in for an undo
where there is none — so the button holds no state and `parts` may go on
rebuilding the panel whole.

**Two buttons share one right edge inside `.controls`, and that is a
correction.** `margin-left: auto` was on `.set` "because only one of the two is
ever on a row", meaning the button and the `◀ main` mark; a non-main markdown
row now draws two buttons, and two elements each claiming the free space would
push the first off that edge. So the group claims it, the buttons lose it, and
the mark keeps it alone on the row that has no buttons at all. `.trash`'s
stroke wears `--alarm` only under the pointer, through `currentColor`, so a row
does not read as a warning at rest. **The padding that shrank the panel names
`.trash` and not the rule both buttons wear**, which would take `main` with it.

**A refused delete is the one refusal in this panel that does reach `fail`**,
against the rule below, and the exception is argued rather than overlooked: it
is `openInPane`'s and `setMain`'s own route, and none of the three sentences is
reachable from a row — the `main` row draws no button, and every other row came
out of Rust's own listing, so only a hand-typed command or a file vanishing
between the walk and the click gets one. The two refusals a reader *does* meet —
the create's, and the figure's — keep their own surfaces.

**The panel's own gesture is not on a row, and that invariant is why.** A `+` at
the end of the `<h2>` reveals a field taking a whole root-relative path, and the
create is `document::create_file`. **It sits in `#files` and outside `<ol
id="parts">`**: `parts` replaces that list whole on every status, so a field
inside it would lose what the author had typed to an event they did not cause —
the rows holding no selection is what buys that rebuild, and this is that same
fact from the other side. Showing or not is the page's own, for the fold's
reason, and `clear()` closes it and empties it and its sentence with the rows,
which nothing else there touches. **A refusal is drawn beside the field**, in
Rust's words and placed as every status sentence is, reaching neither `fail`,
which would mark the compiled page stale for a gesture that compiled nothing,
nor the divergence bar, whose `Discard` names nothing to discard. It is
`saySoInstead`'s exception, and the one refusal here a reader reaches by typing.

**The figure is a view over the text pane and not a third pane**, the way `Lines`
is a view: `#viewer` is a `<figure>` positioned absolutely over `#text`'s own
column inside a `<main>` that carries `position: relative` for it, and nothing it
does reaches `edited`, the buffer, the compile, the bytes or the anchors — `⌘S`
still writes the markdown, the page still shows the whole document, and `Status`
gains no field. **It covers the textarea rather than replacing it** because
`#divider`'s drag reads `#text.getBoundingClientRect()` at every `pointerdown`
and a hidden textarea measures zero. `placeViewer` mirrors that column's
`offsetLeft` and `offsetWidth` on five occasions — a show, a window resize, the
end of a divider drag, the panel fold and the `Lines` toggle — and **that
enumeration is the exception to this file's own rule** that the page watches the
pane rather than the causes: an observer over `#text` never fires for a fold,
which moves its left edge without changing its size, and the one over `#pages`
does not fire while that pane is hidden. The sheet is `flex: 1; min-height: 0`,
which is what makes its top padding free — flexbox distributes free space over
items' **outer** sizes, so a figure's `max-height: 100%` resolves against a
content box that already excludes it. `box-sizing` is not what does that, and two
drafts of `mpdf-010` Phase 5 said it was.

**The surface is placed off the three boxes that decide its column, not off a
list of gestures.** `placeViewer` mirrors `#text`'s `offsetLeft` and
`offsetWidth`, and a `ResizeObserver` over `#files`, `#lines` and `#text` drives
it: those two are all that sit to the text pane's left, and `#text`'s own width
is what the divider and the window set. **An enumeration of gestures was tried
and was wrong**, which is this file's rule earning itself a third time: `#files`
is `flex: 0 0 auto` and `#lines` has no width of its own, so both are as wide as
their contents, and `parts` rebuilds the panel on every status while `relines`
rewrites the gutter on every keystroke — a project gaining a longer filename, or
a document crossing 99 lines, moves that column with no gesture at all. The
observer cannot loop: `#viewer` is absolutely positioned and out of flow, so
nothing it writes resizes anything observed.

**A figure that cannot be drawn says so in the sheet, and never through
`fail`.** Three sentences land there: `document::asset_bytes`'s refusal, in
Rust's own words and placed the way a compile's is; the `.pdf` line; and an
undecodable figure's, both of those the page's own, as labels about a kind of
file. `app/dist/index.html:fail` is refused for all three because it marks the
compiled page stale, which a click that compiled nothing must not do. An
`.svgz` is gunzipped before the blob is minted — it is in
`md2pdf_core::IMAGE_EXTENSIONS` and a blob URL carries no `Content-Encoding`, so
handed over as-is it drew a permanently blank sheet.

**The read crosses IPC, so the surface carries a sequence.** `viewSeq` is
`renderSeq`'s idea applied to a second asynchronous pass: every entry to
`showAsset` and every exit through `hideAsset` takes the next number, and a read
that comes back to find it moved draws nothing and reports nothing. Without it
each of the three ways back was undone by the bytes arriving after it, and the
path label — written before the read rather than after — named the last row
clicked while the sheet held the last one to arrive. **`Escape` is
unconditional for the same reason**: while the read is in flight the surface is
still hidden, so a guard on `!viewer.hidden` made the key do nothing in exactly
the case a reader presses it.

**Three ways back, because the reader arrives by three routes**: the surface's
own control, `Escape`, and clicking a markdown row that opens — which already
means *put that file in the pane* and must not leave a picture over it.
`clear()` closes it too, an open being a new project. The markdown row the pane
already holds stays inert, so clicking the row you are on while a figure is up
does nothing; that is accepted rather than fixed, the alternative being a row
whose drawing depends on page state. **A `.pdf` row draws no figure and says so
in a sentence the page writes itself** — a deliberate exception to "the status is
placed and never composed", on the ground that this is a label for a file kind
and not a status about the document. `app/dist/index.html:fail` was the other
route and it marks the compiled page stale, which a click that compiled nothing
must not do; `document::asset_bytes` is never called for a `.pdf` at all.
`mpdf-010` OQ-8 carries whether the vendored `pdf.js` should draw one instead.

**The disk half is stable and only the marked-missing half moves.**
`document::files_under` walks the tree at an open and at a `Change::Tree` event,
and `Preview` holds the answer; `document::merge` adds the sections the master
names that the walk did not find, on every status, off `Preview::sections` and
off no directory. So a half-typed marker moves one row where the shipped section
panel lost all of them — strictly less motion than `mpdf-008` §2 accepted — and
`status()` still reads nothing from the disk.
