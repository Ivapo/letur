---
title: desktop-panes
sources:
  - app/dist/index.html
  - app/tsconfig.json
  - app/typecheck.mjs
  - app/src/document.rs
  - app/src/preview.rs
  - app/harness/stub.mjs
  - app/harness/serve.mjs
  - app/harness/checks.mjs
  - app/driver/drive.mjs
covers: >
  the desktop app's two panes: the one file the front end is, the text pane and
  the pages the app rasterises the artifact onto, the divider between them and
  the press's own default action its drag cancels, the wrapper a page is and the
  canvas and two layers inside it, the vendored renderer and its worker, the retained document a geometry-only redraw draws from, the
  redraw that moves the reader and the three causes that decide where to, the
  anchor path that opens on the author's own page and the one file it is
  filtered to, the status the page places and never composes, the bar along the
  top and the two actions it keeps, the marks it draws for them and the ground
  that says they can be pressed, the rule that decided which of the two bars a
  control belongs to and the third thing that is neither, the bar along the
  foot, the five cells and one rule it carries and the one control each of its
  settings has, the two of those that also take a menu event carrying no state,
  the two marks it draws rather than borrows, the file it
  names, the one it does not and the figure that is neither, the receipt a save
  leaves and the cell that costs nothing while it is silent,
  the appearance the author chooses and the four token blocks that wear it, the
  button's two positions against the three values behind them and the state that
  is not a destination, the mark that is what is worn rather than what is
  offered, the one value in the bar the page asks for rather than decides, the
  auto margin that moved and the reading that can tell it moved, the panel that
  is a rule file of its own and the two places it still touches this one, the text the reader can
  select and the stream it is read off, the link filter and the destination a click resolves, the scaffolding the
  bundle does not carry and the app supplies, the one element that is both the ink
  a reader sees and the ruler the gutter is measured from, the textarea gone
  transparent above it and the caret it keeps, the split that stopped the layer
  being gated on the gutter and the two memo pairs a single one would break, the
  readings that came off the keystroke path and what they were worth, the ink
  suppressed for a drag and for any width the mirror was not built at, the three
  grammars chosen off one extension and the one of them written once and used
  twice, the four inks and the properties the layer may not carry, the marker that
  is the whole of its own line and the compiler that is right where it disagrees,
  the gutter whose rows are as tall
  as their lines render, the follow
  that keeps the numbers against their lines, the caret's two marks, the third
  the pane does not draw, and the pane that loses both when it empties, the check that reads this file and the
  two declarations it holds to each other, the harness that drives it in two
  engines and the copy it drives rather than the file, the boundary it records
  because the DOM cannot show one, the twenty-three clauses it
  asserts as properties and the twenty broken pages that falsify them, the rule
  that decides what a clause may read as well as what it may assert, the second
  rig that drives the shipped binary instead and which of the three kinds of
  claim belongs where, and the seven defects none of them reaches
max_lines: 720
generated: 2026-09-03
---

# Desktop panes

What the window puts on screen, and the geometry it keeps while it does.
`rules/desktop.md` has the crate, the commands, the file I/O, the watch and the
bundle; this file has the two panes those things feed.

## The page

`app/dist/index.html` is the whole front end — one file, inline CSS and JS.

Two panes: a `<textarea>` at 40% of the width, a divider the reader drags, and
`#pages`, a scroll container holding one `div.page` per page of the document.
**The text pane draws no glyphs of its own** — it is transparent, and what a reader
sees is `#mirror` underneath it, one element carrying both the ink and the layout
the gutter is measured from. It stays the editor: the buffer, the caret, the
selection, the disabled state and the divider's geometry are all still the
textarea's, and every change goes straight to Rust, which holds the buffer. **No
autocomplete and no formatting commands**, which were the other two halves of the
sentence this replaces.

**The divider's press cancels its own default action, and the capture is not what
does that.** `setPointerCapture` routes the gesture — the `pointermove`s and the
`pointerup` come back to the divider, which is all the resize needs — and says
nothing about the **default action** of the press that opened it, which on a bare
element is to anchor a selection. Unprevented, that damaged the pane three ways and
the direction decided which showed. **Narrowing lost focus** to `body` at the
`mousedown` and still held it at the `pointerup`, so the keystroke after the drag
was swallowed; WebKit additionally left a `"\n"` selection where Chromium left
nothing. **Widening kept focus and moved the caret**, 65 to 279 at a 30px span with
nothing selected and the pointer never leaving the divider — the quietest of the
three, since nothing highlights and the next keystroke lands two hundred characters
away. Past a longer span a widening drag also selected 21 characters, **and the
clamp does not cause that**: 30 and 60 select nothing; 88, 120 and 160 select the
same 21 with the pointer still on the divider; 200 reaches the `room - 160` ceiling
and selects those same 21. The pages contribute only that there is something to
select — the `.textLayer` is real text, transparent, and deliberately selectable.
**Why any of the three happens is not established**: `down.preventDefault()` is the
whole repair, measured across both engines, both directions and both spans rather
than reasoned from a mechanism. It is that and not `user-select: none`, a claim
about what may be selected *in the divider* which moves no focus, and not a class
on `body`, which would be two writes that must pair across a gesture.

**The app rasterises the artifact itself.** `pdf.js` is vendored as two static
ES modules under `app/dist/pdfjs/` — `pdf.min.mjs` and `pdf.worker.min.mjs`,
1,717,067 bytes together, with the Apache-2.0 `LICENSE` beside them — loaded
from a `<script type="module">`. `pdfjs-dist` ships browser-ready modules, so
there is still no bundler, no npm at build time and no node. The worker is not
optional: parsing and rasterisation go off the main thread, because the pane
re-renders every time the typing stops and rasterising on the main thread would
freeze the text the author is typing into.

`openPdf` takes the bytes and a page. **`getDocument` transfers the buffer to
the worker**, so the bytes cannot be read twice. **The loading task is retained
between renders** and a re-render caused by geometry alone draws from it and
invokes nothing — which is not an optimisation, since `current_pdf` refuses
while the preview is stale and a pane that re-fetched on a resize could not
re-fit a stale page. What is destroyed on a re-open is the *task*, not the
document proxy, which has no `destroy` of its own; the task owns the worker, and
dropping one undestroyed leaks a thread per compile.

**A page is a wrapper and not a canvas.** `#pages`'s children are `div.page`,
each holding the canvas plus a text layer and an annotation layer. Three boxes
have to share one origin and one size, and only an element holding all three can
be the child `drawPages` indexes and swaps, the prune removes, and the reader's
place is measured against — so `.logical` sits on the wrapper, `size()` writes
that box, and the canvas inside takes `width: 100%; height: 100%`. Layers as
siblings of the canvas *in* `#pages` would break the indexing; layers under a
wrapper that is not itself the child would be dropped by `replaceChild`. A page
removed takes its layers with it, which is the second reason the wrapper is the
child and is why `clear`'s `replaceChildren` disposes them too.

**A page is swapped in only once it holds pixels.** Setting `canvas.width`
clears it, so re-rendering in place would flash empty pages at every rest — and
since the whole page is swapped, its layers ride that rule too and are built
while it is still detached, which nothing in their path needs layout for.

**The text comes off `streamTextContent` and never off `getTextContent`.** That
call is a `for await (… of readableStream)` over its own stream, and
`ReadableStream.prototype[Symbol.asyncIterator]` is `undefined` in WKWebView —
Chromium and Gecko implement async iteration of a readable stream and WebKit does
not, so it rejects with *"undefined is not a function"* from inside the minified
bundle, which names nothing useful. `TextLayer`'s constructor takes the stream
itself and drives it with `getReader()` and `read()`, so handing it
`page.streamTextContent()` obeys the rule by construction rather than by
remembering it. The showcase carries 1184 text items across its six pages —
re-measured 2026-09-01 by `mpdf-007` Phase 5 through the vendored `app/dist/pdfjs/`,
served with the PDF to a browser, which is the engine that produced the number; the
unedited source read 1140 under both binaries, so the whole delta is that phase's
showcase edit. Nothing in `cargo test` reads it, so it is a phase that edits the
showcase that moves it — and **the showcase it means is `tests/fixtures/samples/showcase/`,
frozen at `mpdf-011` Phase 1** rather than the engine's, which goes on changing next door.
That is what stops this number and the sixteen below rotting unobserved: a dialect phase
cannot move either any more, so only a phase of *this* repository's can.

**Only a link carrying an internal destination is rendered**, filtered before the
layer is built. An external one has no element, no `href` and nothing to
activate, so *refused* is something a second person can check rather than a
behaviour observed not to happen — `mpdf-003` §1.1, no servers and no network,
ever. The same filter needs no second rule for a markdown link *with text*
pointing at a figure: it reaches the PDF as a `/URI`, `pdf.js` refuses it on its
protocol, and it arrives with `url` and `dest` both null. The showcase's sixteen
internal links are seven cross-references *plus* the footnote marks and their return
arrows: Typst attaches a citation's destination to the element its renderer tags as
the entry, which a numeric `[1]` is and an author-date mark is not, so
`citations: author-date` took the count from twenty when `mpdf-007` Phase 5 gave the
showcase that key — measured 2026-09-01, four marks and four links fewer. Under
`numeric` the marks link too, and the filter delivers a document navigable three
ways rather than one.

**The link service is three methods written here, not a port.**
`SimpleLinkService` is not in the bundle, and its route sets a real `href`, which
in this window would navigate the webview off `tauri://localhost` with nothing to
come back with — so the element gets a fragment and the click is answered by
`goToDestination`, which resolves the destination, takes its page index, turns its
own coordinate into a CSS offset through that page's render viewport, and scrolls
the container. **The destination's own coordinate, not the top of its page**:
`[](#fig:halves)` lands 678 pt down page 4, measured, and the showcase's four
cross-page references all point from page 5 back into page 4 and land between
132 and 678 pt down it — four destinations a page-top implementation would
collapse onto one.

**The bundle carries the two classes and none of their scaffolding**, which lives
in `pdf_viewer.css` and is not vendored. So the app defines `--scale-round-x` and
`--scale-round-y` as `1px` on the page, writes `--total-scale-factor` from
`size()`, and brings rules for what `pdf.js` styles nowhere. A text span is given
percentage `left` and `top`, `--font-height` in *unscaled* px, a `font-family`
and — where they apply — `--scale-x` and `--rotate`; it is given **no `position`
and no `font-size`**. An annotation is a `<section>` with a percentage box, a
`z-index` and no `position`, and the `<a>` inside it gets no box at all. Left
undefined the layers take no size and the text paints raw over the raster; left
unstyled a link has nothing to click. The glyphs a reader sees are the raster's,
so the layer's own text is transparent, and a span's rotation is applied before
its `--scale-x`, that being a correction along the text's own direction.

**Only a render that drew advances `drawnRevision`.** `openPdf` answers
whether it did, and returns without drawing when a newer render supersedes it or
the pane measures no width. Recording the revision regardless would tell the next
signal that a page nobody drew is already on screen, after which `refresh`
returns at its own guard forever — a blank pane with no error and no way back.

**A re-render moves the reader only where it should, and the cause decides
where.** A compile that took a text from disk opens at page 1; a compile after
the author's own edit opens at the caret's page; a re-render caused by geometry
alone restores where the reader already was, which the blob frame could never do
because WebKit leaked no position to restore.

`refresh` asks for the status first, and for the bytes only when the state is
`current` **and** the status's `revision` is one it has not drawn — so the page
never draws a page it has been told is out of date, and never redraws one the
reader has scrolled for a signal that compiled nothing, which the app's own save
now is. It re-reads the document's text on the `reloaded` count and on nothing
else, so a fetch cannot race a keystroke still in flight.

**It captures whether it took a reload *before* it advances `takenReload`**, and
hands `openPdf` no page on that pass. This is load-bearing rather than tidy:
assigning a textarea's `value` moves its caret to the end of the control, so a
pass that replaced the text would read the document's last line and open it on
its last page. An open, an external reload over a clean buffer and a Finder
launch therefore all still draw page 1. The rule is "took no reload", not "the
author typed" — an external *figure* change carries a fragment though no
keystroke caused it, which is left as it falls, since the caret is still where
the author put it.

`caretPage` is the lookup: the caret's line is the newlines before
`text.selectionStart`, and the target is the last anchor at or below it, or
nothing when there is none. **Counting newlines is not parsing markdown** — the
page owns no knowledge of the dialect — and it is the one quantity both sides
agree on, `selectionStart` being a UTF-16 offset where a line in Rust is counted
from bytes. A caret above the first heading and a document with no headings take
the same branch and ask for no page, which is the page-1 case above. The precision is the document's heading density, and it follows the
*author* rather than the reader.

**The list it walks holds only the headings written in the file the pane holds**,
and `document::Pane` is the one comparison `render_with` drops the rest by. A
line means something in exactly one buffer, so an anchor from another file is not
a worse match but a number about a document the pane is not showing — and this
lookup breaks at the first anchor past the caret rather than searching for a best
one. **Three arms and not an `Option`**: `Master` keeps the headings carrying no
file, `Beside` those the master names by that path, and `Away` none — a file the
master's directory does not reach, which as an absence would arrive as `Master`
and take the master's line numbers. A pure manifest in the pane yields none and
opens at page 1, the no-heading case above; a master carrying a preface syncs on
its own headings, and **a section in the pane syncs on that section's**, which
the caret could never do while the pane held only what compiles.

`report` places the status: the line in the header, the message in a bar above
the pane, the divergence in a bar of its own, and the dimming a stale page wears
— with no page under it the message takes the whole pane instead. **Every word it
places was chosen in Rust**, so the page composes none of it and the four states
are checked by tests rather than by eye. It survives a failure, because an author
mid-edit passes
through broken states constantly and blanking the pane would lose their place.
`fail` is for the refusals that are not a compile status — an open that will not
read, an export the pane cannot serve — and the next status replaces what it
wrote.

## The header

A bar along the top: 27px and its own 1px rule, so 28px out of `main`. **Its element
children, in order — `#open`, `#save`, `#status` — are two actions and a status line,
and nothing that decides what the window shows**, which is the rule the two bars are
split on: **the footer sets and the header acts**. A setting is left where a reader
likes it and stops being noticed, so the fold, the gutter and the fit are the footer's
one copy each; an action wants the bar the eye starts at. **The floppy is `Save as…`,
one button and not two**: a button earns its place by being what a reader cannot guess,
and `⌘S` — still Save, still the menu's — is the chord every author already has. *Save a
Copy…* is neither and
is argued on its own — Open and Save act on the document being edited, the file `⌘S` writes
and the title bar names, where the export writes a **derived artifact** to a path picked
once — so it is the `File` menu's alone, `app/src/main.rs:export`, the item and its
`Shift+CmdOrCtrl+S` all standing with no button beside them.

**`Save as…` writes wherever it is pointed and the pane follows it only inside the
project.** `document::save_file` confines nothing; `Preview::save_as` asks
`document::confined` and `document::spell` *after* the write and moves `edited` only
where both answer, so a save outside the root is a **copy** — the file is written, the
pane keeps what it was holding, and nothing recompiles and nothing re-arms. **Both saves
answer a receipt** the footer shows and drops, which is what says which of the two just
happened; `#receipt` below is where it lands.

**Two marks, drawn rather than borrowed**, the footer's rule rather than a taste
restated: no glyph means *open a folder* or *write this out*. A folder, and a floppy of
**two internal shapes and not three**, the shutter dropped as busy at 14px. **14px
against the footer's 12**, these being the window's two primary actions and not two
toggles beside a file name, in a 22px button leaving 2.5px of air; every coordinate on a
half pixel; `currentColor`, a 1px stroke, `title` and `aria-label` saying `Open…` and
`Save` — the footer's vocabulary unchanged. **The hover
ground is the affordance, and the footer needs none**: its marks are toggles carrying
their state in their ink, these two are actions with none to carry, so `var(--edge)`
comes up behind them at a 3px radius and the ink goes `--quiet` to `--ink`. **`--edge`
and not a new token** — `--chrome`'s contrast neighbour in both palettes.

`#status` sits at the end on an auto margin, four states composed in Rust and placed by
`report`, wearing the bar's own 11px rather than the body's 13px. **Nothing reads the
header's element order, its 12px shared left column or its 22px button** — clause 1 asserts
`body`'s children, clause 3 the bar's box, and a person the marks.

## The footer

A bar along the foot of the window: 23px and its own 1px rule, so 24px taken out of
`main`. **Five cells and one rule**, counted as this file counts them — the
footer's own element children, in order: `#views` (holding `#views-files`, then
`#views-lines`), `#edited`, `#receipt`, `#controls` (holding `#fit-footer`, then
`#theme`), `#sep-brand`, `#brand`. **Three of those placements are load-bearing rather
than aesthetic**: the fit select sits *inside* `#controls`, because the auto margin
that right-aligns the group is the group's; `.sep` sits *outside* the group
it follows, for the same reason; and `#receipt` sits *before* `#controls`, so the
auto margin still belongs to the group and the sentence reads as a second line about
the file named beside it. **A second rule stood between `#views` and
`#edited` and was withdrawn**, the bar's own gap being enough to part two marks
from a file name; it was `aria-hidden` and read by no clause and neither rig,
where `#sep-brand` is what clause 9 measures two gaps across. It is `main`'s
next element sibling and the last element before the module script — **not `body`'s
last element child**, and those are two facts rather than one restated. By the file
`body` ends `FOOTER, SCRIPT`; **at runtime it ends `FOOTER, SCRIPT, CANVAS`**, `pdf.js`
appending a hidden zero-size canvas of its own to measure text with. So the last
element child is the script by source order and that canvas by the DOM, and the footer
is neither.

**The left cell names what the pane is holding**: `Status::edited` and not
`Status::main`, which from the first click on a panel row is not the file that
compiles — **and a figure when there is one**. A click on an image row puts a surface
over the text and never moves `edited`, and must not: `edited` is the file being typed
in, the one a save writes and the one `main.rs:set_edited` hands to `window.set_title`,
and a figure is none of those. So the page holds `figureInPane` beside the viewer's own
state and **`namePaneFile` is the single writer of the cell**, called from three places
— the status, a figure opening, a figure closing — two sources taking turns written
from two places showing whichever wrote last rather than what is on screen. The value
is `(figureInPane ?? editedPath)` reduced to its bare name, `report` keeping
`editedPath` current beside `parts(state)`; the rule above holds, so this is a cell of
its own, and the empty state collapses it to nothing while the brand stays. **Both
surfaces count as held, including the one that draws nothing** — a `.pdf` row gets
`saySoInstead`'s sentence rather than a picture, and the pane holds that file just as
much. **Page state, for `folded`'s reason**: it decides nothing but its own drawing,
and `hideAsset` is the one exit all three ways back go through. **Nothing marks that
`edited` may differ from `main`** — `rules/desktop-panel.md` draws that
distinction.

**Three of the bar's settings have one control each, and the bar is where a setting
goes**: the footer sets and the header acts. The fit has `fitControls` and `showFit`,
the fold `foldControls`, `showFold` and `offerFold`, the gutter `lineControls`,
`shown` and `showLines`. Each was
briefly two — the header carried a copy while the bar's shape was being looked at — and
the shape of that survives: the listener is installed on every control rather than
against an id, which is what made withdrawing the second copy one line.

**Two of the three now have a second way in, and it is not a second control.** The
`View` menu's `Files` and `Lines` items emit `view-files` and `view-lines`, and the page
takes both through `toggleFold` and `toggleLines` — the same functions the bar's own
presses take, which is what keeps the item and the button one code path. **The state left
the control to make that possible**: the gutter's press used to be read off *the control
that was pressed*, which an event with no control cannot answer, so `shown` sits beside
`folded` as page state and `showLines` writes it onto every control, onto `#lines.hidden`,
and then calls `relines()` and `markLine()`. **All four statements are load-bearing.**
Hiding early-returns out of `relines`, so `markLine` alone clears the caret band off
`text.style.backgroundImage`; showing needs `relines` to build the rows at all, `settle()`
being 200 ms away. **The two items carry no checkmark**, so the marks in the bar remain
the only things that wear either state, and `toggleFold` alone declines: with no document
open `offerFold` has hidden the button and `files.hidden` is what the listener asks.

**The two view toggles are drawn marks where the appearance toggle stays a glyph, and
that is one rule rather than an inconsistency: a mark a glyph names is a glyph, a mark
no glyph names is drawn.** No glyph means *a panel*, and the two that come closest —
`▥` and `☰` — differ only by their fill at 10px. So a pane with a column down its left
and lines with their numbers beside them: two shapes from different families, which is
what makes them one glance apart. **Every coordinate sits on a half pixel** so a 1px
stroke lands on a pixel rather than across two, which a 12px mark at `devicePixelRatio`
1 needs — and that set the gutter mark's spacing rather than the other way round, three
rules evenly about a 12-box's centre putting the middle one on a whole pixel, so they
are `1.5 / 5.5 / 9.5`. `currentColor` and nothing else, so the state is one
declaration: **on is `--ink`, off is `--quiet`, for both**, and neither of the header's
devices — a turning chevron, a filled background — is carried across, neither surviving
at 10px in a bar whose entire ink is one colour. **A bare mark names nothing to a
screen reader**, so each carries `title` and `aria-label` both, `wearAppearance`'s
rule, and the word each says is the one its mark draws: `Files` and `Lines`.
`app/driver/drive.mjs` read them 12x12 in the shipping
WKWebView, each in its two inks.

**The rule has since decided a third case, and the window draws five marks in
three paint rules.** The panel row's delete is the fifth: no glyph names *move
this to the Trash* — `🗑` is emoji-presentation on macOS and ignores `color`,
which is the one thing that button must obey — so it is drawn too, a lidded
basket at the footer's 12px on the footer's half-pixel grid, **and with no
ribs**, two 1px ribs inside a 12px basket merging with the body outline exactly
as `▥` and `☰` do above. It is what made the two rules three: its body is a
trapezoid whose corners are joins in a single path, so it takes the *header's*
set with `stroke-linejoin`, where these two make their only corners out of a
`<rect rx="1.5">` that `rx` rounds. Consolidating the three was declined rather
than missed.

**It duplicates the window title**, which `main.rs:set_edited` sets to the same
`document::title`. What the second placement buys is that the title is native chrome
outside the content area, dimmed when the window is not frontmost and sitting above
the header rather than at the pane's foot. **Bare names collide** and the bar tolerates
it: a project holding `sections/notes.md` and `drafts/notes.md` shows `notes.md` for
either, which is the hazard the panel answers by carrying the whole path.

`#brand` is the literal `Letur`, and
`preview.rs:the_brand_cell_says_exactly_the_bundles_product_name` holds it to
`tauri.conf.json`'s `productName` by `include_str!` on both. **It is the one name in
this app a rename could leave wrong in silence**, the page being outside every other
suite here.

**`#receipt` is what the last save did, for four seconds, and it is usually not
there.** Both sentences are `app/src/preview.rs:Session`'s — `SAVED` is the bare word
`⌘S` gets, since the file is the one the cell to its left already names, and
`Session::save_as` composes `saved as <name> in <folder>` with the folder absolute and
spelled as landed. They ride the two commands' **return** and not `Status`, a receipt
being an event rather than state: a field would re-arrive on every render, need
clearing, and put a near-always-null property in the page's typedef block. So the page
owns the timer and nothing else, and it is **re-armed rather than stacked** — a second
save replaces the sentence and restarts the clock, where a timer per save would let the
first blank what the second had just written.

**`#receipt:empty { display: none }` is what makes it cost nothing while it is
silent**, and it is not tidiness: the bar sets a `gap`, so an empty `span` is still a
flex item and still takes its gaps. Everywhere but the floor `#controls`' auto margin
absorbs them; at 240px there is no free space left and they would come out of `#edited`
instead — the width the sweep reads and the `flex-min` mutation is measured at.
`checks.mjs` owns the cell as a clause, driving the page's own `save` listener and
reading the cell's **emptiness** afterwards rather than the timer's length; `Save as…`
is undrivable there, the stub's `dialog.save` answering `null`, so the window gate reads
that half.

**`#controls` holds the fit select and then `#theme`: the appearance the author
chose.** It was the bar's first interactive control and is now its third, the two view
toggles standing to the left of the cell. **Two positions and three values, which is not a
mismatch**: it gives `light` or `dark`, marked `☀ ☾`, and `system` is what the app
holds before the button has ever been pressed — the window following the machine, at
sunset included, until the first press pins it. **There is no way back to following**
short of deleting `settings.json`, which `specs/desktop_app_spec.md` Phase 13's dated
note records as overruled knowingly rather than unnoticed, and OQ-13 carries.
**So the mark is the appearance in effect and never the one on offer**: in the unset
state it is read off `matchMedia('(prefers-color-scheme: dark)')` and re-read when that
changes, or the bar would show yesterday's answer at sunset; and the click asks for the
other of the two read off *that* rather than off the stored value, so the one press a
person expects from the unset state on a dark machine gives light.
**The page places and never decides**: the value is
`Status::appearance`, `report` wears it beside the cell it writes, and the click's
only act is `invoke('set_appearance')`, so the attribute moves when Rust answers and
not when the button is pressed. `checks.mjs` owns that as a clause — pressing from
all three values, the `system` press being what separates *the other of the two* from
*always dark* — and the `theme-click-direct` mutation, the click rewired to set
`data-theme` itself, is what makes it a claim rather than a hope, since nothing read
off the DOM alone can tell the two apart. The button carries `title` and `aria-label`,
which is more than the bar had; `<footer>` is still an unlabelled `contentinfo`
landmark carrying **four** interactive controls now, two of them named by nothing
else, and `specs/desktop_app_spec.md` OQ-11 carries that. **The glyphs are
measured rather than chosen now**: 9.23 and 9.22px in the shipping WKWebView, the same
figures Playwright's two engines gave, with `#brand` unmoved as they swap — which is
OQ-12 resolved against inline SVG.

**The palette is four token blocks where it was two, and the dark values are written
twice.** The duplication is the pattern: a media query's condition cannot be shared
with an attribute selector, so winning in *both* directions takes both —
`:root:not([data-theme='light'])` inside `@media (prefers-color-scheme: dark)` to opt
out, and `:root[data-theme='dark']` to opt in. **No attribute at all is the third
state** and it is the page's own earlier behaviour unchanged, which is why `system`
removes the attribute rather than setting a third value. `color-scheme` follows all
three ways and is not decoration: it paints the `#fit-footer` select, its native arrow
and the scrollbars. **`--paper` is in none of them** — the page Typst compiles is white in
either palette — and `checks.mjs` pins it unmoved in all six system-by-state
readings, which is what keeps `specs/desktop_app_spec.md` §1.1's narrowing honest:
this app themes its own chrome and nothing about the document.

**The auto margin is `#controls`' and the brand comes last.** It moved off `#brand`
in Phase 13 so the shape is readouts left, icon group right, product name last; left
where it was, the toggle would have sat outside the group's own right edge. **`#brand`'s
own rect cannot tell the two layouts apart** — an auto margin absorbs exactly the free
space in total, so a last child with no right margin does not move, which falsified
the first draft of the clause meant to catch this. What separates them is the group's
distance to the brand — **two gaps and not one**, `#sep-brand` standing between them,
each equal to the footer's own `gap`. **The second is the one the layouts differ in**:
with the auto margin on the group, the separator and the brand are packed at the right;
moved to the brand, the free space opens *between* them, while the first gap reads the
bar's own under either, so a clause measuring only it would falsify nothing. And **only
at a wide viewport**: at
240px a 58-character name has filled `#edited` and left no free space to absorb, so
both layouts read both gaps. The button takes its font, padding, border
and background off and pins `line-height: 1`, so the bar's height does not move.

**What keeps the brand in the bar is `#edited`'s zero automatic minimum size**, and
`min-width: 0` and `overflow: hidden` **each supply it independently** — so the pair is
redundant with each other: dropping either alone changes nothing, and dropping both pushes
the brand off the bar — **measured**, at a 240px viewport with a 58-character name,
identically in Chromium and WebKit, and held now by `app/harness/checks.mjs`: its width
sweep, and the `flex-min` mutation that falsifies it. The literals are name-dependent and
stay where they were taken, in `specs/desktop_app_spec.md` Phase 12. `flex: none` on
`#brand` was not shown to matter at any width tested. As the cell stands the footer holds
one line and keeps the brand down to a 240px viewport. **The header holds its line a
different way and neither bar grows**: it pins `height: 27px`, so what a wrap can do
there is not get taller but push a child out of the box — the half `header-wraps`
falsifies and clause 3 reads. It did grow while it carried seven children, below the
627px they derived, and `rules/desktop-geometry.md` keeps those heights as the
measurement that established its own rule rather than as a live reading.

**The three boxes sum to `innerHeight` exactly**, which is how the bar's cost is
checked. `rules/desktop-geometry.md` has the reading that sum must be taken by, and
why the obvious one is wrong. The footer is a box in flow in the column the observer
above watches and does not make it loop — measured in both engines, four widths in
the window and sixty per-frame changes in the harness, zero of that error in either.

## The panel

**It has a rule file of its own**: `rules/desktop-panel.md`. The column outgrew a
section here — it is now the entries and their order, two kinds of fold, four
gestures across two kinds of row, a create that is on no row and a figure
surface over the text pane — and this file is about the two panes and the bars.
What stays here are the two places the panel reaches into this file: the
`:focus-visible` rule in the gutter section below, which is the file's only one
and draws a keyboard-focused row control at all, and the harness section at the
foot, whose count includes every clause that reads a panel row.

## The gutter

**`Lines` is a view and changes not one byte of the buffer.** Off, the pane is
coloured but unnumbered — the ink is ungated and the gutter is what `⌘L` adds.
On, it gains numbered rows and marks the caret's line **twice**, the row's number
in the gutter and a band behind the line itself, and both marks are classes on a
row that one index puts on and takes off.

**`#text` draws no focus ring, and those two marks are why.** The engine's default
is `outline-style: auto`, 3px in the shipping family, painted whenever the textarea
has focus — nearly the whole time the app is open, and a border on the one pane
whose design says the divider is the only chrome between the two. A focused text
field already shows a caret, the platform's own indicator and the only one saying
*where* in the text the focus is; with `Lines` on, the two marks above say it twice
more. The ring is a third mark of a fact already marked twice and the only one
nobody chose, so `#text:focus` sets `outline: none`. **Scoped to that one
control**, the only one in the window carrying a caret: every other, the row
buttons included, keeps the engine's ring. `:focus` rather than `:focus-visible`
costs a keyboard reader nothing here, measured — a mouse-clicked textarea matches
`:focus-visible` in both engines, text-entry controls always do, while `#save`,
focused by script, does not. The file's one `:focus-visible` rule is a
**visibility** rule and not an indicator:
`#files li .controls button:focus-visible` sets `visibility: visible` so a
keyboard-focused row control is drawn at all.
The textarea stays the editor either way, so nothing that assumes one — the
buffer sent to `edit`, the reload that replaces `value`, `caretPage`, the
disabled state, the divider — is forked.

**A row is as tall as its line renders, not one line-height.** A textarea
soft-wraps and will not say where, so the heights are swept off `#mirror` — the
same text, the same font, the same content width, one element per logical line,
and no longer hidden, `## The ink` above being what it became. What the browser
did to it is what it did to the textarea. **An empty logical line is mirrored as
a zero-width space**, an empty line still occupying one where an empty element
does not, and the failure without it **accumulates**: the offsets are a running
sum, so every blank line above a row shifts it by one line more. The rebuild
happens when the text or the width changed and is skipped when neither did, and
the width-change half rides `settle`'s 200 ms timer rather than a `pointermove`,
because the measurement is a layout.

**The gutter does not scroll; it follows** — its `scrollTop` driven from the
textarea's, on that element's own `scroll` event and again whenever the rows are
rebuilt. Without it the numbers part company with their lines the moment a
document is taller than the pane, which is every document.

**The caret's line is counted the way `caretPage` counts it**, off
`selectionStart` — a second way of finding it would be a second thing that can
disagree with the page the pane opens on. **The band behind that line is a class
on the mirror's own row**, `#lines div.here`'s twin and only a background, the
row's own ink being whatever its tokens made it. It was the textarea's own
`background-image` until the ink layer could carry it: one colour sized and
placed in pixels from the measured row, with `background-attachment: local` to
scroll it with the text and `background-repeat: no-repeat` without which the
one-row gradient tiled down the whole pane and cost a build. The argument against
an element was that it would mean wrapping the textarea and layering over it —
two of which are now the arrangement, the third avoided by absolute positioning,
so the reason is spent and the gradient is gone. **`unmark()` is the one clear**,
called by `markLine`'s guard and by each rebuild, taking the class off **both**
columns at the one index and forgetting it: removing it from a row a pass just
created is a no-op, and removing it from the column that pass did not rebuild is
what stops a mark being stranded.

**An emptied pane loses its band with its rows.** `clear` empties the buffer and
calls `relines`, which returns early there — and the two other things that move
the mark both need a focused, enabled textarea, which a cleared pane is not. So
that early return clears the mark on its way out; without it the previous
document's band stayed painted across an empty, disabled pane until something
opened.

## The ink

**`#mirror` is one element and it is two things**, which is the decision the
layer rests on: the layout the gutter's heights are measured from, and the glyphs
the reader sees. A second element would be two layouts that must agree on every
wrap, that nothing forces to agree, and whose disagreement is invisible until a
paragraph is long enough. It sits on `#text`'s own box — same `padding: 12px
14px`, `left` from `placeInk`, `width` from `remirror` as `text.clientWidth`,
`top` and `bottom` against a `main` the textarea also stretches to — at
`z-index: 1` under a textarea at 2 and the figure surface at 3; it takes neither
the selection nor the pointer, both the textarea's; and it does not scroll, it
follows, on `#lines`' arrangement.

**`relines` is `remirror()` then `regutter()`**, and the split is the founding
fact rather than tidiness: `relines` used to open `if (lines.hidden) return`, so
in the default state the mirror was never built and ink drawn from it would have
coloured nothing until `⌘L`. The mirror is unconditional; the gutter keeps the
gate, and **the heights sweep and `rows` are the gutter's**, so the mode that
pays for the measurement is still the mode that gets it. **Two memo pairs and not
one**, a single pair being marked current by a mirror built while the gutter was
hidden, after which `showLines()`'s rebuild never runs.

**Nothing in `remirror` reads layout**, and that is measured rather than tidy:
`#text`'s box is read by the `ResizeObserver` that watches it, in **two** numbers
— a content width for the memo and a padding-box one for the assignment, caching
only the first leaving the second paying the reflow the first was moved to avoid
— and the rebuild's scroll follow rides a `requestAnimationFrame`, the whole
statement in it, a `text.scrollTop` read being as forced as the write. On a
300,527-byte document, median of twenty appended keystrokes: **36.2 ms in
Chromium and 26 in WebKit before, 1.2 and 2 after**, against a budget of 8. The
`scroll` listener is **not** deferred, having a reader dragging a scrollbar; and
`Lines` mode still costs a whole-document sweep, which is Phase 8's and is OQ-16.

**The ink is suppressed for a drag and for any stale width.** `placeInk` hides
`#mirror` and arms `settle()` whenever the content width is one the mirror was
not built at — the arming being what covers the route with no press and no
`#pages` resize behind it, a scrollbar appearing inside `#text` narrowing its
content box without moving its border box. `dragging` is the second condition on
the restore, so a settle armed by the `pages` observer just before a press cannot
put the ink back detached. `visibility: hidden` and never `display: none`, or a
`scrollHeight` read near a drag answers 0; and `#text` takes its own colour back
for the drag, so the pane goes plain rather than empty.

**Three grammars, one pass, chosen off the extension of the file in the pane**,
folded to lower case as `app/src/document.rs:kind_of` folds it — so `.yml` and
`.yaml` are bibliographies and the YAML lexer is written once and used twice, for
those and for a markdown frontmatter block. The markdown pass carries block state
and cannot be per-line: a fence opened on line 40 makes line 41 code. A line with
no runs gets one text node and no element, which is what holds the budget on a
document that is mostly prose. **Four inks and no fifth**, `--mark` the one token
this added — `#1d5f57` light and `#5fbdad` dark, teal because `--ink` is a navy
and `--alarm` a rust red and those neighbourhoods are spoken for.

| | `--ink` | `--mark` | `--quiet` | `--alarm` |
|---|---|---|---|---|
| markdown | body text, a link's own text | heading text (**bold**), emphasis, strong (**bold**), include marker | punctuation, fence and code-span bodies, frontmatter delimiters, an inline link's destination | an include naming a file the listing marks `missing` |
| BibTeX | field values | `@type` (**bold**) and field names | braces, commas, `=`, comments | — |
| YAML | scalar values | keys (**bold**), `-` item markers | `#` comments, `---`, quotes | — |

**What may not appear in that layer is the point of it.** `font-size`,
`letter-spacing`, `font-family` and `font-variant` all move a glyph's advance and
the textarea does not move with them, so the ink would slide off the words and
the numbers off the lines. Bold is free — a monospace family holds its advance
across weights, 0.000px in both engines. **Italic is forbidden outright** rather
than allowed on runs a table clears: whether a glyph resolves inside the stack or
falls out to a proportional face carrying a real oblique depends on the machine's
installed faces and is not answerable from JS. **A heading may be bold and
coloured and may not be bigger.**

**An include marker is the whole of its own line at zero indentation**, empty
brackets, no title, `.md` case-insensitively; inline it is an inert link, and the
two draw differently. Its target resolves against **`Status.main`'s own
directory** before being compared to `Entry.path`, which is root-relative where a
section path is relative to the master. `core/src/emit.rs:lone_markdown_link`
is stricter still — it wants a top-level paragraph — and **where the two disagree
the compiler is right and the colour is wrong**. A target the listing has never
heard of draws as an ordinary marker rather than reddened: the panel moves at a
compile and not at a keystroke, and reddening would flash every path an author is
halfway through typing. **The highlight is a hint and never a claim**: it gates
nothing, refuses nothing, writes nothing and reaches no Rust. A known rather than
a fix: `--quiet` reads 4.0:1 over the caret's `--band` against 4.4 over
`--ground`, under the line either way, which is `--quiet`'s to answer.

## What checks it, and what that does not reach

**`app/typecheck.mjs` type-checks this file.** It mirrors the page — every line
outside `<script type="module">` blanked, and the one relative `pdfjs` specifier
rewritten to the bare name, which `app/tsconfig.json`'s `paths` answers with the
vendored declarations and a relative specifier cannot be given types for — and
runs TypeScript **5.9.3, pinned**, over the result. Both rules are
line-preserving and the script asserts it, so every diagnostic cites a real
`app/dist/index.html` line. It runs in CI on a push touching the page, the
settings, the declarations or the script, and **never in `cargo test`**: bun is
not a prerequisite of this workspace, and the Rust suite does not acquire one to
check a file it otherwise never opens.

**`strictNullChecks` and `noImplicitAny` are off, and that was measured rather
than assumed.** With them on the shipped page reports **243** errors and not one
is a defect: 112 nullable `getElementById` results, 78 implicit `any`, and the
rest the page's own expando properties. With them off it reported **41**, and
every one of those is an annotation the file now carries — the typedef block at
the top of its script, the DOM subtypes `getElementById` cannot know, `doc` and
`loading`, and the two `AnnotationLayer` call sites whose declarations require
fields `pdf.js` tolerates the absence of at runtime.

**The page and `Status` are held to each other from both sides.** The typedef
block declares `Status`'s twelve fields and `Anchor`'s two; the type check binds
the page's reads to those, and `app/src/preview.rs`'s
`the_page_typedefs_name_exactly_the_fields_status_serializes` compares the same
`@property` lists against a serialized `Status` carrying one `Anchor` — which it
must carry, since an empty list puts no `Anchor` in the JSON at all. **Two
declarations compared with each other**, rather than usage compared against a
declaration. A field renamed on either side alone fails one of the two.

**`app/harness/` drives this page rather than reading it, and it needs the engine's CLI
on `PATH`.** Every document it serves is compiled by a `md2pdf` binary, installed with
`cargo install --locked md2pdf-cli` since `mpdf-011` Phase 3 put the crate on the
registry — a third prerequisite beside `bun` and Playwright's Chromium, stated in
`serve.mjs`'s header and in the README. **`--locked` and not the plain line the engine's
README gives a reader**: this rig compiles documents a gate then compares, so it wants
the resolve the published lockfile pins rather than whatever a fresh one picks up. It was
`cargo run -p md2pdf-cli` until Phase 1, which reaches workspace *members* only: the
engine left with the split and `app` is this workspace's one member, so the old spelling
would die at its own "the CLI would not compile" before a clause ran. `serve.mjs` copies
the page into a gitignored scratch directory — never edits it, `typecheck.mjs` dying on a page with two module
scripts — injects `stub.mjs` into the `<head>`, where it must run before the page reads
`window['__TAURI__']` at module top level, and serves real compiled bytes for `current_pdf`
**and the project's own images for `asset_bytes`**, copied into the scratch tree rather
than served from the source tree, for that server's one rule that nothing outside it is
served however the URL is spelled, and answered on `current_pdf`'s exact route, Rust
returning a `tauri::ipc::Response` for both. **Until it did, the drawn-figure path had
never been exercised by anything**: the stub threw on `asset_bytes`, a refusal reaches
the same surface through `saySoInstead`, so every image row in the rig landed on a
sentence — found by a mutation that falsified nothing, not by reading.
**The stub rejects what the app forbids as well as answering what it returns**, `setSize` in
`core:default`'s own words — the half a stub that merely omits it never tests. It also keeps
**a log of every command the page sends**, because a page that did a thing itself and a page
that asked Rust to are indistinguishable from the DOM. `checks.mjs`
asserts twenty-three clauses in Playwright's Chromium and WebKit, **both of which must pass, every
clause a property and none a metric literal**, and is falsified first against twenty broken copies
of the page, each failing exactly the clause it owns. **Two mutations may own one clause
without either being redundant**, and clause 3 is where that now stands: `flex-min` reaches
the footer's half, the brand pushed out of a bar that holds one line, and `header-wraps`
the header's, a pinned box whose children have left it. Clause 10 was the other such pair;
`views-one-way` went when the header gave its copies up, a copy that cannot disagree being
no copy, and `marks-unlit` owns that clause alone. The error clause stays last and its
number moves as clauses are added, being the only one that accumulates across the others.
**The rule also decides what a clause may read**, which clause 18 is the record of: it
asserts a bibliography row's click through `invoke('status')` and *notes* `#edited`
rather than asserting it, the cell following `edited` being clause 5's own property and
`cell-main`'s to break — a second clause reading it made that mutation fail two and stop
isolating, and the falsification found it where no reading of the file had.
**One clause drives two colour schemes**, `light` being written down as the default rather
than inherited from Playwright, because the palette has to win in both directions and a
suite run in one of them would miss half of it.

**`app/driver/drive.mjs` is a second rig, and the two are not interchangeable.** It launches
`target/debug/letur` from a `--features driven` build and speaks plain HTTP to the WebDriver
server inside it, so what it drives is the **real WKWebView** and the shipped binary; its
mutations go into the live session rather than into a copy, `generate_context!` having walked
this file into the executable, and each must fire or the run failed as an instrument and not as
a clause. **Which to reach for is the distinction worth carrying, and it sorts three kinds of claim
rather than two**: a claim about this page — geometry, the panel, the palette, anything
wanting a broken copy or a second engine — is `app/harness/`'s; a claim about the *window*
— the real IPC, `settings.json` on disk, an OS resize the page is refused — is the
driver's; and **a claim about how the engine that ships *renders*** is the driver's too,
not because it is about the window but because only the driver reaches that engine. The
two drawn marks are that third kind: it reads them at the size their own `svg` attributes
declare, in their two inks, with the footer still the height its own rule declares —
**five clauses and three mutations**, the third `marks-unlit` in this rig's terms, an own
`setAttribute` swallowing the state attribute the ink rule selects on. **The fifth is the
ink's**, and it is the only place the two boxes are compared in the engine that ships:
`#mirror` and `#text` report `scrollHeight` **236612 and 236612** over 1,880 rows, exactly
and not within the tolerance, with `remirror`'s median at **2 ms**. **It fills the pane
rather than opening the document it measures** — `tests/fixtures/` holds several masters,
so an open there climbs to that root and puts one of *them* in the pane, which makes that
document unreachable from this rig as it is from the other, for a different reason. It opens
`tests/fixtures/panel/book.md` in place and writing nothing to do it, a control over a
pane needing the pane. Neither rig reaches more of the seven below than the other.

**What none of it reaches is most of what has gone wrong in this file.** Of the eight defects it
has produced, a type check catches **one** — a `destroy` that `PDFDocumentProxy` does not have,
swallowed by optional chaining, and caught in review before it shipped. The seven it misses are
`ResizeObserver` feedback through a box the callback resized, an `IntersectionObserver`'s
delivery racing an animation frame, a forced pass dropped rather than re-armed, a stale
`deliveryMs`, a settle timer cancelling the render that was about to set the fit, a counter
advanced for a render that never drew, and an early return leaving a caret band on an empty
pane. Every one is behaviour, and **neither the type check nor either rig sees any of them** —
the second half measured: the A/B that justified a browser reaching them, 0 `ResizeObserver`
errors before `overflow-x: auto` and 21 after, was re-run through `serve.mjs --rev` against both
revisions at `devicePixelRatio` 2 and a 520px pane, in both engines headless and headed, and
answered 0 on all eight. `tests/gates/mpdf-009-phase5.js`, pasted into a real window's console,
is still the only thing that has seen them, and `specs/desktop_app_spec.md` OQ-10 carries what
is left.
