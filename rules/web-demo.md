---
title: web-demo
sources:
  - web/index.html
  - app/tests/page_examples_test.rs
covers: >
  the landing page: the one page and the module it no longer loads, the list of
  what the dialect adds and the marked examples that carry it, the byte rule
  those examples obey and the test that enforces it, the CSS that renders a
  script element, the other column generated from the same parse and the
  markers and substitution that carry it, the link every row carries into the
  app, and the two files the page carries down one attribute and the fourth
  reader of them
max_lines: 200
generated: 2026-09-26
---

# Web demo

Letur's landing page, `web/index.html`, published at `https://ivapo.github.io/letur/`:
the argument for the dialect, twelve examples long, each one a snippet the workspace
suite compiles. `mpdf-006` owns the page; `ltr-001` Phase 2 took its panes away and put
Letur itself one link further on, at `app/`, which `rules/web-app.md` covers.

**It came here with `mpdf-011` Phase 1**, which split the engine off into
`Ivapo/md2pdf`. Two things moved with it: the URL above, and the test compiling the
page's claims, `app/tests/page_examples_test.rs`, which was `core/tests/`'s. Every
`core/…` citation below points into the engine's repository deliberately: that is where
the code went, and a reader wants the pointer, not its absence.

## The page, and the module it no longer loads

**One page of text, and it loads no module** (`ltr-001` §2). It was one page doing two
jobs until that phase: the argument, and a textarea and a PDF pane beneath it, which
needed the module and made **9,900,913 bytes** of brotli'd wasm the price of reading about
the dialect. The HTML column was already inlined as bytes (`mpdf-006` Phase 4), so every
row rendered without the module; the module was on the page only to serve the panes, and
with the panes gone it is not requested at all — `web/check.mjs` clause (a) holds that.

**Every row carries a link into the app**: `<a class="open" href="app/#example=NAME">`,
where `NAME` is the row's own `data-example` value. It carries no `data-example`
attribute of its own, so `app/tests/page_examples_test.rs`'s count of twelve still holds.
The link is a navigation and works with scripting off; the header, the lede and every
sentence that promised a compile in this page say so rather than that.

## What the page claims, and the test that holds it to the compiler

The page carries three groups and **twelve examples**: syntax an ordinary renderer passes
through as text (a caption over a table, over a listing and over an image, a `:::` group, a
`{#name}` and the `[](#name)` that points at it, display math), things markdown has no way
to say (the nine frontmatter keys that decide the look, a footnote, a citation and the
reference list it earns), and three refusals — raw HTML, a task marker in a numbered list,
and a LaTeX command off the accepted list. Twenty-eight constructs are supported, so the
page is a chosen few and links out to the README for the rest. The middle refusal was a
task list until `md2pdf-core` 0.4.0 accepted one (`mpdf-006` Phase 5).

**The page's own lede is five behind that**, and the gap is logged rather than half-fixed
here: the page's `<p class="lede">` reads "Twenty-three constructs are supported
and twelve are shown here", which was one behind before `mpdf-005` Phase 10, two behind
after it, three behind after that spec's Phase 11, four at 0.3.0 — which this rule missed,
still saying twenty-six — and five once 0.4.0 took task lists. Correcting the number alone would
leave the page claiming a count it shows no example of, and an example is real work under
this spec's own gate — every claim on the page is a snippet the workspace suite compiles —
so the count, an `::: abstract` row and a `::: keywords` row all belong to a phase of
`mpdf-006` rather than to another spec's close-out. **That the gap has now widened four times is
itself the finding**: a logged gap that keeps growing is a phase of this spec waiting to be
drafted rather than a note to re-write a third time.

**Each example is one element, and four consumers read it.** A
`<script type="text/markdown" data-example="…" data-expect="ok|error">` holds the source;
the reader sees it, `app/tests/page_examples_test.rs` reads it through `include_str!`, the
generated column is made from it, and the app's seed (`web/host/host.mjs`) fetches this
page and opens it as `document.md`. **The page stays the one copy of every example.** A `<script>` holds raw text, so markdown inside one needs no
escaping and a block of a non-JavaScript type is never executed.

**The content is load-bearing bytes: flush left, no leading and no trailing newline.** Not
tidiness: measured at two spaces of indent the frontmatter example stops being frontmatter
and reaches the page as a setext heading over prose, and the caption example keeps its
table but emits `: The measurements.` as literal text — and `md_to_pdf` returns `Ok` for
both. A leading newline is the same hazard one line on, moving every refusal's `at line N`
by one. The rule needs no stripping step in any consumer, so they cannot drift apart by
normalising differently.

`app/tests/page_examples_test.rs` asserts that rule, a count of exactly twelve, a mark of
`ok` or `error` on each, unique names, two asset elements, and message elements matching the
refusals; then that each `ok` example compiles — **each handed the page's two files**, as the
app opens them beside every example — and that each `error` example's `to_string()` equals its
row's visible `<code data-error-for="…">` text, character for character. **The checked
sentence is the one the reader sees** — an attribute copy would prove agreement with a
string nobody reads. The `<code>` scan is weaker than the `<script>` scan, parsed markup
equalling its raw slice only while the sentence needs no character reference, so a separate
assertion refuses a message carrying `<`, `&` or a newline. It does **not** assert the 9/3
split.

**That test is a workspace test over a file outside the workspace, and that is the point.**
`web/Cargo.toml`'s empty `[workspace]` table detaches the directory, so `cargo build
--workspace` and `cargo test --workspace` behave as they did before it existed — reading a
file is not membership.

**A `<script>` is `display: none` in every UA stylesheet**, and the source column of every
row is one. `script[data-example] { display: block; white-space: pre; overflow-x: auto }`
renders it. A visible `<pre>` duplicate was refused deliberately: two copies of an example
can differ, which is the failure the whole arrangement prevents.

## The other column, and where its markup comes from

**The column beside each example is generated, not written.** It is
`core/src/lib.rs:md_to_html` over that row's own source — the same parser through
`core/src/emit.rs:parser`, options and broken-link callback together, written out by
pulldown-cmark's own HTML backend instead of by
the emitter, so it is not a second renderer but the one the page's whole claim is already
about. It shows what this parse looks like when something other than the emitter sets it
down: the caption marker is lost because nothing but the emitter is looking for it. The
twelve labels read `the same parse, as HTML`. **The bytes are inlined rather than produced
at load**, so no column sits behind the module and a reader with scripting off meets both
halves of every row.

**The blocks sit between `<!--html:NAME-->` and `<!--/html:NAME-->`**, keyed to the row's
own `data-example` value. This is the one region in the page that cannot end at a closing
tag — the `raw-html` block ends `</div>`, the `footnote` block carries a
`<div class="footnote-definition">` — and counting opens against closes is an HTML parser
under another name; a comment cannot nest and `push_html` emits none. **What lies between a
pair is exactly what the generator returned**, nothing trimmed at either end, so the page
and the test compare the same bytes by construction; `raw-html` ends without a trailing
newline. A `div.rendered` wrapper sits outside the markers, uncompared, carrying the type
scale four real tables, two real checkboxes, a heading, a listing and a real `<div>` need.

**One substitution, and exactly one**: an image destination equal to the page's
`data-asset` name becomes a `data:` URI over those same bytes, **percent-encoded over an
explicit set — every byte outside ASCII letters, digits and `-._~`**. The set is named
because the reflex is broken and the break is invisible to an equality check —
`pulldown_cmark`'s `escape_href` leaves `#` unencoded and the SVG carries
`stroke="#1e3c82"`, so a raw URI truncates at the fragment and renders nothing while both
sides agree about the broken bytes. Measured 2026-08-22: 509 bytes to a 954-byte URI that
round-trips exactly, the diagram loading at its declared `320×72`.

**The generator is the test** — `app/tests/page_examples_test.rs:generated` produces a
block, `every_generated_block_is_the_parsers_own_html` compares all twelve against the page,
and `bless_the_generated_blocks`, `#[ignore]`d so `cargo test --workspace` skips it, writes
them in; a generator of its own would implement the substitution twice. Three assertions
guard it: the marker counts, that the image is named once across the twelve outputs and
survives nowhere after substitution, and that no block holds `<script`, `data-example="`,
`data-asset="` or `<!--` — the first three handing the file's own scans a phantom element,
the fourth ending the delimiter early. Checked 2026-08-22 in headless Chromium **with
JavaScript disabled**: every column rendered, the diagram visible rather than a
broken-image box, the frontmatter row showing no keys at all.

## The two files, and the ones that stay out

**The page owns two files and the reader owns none**: `samples/pipeline.svg`, and the
bibliography the citation row names. Both are carried inline in `web/index.html`, and
**both ride one attribute**. `data-asset`'s value *is* the path `md2pdf_core::Asset` is
given — as true of a `.yml` as of an `.svg` — so a second attribute would be a second
mechanism for something the page already has one word for. It is one copy of one name
apiece, which the caption row and the citation row must write identically or fail the
suite.

**The `type` is the discriminator, and it was already one.** The image is a
`<script type="image/svg+xml" data-asset="…">` and the bibliography a
`<script type="application/yaml" data-asset="…">` — non-JavaScript types, so neither is
executed and neither needs escaping. `app/tests/page_examples_test.rs` scans for
the pair and the app's seed takes every `data-asset` element under its own name, so
neither depends on document order, and the type is what keeps the `data:` URI
substitution keyed to the image alone. Both obey the
examples' byte rule at the ends — no leading and no trailing newline — but only the
bibliography's *inner* indentation is load-bearing: the SVG's bytes reach Typst's image
loader, which does not read them as structure, where a YAML reader does.

**The app opens both beside every example**, under their `data-asset` names, whatever the
example names: `md_to_pdf` ignores an asset the document never names, and a project that
carried them for some rows only would draw Figure 1 in one example and refuse it in the
next. The bytes are the elements' text, encoded — the bytes the test compiles.

**What the page reads is still these two files and none of the reader's.** Their own
markdown reaches the compiler through the app's Open…; their images and bibliographies do
not until `ltr-001` Phase 3's import — an `![…](their-file.svg)` comes back
`Error::MissingImage` from `core/src/lib.rs:collect`. The page is not an editor: the app
is, and `mpdf-001` §1.1 still refuses servers permanently.
