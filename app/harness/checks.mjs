/* What checks `app/dist/index.html` — the assertions the harness exists for.

     bun app/harness/checks.mjs [--webkit] [--headed] [--rev <sha>] [--doc <path>]
                                [--mutate <name>] [--falsify]

   **Chromium is the default, and the default decides little.** Every metric
   literal is forbidden below and `mpdf-003` Phase 12's gate requires both engines
   to pass, so this is not a fidelity choice and is not dressed as one: against the
   same page the header's rect reads 47.40625 in Playwright's WebKit, 46.75 in its
   Chromium and 47 in the window, and at the narrow widths WebKit agrees with
   Chromium and not with the window. Chromium is first because it is the engine
   every recorded run in `mpdf-009` used and the one a contributor most likely
   already has; WebKit is kept because a second engine catches what one cannot,
   not because it is truer.

   **From which follows the one rule every check obeys: assert a property, never a
   metric literal.** The sum is exact; the footer does not change height; the
   header grows below its own threshold; the cell holds the last path segment. **No
   check may encode 46.5, 46.75, 47, 66, 79, 80.5 or 627**, and
   `rules/desktop-geometry.md` carries why.

   **What this does not reach**, said here rather than left to be discovered: the
   seven behaviour defects in `rules/desktop-panes.md`'s list. The A/B that once
   justified reaching them — 0 `ResizeObserver` errors before `overflow-x: auto`,
   21 after — does not reproduce under this driver. `tests/gates/mpdf-009-phase5.js`,
   pasted into a real window's console, is still the only thing that has seen them.

   **The suite is falsified before it is trusted.** `--mutate <name>` serves a
   deliberately broken copy and judges that **exactly** the clause that owns it
   fails; `--falsify` runs all twenty-three. That is the gate's clause 3, run rather
   than read.

   **`light` is the default colour scheme and it is written down**, because one
   of the clauses below is about a page that must behave differently under each
   and Playwright's own default is not a thing to inherit silently.           */

import { chromium, webkit } from 'playwright'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { serve } from './serve.mjs'

const argv = process.argv.slice(2)
const has = (name) => argv.includes(`--${name}`)
const flag = (name, fallback = null) => {
  const at = argv.indexOf(`--${name}`)
  return at < 0 ? fallback : argv[at + 1]
}

/* Which clause each mutation owns. A mutation that fails a second clause is a
   check measuring something it does not claim to; a mutation that fails none is a
   check that would pass on a page the scope forbids.

   **Two mutations may own one clause without either being redundant**, and
   clause 3 is where that stands: `flex-min` reaches the footer's half — the
   brand pushed out of a bar that holds one line — and `header-wraps` reaches
   the header's, a pinned box whose children have left it. `views-one-way` was
   the other such pair and was withdrawn when the header gave its copies up:
   it reached the sync between a toggle's two copies, and there is one copy. */
const OWNS = {
  'footer-last': 1,
  'flex-min': 3,
  'header-wraps': 3,
  'cell-main': 5,
  'theme-dark-attr': 7,
  'theme-click-direct': 8,
  'controls-auto-margin': 9,
  'marks-unlit': 10,
  'figure-unnamed': 11,
  'save-as-mislabelled': 12,
  'receipt-sticks': 13,
  'divider-selects': 14,
  'views-deaf': 15,
  'trash-unnamed': 16,
  'folds-one-level': 17,
  'bib-opens-nothing': 18,
  'ink-bigger-headings': 19,
  'ink-lines-gated': 20,
  'ink-include-anywhere': 21,
  'ink-band-tiles': 22,
  'ink-anchor-is-a-link': 23,
  'ink-captions-anywhere': 24,
  'ink-math-per-line': 25
}

/* **58 characters, and the length is asserted rather than trusted.** The
   `flex-min` mutation only bites where `#edited`'s content overflows the bar, and
   the default fixture's longest bare name is `missing.md` — so without this name
   and the 240px floor below, that mutation falsifies nothing. **It is deliberately
   not one of the fixture's eleven entries**, so the panel marks no edited row
   during the sweep: that is why the sweep and the row click are two checks. */
const LONG_NAME = 'notes-and-sources-for-the-second-chapter-final-revision.md'

/* Descending, and it must reach 240px — the width the brand was measured to
   survive down to, and the one the mutation was measured at. The widths are
   inputs; nothing below is keyed to any of them. */
const WIDTHS = [900, 620, 500, 320, 240]
const HEIGHT = 600

/* -------------------------------------------------------------- the ledger */

let passed = 0
let failed = 0
const owned = []

const ok = (n, name, good, detail) => {
  good ? passed++ : (failed++, owned.push(n))
  console.log(`${good ? 'PASS' : 'FAIL'}  ${String(n).padStart(2)}. ${name}${detail ? '\n          ' + detail : ''}`)
}
const note = (s) => console.log(`....  ${s}`)

/* ---------------------------------------------------------------- the page */

const settle = async (page) =>
  page.evaluate(
    () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 250))))
  )

/** A page with the document open and its first compile drawn. Every check gets
    its own, so no check inherits the state another left behind. */
const opened = async (browser, url, width = WIDTHS[0], colorScheme = 'light') => {
  const page = await browser.newPage({
    viewport: { width, height: HEIGHT },
    deviceScaleFactor: 2,
    colorScheme
  })
  await page.goto(url, { waitUntil: 'load' })
  await page.waitForFunction(() => typeof window.__harness === 'object')
  await page.evaluate(() => {
    window.__harness.open()
    window.__harness.fire('rendered')
  })
  /* The pane has drawn when a wrapper exists: `openPdf` measured the pane and
     rasterised, so a geometry reading taken after this is taken against a page
     and not an empty column. */
  await page.waitForFunction(() => document.getElementById('pages').children.length > 0, null, {
    timeout: 30000
  })
  await settle(page)
  return page
}

/** Work the gutter, **through the bar's own control and not through the menu
    event**. The two are one code path in the page and either would show the
    gutter — but `views-deaf` drops the two `listen` registrations and owns
    clause 15, so a check that drove the event would fail under that mutation
    too and cost it its isolation. The chord is clause 15's subject; the button
    is every other clause's way in. */
const pressLines = async (page) => {
  await page.evaluate(() => document.getElementById('views-lines').click())
  await settle(page)
}

/** What a page's error listener saw. Read before the page is closed, because the
    listener lives in the page — `page.on('pageerror')` does not see a
    `ResizeObserver` loop error, which is measured and is why `stub.mjs` installs
    one of its own in `<head>`. */
const drainErrors = async (page) => page.evaluate(() => window.__harness.errors)

/* -------------------------------------------------------------- the checks */

/* 1. **All three, and the position is satisfiable by and only by one
      placement.** `body`'s element children are the marks' `SVG`, then
      `HEADER, MAIN, FOOTER, SCRIPT`:
      "after `</main>`" and "the last element of `body`" name two different
      places — the second being that script, and at runtime a hidden canvas
      `pdf.js` appends — so a clause asserting only one of them would pass on a
      page the scope forbids. */
const elementOrder = async (browser, url) => {
  const page = await opened(browser, url)
  const read = await page.evaluate(() => {
    const main = document.querySelector('main')
    const footer = document.querySelector('footer')
    /* The page's own module, not the stub's: the harness injects a second
       `script[type=module]`, and it lives in the head. */
    const module = document.body.querySelector('script[type="module"]')
    return {
      nextIsFooter: main.nextElementSibling === footer,
      thenModule: footer.nextElementSibling === module,
      notLast: document.body.lastElementChild !== footer,
      order: [...document.body.children].map((e) => e.tagName).join(', ')
    }
  })
  const errors = await drainErrors(page)
  await page.close()

  ok(
    1,
    "the footer is main's next sibling and the last element before the module script",
    read.nextIsFooter && read.thenModule && read.notLast,
    `body holds ${read.order}`
  )
  return errors
}

/* 2 and 3 share one sweep. **They are still two clauses**: the sum is what the
   two bars cost the column, and clause 3 is what each bar holds while the
   window narrows — the header the box it declares, the footer its height and
   its brand. `flex-min` moves the second without moving the first, and
   `header-wraps` moves the header's half of the second without moving either
   of the others. */
const sweep = async (browser, url) => {
  if (LONG_NAME.length < 58) throw new Error(`the sweep's name is ${LONG_NAME.length} characters, wanted 58`)

  const page = await opened(browser, url)
  /* **Both fields carry the long name, and that is what keeps this clause off
     clause 5's ground.** This one is about the bar's geometry — that a name too
     long for the cell does not push the brand out — and it must not be keyed to
     *which* `Status` field the cell is wired to, which is the only thing clause 5
     asserts. Setting `edited` alone, the `cell-main` mutation emptied the cell,
     the sweep went vacuous and this clause failed for a reason it does not own.
     Measured, not reasoned about: that is what the falsification run reported
     before this line was written. */
  await page.evaluate((name) => {
    window.__harness.set({ edited: name, main: name })
    window.__harness.fire('rendered')
  }, LONG_NAME)
  await settle(page)

  const readings = []
  for (const width of WIDTHS) {
    await page.setViewportSize({ width, height: HEIGHT })
    await settle(page)
    readings.push(
      await page.evaluate(() => {
        /* `getBoundingClientRect().height` and never `offsetHeight`: the
           header was fractional when this was written and `offsetHeight` rounds
           it, so a three-term sum overshoots `innerHeight` at some widths and
           not at others. `rules/desktop-geometry.md` has which engine's numbers
           those were. */
        const rect = (sel) => document.querySelector(sel).getBoundingClientRect()
        const header = rect('header')
        const main = rect('main')
        const footer = rect('footer')
        const brand = rect('#brand')

        /* **The header's own rule, off the CSSOM, and never `getComputedStyle`
           — and the reason is not the driver's reason.** There, on the footer,
           that call resolves to the *used* height and would grow with the
           content, holding however tall the bar got. Here the header's height
           is pinned, so it never grows: `getComputedStyle` returns `27px` under
           the mutation too, and the reading would not be wrong, it would be
           **vacuous** — and it would go on being vacuous on the day someone
           drops the `height` declaration. The sheet fails loudly instead. Last
           match wins, the harness serving a stub stylesheet of its own. */
        let declared = null
        for (const sheet of document.styleSheets) {
          for (const rule of sheet.cssRules) {
            if (rule.selectorText === 'header' && rule.style.height) declared = parseFloat(rule.style.height)
          }
        }

        const bar = document.querySelector('header')
        return {
          width: innerWidth,
          innerHeight,
          header: header.height,
          declared,
          /* The border is beside the declared height because the page sets no
             global `box-sizing`, so the 1px rule is outside the box — which is
             how `app/driver/drive.mjs` reads the footer's. */
          border: parseFloat(getComputedStyle(bar).borderBottomWidth),
          /* **The half a pin can lie about.** A flex container with an explicit
             `height` does not grow when its line wraps — its content overflows
             — so a clause reading only the height would hold on a page whose
             children had left the bar. */
          outside: [...bar.children].filter((child) => {
            const box = child.getBoundingClientRect()
            return box.top < header.top - 0.5 || box.bottom > header.bottom + 0.5
          }).length,
          main: main.height,
          footer: footer.height,
          sum: header.height + main.height + footer.height,
          inside: brand.left >= footer.left && brand.right <= footer.right,
          brand: document.getElementById('brand').textContent.trim(),
          cell: document.getElementById('edited').textContent
        }
      })
    )
  }
  const errors = await drainErrors(page)
  await page.close()

  const said = (r) =>
    `${r.width}px | header ${r.header} against ${r.declared} + ${r.border}, ${r.outside} children outside | ` +
    `footer ${r.footer} | sum ${r.sum} against ${r.innerHeight} | ` +
    `brand ${r.brand || '(empty)'} ${r.inside ? 'in the bar' : 'OUTSIDE the bar'}`
  for (const r of readings) note(said(r))

  ok(
    2,
    'the three boxes sum to innerHeight at every width, read off getBoundingClientRect',
    readings.every((r) => r.sum === r.innerHeight),
    readings.filter((r) => r.sum !== r.innerHeight).map(said).join('   ') || 'exact at every width'
  )

  /* **The positive control for this sweep is `flex-min`**, and naming it is
     what stops the clause proving the viewport ever narrowed by nothing. That
     mutation bites only at 240px with the 58-character name, so a run in which
     it isolates is a run that reached a narrow viewport with a full cell. It
     used to be `grew` — the header wrapping below its own threshold — and that
     control went when the header stopped wrapping at all. */
  const widest = readings[0]
  if (readings.some((r) => r.declared === null))
    throw new Error('no `header` rule declares a height — the reading this clause takes is not in the sheet')

  const off = (a, b) => Math.abs(a - b) > 0.5
  const pinned = readings.every((r) => !off(r.header, r.declared + r.border))
  const held = readings.every((r) => r.outside === 0)
  const height = readings.every((r) => r.footer === widest.footer)
  const kept = readings.every((r) => r.brand === 'Letur' && r.inside)

  ok(
    3,
    'the header is the box its own rule declares and holds every child inside it, and the footer keeps its height and its brand across a sweep to 240px with a 58-character name',
    pinned && held && height && kept && readings.every((r) => r.cell === LONG_NAME),
    `the header was ${widest.declared} + ${widest.border} everywhere: ${pinned}; ` +
      `no child outside it: ${held}; ` +
      `the footer held ${widest.footer}: ${height}; the brand stayed in the bar: ${kept}; ` +
      `the cell held the long name: ${readings.every((r) => r.cell === LONG_NAME)}`
  )
  return errors
}

/* 4. **What "places and never composes" is assertable as.** The sentence is
      worded in `app/src/preview.rs` out of `state` and `time`; the page places
      it. So: take those two values out of what the line holds, and what is left
      must carry no word of its own — a separator and nothing else. And the line
      must not move when the values the page *could* fold into it do. */
const statusPlaces = async (browser, url) => {
  const page = await opened(browser, url)

  const STATES = [
    { state: 'empty', time: null, error: null, page: false },
    { state: 'current', time: '31 ms', error: null, page: true },
    { state: 'stale', time: '12 ms', error: 'the harness put a sentence here', page: true },
    { state: 'failed', time: null, error: 'the harness put a sentence here', page: false }
  ]

  const read = []
  for (const patch of STATES) {
    read.push(
      await page.evaluate(async (patch) => {
        window.__harness.set(patch)
        window.__harness.fire('rendered')
        await new Promise((r) => setTimeout(r, 150))
        const status = document.getElementById('status')
        return { asked: patch, text: status.textContent, className: status.className }
      }, patch)
    )
  }

  /* The invariance half: the same two values, everything else moved. A page that
     folded a file name or a row count into the line would move here. */
  const before = read[1].text
  const after = await page.evaluate(async () => {
    window.__harness.set({
      state: 'current',
      time: '31 ms',
      edited: 'somewhere/else-entirely.md',
      main: 'another.md',
      entries: [{ path: 'another.md', kind: 'markdown', missing: false }]
    })
    window.__harness.fire('rendered')
    await new Promise((r) => setTimeout(r, 150))
    return document.getElementById('status').textContent
  })

  const errors = await drainErrors(page)
  await page.close()

  const composed = read.filter((r) => {
    const left = r.text.replace(r.asked.state, '').replace(r.asked.time ?? ' ', '')
    return /[\p{L}\p{N}]/u.test(left) || !r.text.startsWith(r.asked.state) || r.className !== r.asked.state
  })
  for (const r of read) note(`${r.asked.state}: ${JSON.stringify(r.text)} class ${JSON.stringify(r.className)}`)

  ok(
    4,
    'the status line carries no value the page chose, in any of the four states',
    composed.length === 0 && after === before,
    composed.length
      ? `composed: ${composed.map((r) => JSON.stringify(r.text)).join(', ')}`
      : `and did not move when edited, main and entries did: ${JSON.stringify(before)}`
  )
  return errors
}

/* 5. **The behaviour a reader could most reasonably expect to be the other
      way.** The bar names the file being typed in, which from this click is not
      the file the page beside it came from. Asserted against `main` as well as
      against `edited`: the two are equal at the open, so a cell wired to `main`
      passes everything until here. */
const cellFollowsThePane = async (browser, url) => {
  const page = await opened(browser, url)

  const atOpen = await page.evaluate(() => ({
    cell: document.getElementById('edited').textContent,
    status: window.__harness.status()
  }))

  const clicked = await page.evaluate(async () => {
    const row = [...document.getElementById('parts').children].find(
      (li) => !li.classList.contains('folder') && li.querySelector('.name')?.textContent === 'text.md'
    )
    const button = row?.querySelector('button.name')
    button?.click()
    await new Promise((r) => setTimeout(r, 500))
    return {
      clicked: !!button,
      cell: document.getElementById('edited').textContent,
      status: window.__harness.status()
    }
  })

  const errors = await drainErrors(page)
  await page.close()

  const last = (path) => (path === null ? '' : path.split('/').pop())
  note(`at the open: cell ${JSON.stringify(atOpen.cell)}, edited ${atOpen.status.edited}, main ${atOpen.status.main}`)
  note(
    `after the click: cell ${JSON.stringify(clicked.cell)}, edited ${clicked.status.edited}, main ${clicked.status.main}`
  )

  ok(
    5,
    'the cell is the last segment of `edited`, and after a row click that is not `main`',
    atOpen.cell === last(atOpen.status.edited) &&
      clicked.clicked &&
      clicked.status.edited !== clicked.status.main &&
      clicked.cell === last(clicked.status.edited) &&
      clicked.cell !== last(clicked.status.main) &&
      !clicked.cell.includes('/'),
    `clicked ${clicked.clicked}; wanted ${JSON.stringify(last(clicked.status.edited))}, ` +
      `got ${JSON.stringify(clicked.cell)}, and main's is ${JSON.stringify(last(clicked.status.main))}`
  )
  return errors
}

/* 6. Rust sends flat entries, already ordered, and never a directory: a heading
      belongs exactly where a path's leading segments differ from the last one's,
      one per newly entered segment, and the depth is how many segments precede
      the name. **The expectation is derived here from the entries** rather than
      read off the page, so this compares two derivations and not a page with
      itself. */
const panelDrawsTheEntries = async (browser, url) => {
  const page = await opened(browser, url)

  const read = await page.evaluate(() => ({
    rows: [...document.getElementById('parts').children].map((li) => ({
      folder: li.classList.contains('folder'),
      name: li.querySelector('.name')?.textContent ?? '',
      depth: li.dataset.depth
    })),
    entries: window.__harness.config.entries
  }))
  const errors = await drainErrors(page)
  await page.close()

  const wanted = []
  let folder = []
  for (const entry of read.entries) {
    const segments = entry.path.split('/')
    const here = segments.slice(0, -1)
    let shared = 0
    while (shared < here.length && here[shared] === folder[shared]) shared++
    for (let at = shared; at < here.length; at++) {
      wanted.push({ folder: true, name: here[at], depth: String(Math.min(at, 5)) })
    }
    folder = here
    wanted.push({ folder: false, name: segments[segments.length - 1], depth: String(Math.min(here.length, 5)) })
  }

  const same = JSON.stringify(read.rows) === JSON.stringify(wanted)
  note(`${read.entries.length} entries drew ${read.rows.length} rows, ${wanted.length} wanted`)

  ok(
    6,
    'the panel draws one row per entry, in order, with the folders derived',
    same,
    same
      ? read.rows.map((r) => `${r.folder ? '[dir] ' : ''}${r.name}@${r.depth}`).join('  ')
      : `got     ${JSON.stringify(read.rows)}\n          wanted  ${JSON.stringify(wanted)}`
  )
  return errors
}


/* 7. **Six readings, and the point is the two that a one-scheme suite would
      miss.** The palette has to win in *both* directions: `dark` chosen under a
      light system and `light` chosen under a dark one. The tokens are one half
      and `color-scheme` the other — it is what paints the `#fit-footer` select,
      its arrow and the scrollbars, so a page whose tokens said dark while it
      said light would put a light scrollbar on a dark pane.

      **And `--paper` is unchanged in all six**, which is the clause that keeps
      `specs/desktop_app_spec.md` §1.1's narrowing honest: this app themes its
      own chrome and nothing about the document, the page Typst compiles being
      white in either palette. Every value is read against the page's own other
      readings — the system's own dark and its own light — so no colour literal
      is written here. */
const paletteTurnsBothWays = async (browser, url) => {
  const read = []
  const errors = { total: 0, loops: 0, spoken: [] }

  for (const system of ['light', 'dark']) {
    const page = await opened(browser, url, WIDTHS[0], system)
    for (const appearance of ['system', 'light', 'dark']) {
      read.push({
        system,
        appearance,
        ...(await page.evaluate(async (appearance) => {
          window.__harness.set({ appearance })
          window.__harness.fire('rendered')
          await new Promise((r) => setTimeout(r, 150))
          const root = document.documentElement
          const style = getComputedStyle(root)
          return {
            attribute: root.getAttribute('data-theme'),
            scheme: style.colorScheme,
            ground: style.getPropertyValue('--ground').trim(),
            ink: style.getPropertyValue('--ink').trim(),
            paper: style.getPropertyValue('--paper').trim()
          }
        }, appearance))
      })
    }
    const seen = await drainErrors(page)
    errors.total += seen.total
    errors.loops += seen.loops
    errors.spoken.push(...seen.spoken)
    await page.close()
  }

  const at = (system, appearance) => read.find((r) => r.system === system && r.appearance === appearance)

  /* The two the page has always had, and every other reading is compared to one
     of them rather than to a literal. */
  const lightGround = at('light', 'light').ground
  const darkGround = at('dark', 'dark').ground
  const wants = (r) => (r.appearance === 'system' ? r.system : r.appearance)

  const wrong = read.filter((r) => {
    const wanted = wants(r)
    const ground = wanted === 'dark' ? darkGround : lightGround
    const attribute = r.appearance === 'system' ? null : r.appearance
    return (
      r.attribute !== attribute ||
      r.ground !== ground ||
      !r.scheme.includes(wanted) ||
      (wanted === 'dark' ? r.scheme === 'light' : r.scheme === 'dark')
    )
  })

  const paper = new Set(read.map((r) => r.paper))
  for (const r of read) {
    note(`system ${r.system} + ${r.appearance}: attr ${r.attribute} scheme ${r.scheme} ground ${r.ground}`)
  }

  ok(
    7,
    'the palette turns both ways, and --paper turns in neither',
    wrong.length === 0 && paper.size === 1 && lightGround !== darkGround,
    wrong.length
      ? `wrong: ${wrong.map((r) => `${r.system}+${r.appearance}`).join(', ')}`
      : `six readings, --paper ${[...paper][0]} in all of them; the two grounds differ: ${lightGround} / ${darkGround}`
  )
  return errors
}

/* 8. **The boundary, which nothing read off the DOM alone can see.** A toggle
      that set the attribute itself would look identical from the page: same
      mark, same palette, same flip. So this asserts both halves — that Rust
      moving the value alone moves the attribute, and that the click's only act
      is to ask, naming the other of the two.

      **Three presses and not one**, because the button has two positions and
      Rust three values: `system` is the unset state, and what a press from it
      must ask for is read off the *system*, not off the value. Under this
      page's light scheme that is `dark` — the same answer a press from `light`
      gives, which is why the `dark` press is here too and is the one that
      separates "the other of the two" from "always dark".

      The stub's answer is what closes the loop, so the asking half is measured
      after it: a page that had already moved the attribute before the answer
      arrived would be deciding. */
const cellPlacesAndDoesNotDecide = async (browser, url) => {
  const page = await opened(browser, url)

  /* Rust moves it, nobody clicks. */
  const placed = await page.evaluate(async () => {
    const seen = []
    for (const appearance of ['dark', 'light', 'system']) {
      window.__harness.set({ appearance })
      window.__harness.fire('rendered')
      await new Promise((r) => setTimeout(r, 150))
      seen.push({ appearance, attribute: document.documentElement.getAttribute('data-theme') })
    }
    return seen
  })

  /* The click, from each of the three, with the boundary read rather than the
     DOM. The page is under the default light scheme, so `system` is light in
     effect and a press from it must ask for `dark`. */
  const clicked = []
  for (const [from, wanted] of [
    ['system', 'dark'],
    ['dark', 'light'],
    ['light', 'dark']
  ]) {
    clicked.push({
      from,
      wanted,
      ...(await page.evaluate(async (from) => {
        window.__harness.set({ appearance: from })
        window.__harness.fire('rendered')
        await new Promise((r) => setTimeout(r, 150))

        window.__harness.forget()
        document.getElementById('theme').click()
        await new Promise((r) => setTimeout(r, 150))

        return {
          asked: window.__harness.invokes().filter((i) => i.name === 'set_appearance'),
          attribute: document.documentElement.getAttribute('data-theme'),
          answered: window.__harness.status().appearance
        }
      }, from))
    })
  }

  const errors = await drainErrors(page)
  await page.close()

  const misplaced = placed.filter((p) => p.attribute !== (p.appearance === 'system' ? null : p.appearance))
  const wrong = clicked.filter(
    (c) =>
      c.asked.length !== 1 ||
      c.asked[0].args.appearance !== c.wanted ||
      c.attribute !== c.wanted ||
      c.answered !== c.wanted
  )

  note(`placed: ${placed.map((p) => `${p.appearance}->${p.attribute}`).join('  ')}`)
  note(
    `clicked: ${clicked.map((c) => `${c.from} asked ${c.asked.map((i) => i.args.appearance).join('+') || 'nothing'}`).join('  ')}`
  )

  ok(
    8,
    'the cell places what Rust says, and the click only asks, for the other of the two',
    misplaced.length === 0 && wrong.length === 0,
    misplaced.length
      ? `Rust moved and the page did not: ${JSON.stringify(misplaced)}`
      : wrong.length
        ? `wrong: ${wrong.map((c) => `from ${c.from} wanted ${c.wanted}, asked ${c.asked.map((i) => i.args.appearance).join('+') || 'nothing'}`).join('; ')}`
        : `three placed with no click; three clicks each asked for the other and the attribute followed the answer`
  )
  return errors
}

/* 9. **Keyed to the group and not to the brand, because the brand cannot move.**
      An auto margin absorbs exactly the free space in total, so a last child
      with no right margin reads the same x under either layout — which is what
      falsified this clause's first draft. What separates them is the distance
      from the group to the brand, and it must equal the bar's own gap.

      **Read off the stylesheet, never written as a number**, per this file's
      one rule; and **taken at the sweep's widest width**, which is part of the
      clause rather than incidental: at 240px the 58-character name has filled
      `#edited` and left no free space for an auto margin to absorb, so both
      layouts read the gap and the clause would falsify nothing.

      **Measured from `#controls`' own right edge, and there are two gaps to
      cross.** The clause's first draft read the theme button's edge, which was
      the group's right edge only while the group held one flush, unpadded
      child; the fit select made that false, and the separator put a third cell
      between the group and the brand. So the reading is the group to the
      separator and the separator to the brand, each one bar-gap.

      **The second of the two is what still separates the layouts.** With the
      auto margin on the group, the separator and the brand are packed at the
      right; moved to the brand, the free space opens between them — while the
      group-to-separator gap reads the bar's own under either. A clause that
      asserted only the first would falsify nothing. */
const groupSitsBesideTheBrand = async (browser, url) => {
  const page = await opened(browser, url, WIDTHS[0])

  const read = await page.evaluate(async () => {
    const footer = document.querySelector('footer')
    const controls = document.getElementById('controls')
    const sep = document.getElementById('sep-brand')
    const brand = document.getElementById('brand')

    const seen = []
    for (const appearance of ['system', 'light', 'dark']) {
      window.__harness.set({ appearance })
      window.__harness.fire('rendered')
      await new Promise((r) => setTimeout(r, 150))
      seen.push({
        appearance,
        gap: sep.getBoundingClientRect().left - controls.getBoundingClientRect().right,
        toBrand: brand.getBoundingClientRect().left - sep.getBoundingClientRect().right,
        brand: brand.getBoundingClientRect().left
      })
    }

    return {
      seen,
      columnGap: parseFloat(getComputedStyle(footer).columnGap),
      last: footer.lastElementChild === brand
    }
  })

  const errors = await drainErrors(page)
  await page.close()

  const off = (n) => Math.abs(n - read.columnGap) > 0.5
  const apart = read.seen.filter((s) => off(s.gap) || off(s.toBrand))
  const brands = new Set(read.seen.map((s) => s.brand.toFixed(2)))

  const said = (s) => `${s.appearance} ${s.gap.toFixed(2)} / ${s.toBrand.toFixed(2)}`
  note(`gap group→sep / sep→brand: ${read.seen.map(said).join('  ')}`)

  ok(
    9,
    `the icon group, the separator and the brand sit one gap apart at ${WIDTHS[0]}px, in all three states`,
    apart.length === 0 && brands.size === 1 && read.last,
    apart.length
      ? `the bar's own column-gap is ${read.columnGap}; read ${apart.map(said).join(', ')}`
      : `${read.columnGap} across both, in all three, the brand still last and unmoved at ${[...brands][0]}`
  )
  return errors
}

/* 10. **Each toggle works the pane it names, and its mark says which state it
       is in.** This clause used to assert one setting behind *two* controls,
       the bar's `Files` and `Lines` duplicating the header's, and it was
       re-keyed when the header gave its copies up: a copy that cannot disagree
       cannot exist, and `views-one-way` — the mutation that reached that
       disagreement — was withdrawn with it.

       **The pane is read and not just the attribute.** A control that placed
       its own state while the panel stayed open would satisfy every ARIA
       reading and be the defect this exists to catch. */
const viewsWorkTheirPanes = async (browser, url) => {
  const page = await opened(browser, url)

  const read = () =>
    page.evaluate(() => ({
      files: document.getElementById('views-files').getAttribute('aria-expanded'),
      lines: document.getElementById('views-lines').getAttribute('aria-pressed'),
      panel: !document.getElementById('files').classList.contains('collapsed'),
      gutter: !document.getElementById('lines').hidden,
      /* **The ink each mark is wearing**, because these two are marks and a
         mark says nothing a word does not. Since they carry no text, the
         *only* visible difference between on and off is this colour — so a
         stylesheet that lost the rule would leave two identical icons and every
         ARIA reading above would still pass. Which value it is is not the
         clause; that the two states differ is. */
      ink: {
        files: getComputedStyle(document.getElementById('views-files')).color,
        lines: getComputedStyle(document.getElementById('views-lines')).color
      }
    }))

  /* **Still four presses, and that is what the header's two rows became rather
     than what is left when they go.** Each toggle is pressed twice, once each
     way, because a single press would leave its mark in one ink — and *each
     mark's two inks* is the half of this clause `marks-unlit` owns and the
     requirement that determined this rewrite. */
  const pressed = []
  for (const [selector, which] of [
    ['#views-files', 'files'],
    ['#views-lines', 'lines'],
    ['#views-files', 'files'],
    ['#views-lines', 'lines']
  ]) {
    const before = await read()
    await page.click(selector)
    /* **Off the control before the colour is read.** A click leaves the pointer
       where it landed, `:hover` paints the mark with the same ink `on` does,
       and the reading below would then say the two states match whatever the
       toggle is actually in. Measured, not reasoned about: without this the
       off state read as ink in three of the four presses. */
    await page.mouse.move(0, 0)
    await settle(page)
    const after = await read()
    pressed.push({ press: pressed.length + 1, which, before, after })
  }

  const errors = await drainErrors(page)
  await page.close()

  /* The control says what the pane says, and the pane moved. `Files` is
     expanded-when-open, `Lines` pressed-when-shown, so each is compared against
     the box it works rather than against a literal. */
  const wrong = pressed.filter((p) => {
    const said = p.which === 'files' ? p.after.files : p.after.lines
    const box = p.which === 'files' ? p.after.panel : p.after.gutter
    const moved = String(p.which === 'files' ? p.before.panel : p.before.gutter) !== String(box)
    return said !== String(box) || !moved
  })

  /* The four presses put each toggle in both states, so each mark's two inks
     are in hand without a fifth reading. */
  const inks = (which) => new Set(pressed.filter((p) => p.which === which).map((p) => p.after.ink[which]))
  const marked = inks('files').size === 2 && inks('lines').size === 2

  for (const p of pressed)
    note(
      `press ${p.press}, ${p.which}: the control ${p.after[p.which]}, ` +
        `the pane ${p.which === 'files' ? p.after.panel : p.after.gutter}, the mark ${p.after.ink[p.which]}`
    )

  ok(
    10,
    'each view toggle works the pane it names, and its mark shows which state it is in',
    wrong.length === 0 && marked,
    wrong.length || !marked
      ? `${wrong.map((p) => `press ${p.press}, ${p.which}: ${JSON.stringify(p.after)}`).join('; ')}` +
        `${marked ? '' : ` — the marks: files ${[...inks('files')].join(' / ')}, lines ${[...inks('lines')].join(' / ')}`}`
      : 'four presses, the control and the pane agreeing after every one, each mark two inks'
  )
  return errors
}

/* 11. **The cell names what the pane is holding, and a figure is not
       `edited`.** Clicking an image row opens a surface over the text and never
       moves `Status::edited` — it cannot, `edited` being the file being typed
       in — so before this the bar named a markdown file that had not been on
       screen since the click. Both surfaces are asserted: the drawn figure and
       the sentence a `.pdf` row gets, which is still a surface the pane is
       holding.

       **And the way back is half the clause.** A cell that took the figure's
       name and kept it would read correctly in exactly the reading a one-ended
       check makes, so `Escape` is pressed and the markdown name must return. */
const cellNamesTheFigure = async (browser, url) => {
  const page = await opened(browser, url)

  const cell = () => page.evaluate(() => document.getElementById('edited').textContent)
  const clickRow = async (name) => {
    await page.evaluate((name) => {
      const row = [...document.querySelectorAll('#parts li')].find((li) => li.textContent.includes(name))
      row?.querySelector('button.name')?.click()
    }, name)
    await settle(page)
  }

  const before = await cell()
  await clickRow('mark.svg')
  const figure = await cell()
  /* **That the sheet holds a picture is part of the clause, not colour.** A
     refused read reaches the same surface through `saySoInstead` and names the
     same file, so a clause that only asked whether the surface was up passed
     against a harness that could not serve a figure at all — which is what it
     did, until `serve.mjs` started copying the project's images in. This is the
     reading that says the drawn path was the one taken. */
  const drawn = await page.evaluate(
    () => !document.getElementById('viewer').hidden && !!document.querySelector('#viewer .sheet img')
  )
  await clickRow('plan.pdf')
  const said = await cell()
  await page.keyboard.press('Escape')
  await settle(page)
  const back = await cell()

  const errors = await drainErrors(page)
  await page.close()

  note(`the cell: ${before} → mark.svg gives ${figure} → plan.pdf gives ${said} → Escape gives ${back}`)

  ok(
    11,
    'the cell names the figure the pane is holding, and the edited file again when it is left',
    figure === 'mark.svg' && drawn && said === 'plan.pdf' && back === before && before !== '',
    `opened on ${JSON.stringify(before)}; the figure gave ${JSON.stringify(figure)} with a picture drawn ${drawn}; ` +
      `the pdf's sentence gave ${JSON.stringify(said)}; Escape gave ${JSON.stringify(back)}`
  )
  return errors
}

/* 12. **The header's second mark says what it does, in both of its names.**
       `mpdf-003` Phase 17 turned this button from `Save` into `Save as…`, and
       Phase 16 shipped a recorded drop — nothing in either rig read the header's
       children at all — which this ends. It is the page's only visible change in
       that phase, and a mark that named the wrong action would be the exact
       defect Phase 16 deferred the rename to avoid: a button saying what it
       would do next release.

       **Both names and not one**, `wearAppearance`'s rule and the footer's: the
       `title` is what a sighted reader hovers for and the `aria-label` is what a
       screen reader says, so a page that moved one and not the other would tell
       two readers two different things. `title` renders in the shipping
       WKWebView — read at the window, since neither rig can see a native
       tooltip. */
const theSaveMarkSaysWhatItDoes = async (browser, url) => {
  const page = await opened(browser, url)

  const read = await page.evaluate(() => {
    const button = document.getElementById('save')
    return {
      title: button.getAttribute('title'),
      label: button.getAttribute('aria-label'),
      text: button.textContent.trim()
    }
  })
  const errors = await drainErrors(page)
  await page.close()

  note(`#save: title ${JSON.stringify(read.title)}, aria-label ${JSON.stringify(read.label)}`)

  ok(
    12,
    'the header\'s second mark says `Save as…` in both of its names',
    read.title === 'Save as…' && read.label === 'Save as…' && read.text === '',
    read.title === read.label
      ? `both say ${JSON.stringify(read.title)}`
      : `title ${JSON.stringify(read.title)} against aria-label ${JSON.stringify(read.label)}`
  )
  return errors
}

/* 13. **A save says what it did, and then stops saying it.**
       `mpdf-003` Phase 19 gave the bar a fifth cell, and the sentence in it is
       Rust's: `app/src/main.rs:save` answers it and `sayReceipt` places it. So
       this drives the page's own `save` listener — the menu's event, since `⌘S`
       has no button — and asserts the cell holds exactly what the stub answered.

       **Both halves, and the second is the one that needs the wait.** A receipt
       that appeared would pass a check written for the first half alone while the
       bar carried a stale sentence for the rest of the session, which is what the
       `receipt-sticks` mutation is. **It reads the cell's emptiness and never the
       timer's length**: no interval is written here, and the wait is
       `waitForFunction`'s own default, so the four seconds could move in the page
       without touching this file.

       **`Save as…` is deliberately not driven.** `stub.mjs`'s `dialog.save`
       answers `null`, so `saveDocumentAs` returns before it reaches the command —
       a rig that made that dialog answer would be testing a panel the app does
       not have. The window gate reads that half. */
const theSaveSaysWhatItDidAndThenStops = async (browser, url) => {
  const page = await opened(browser, url)

  const before = await page.evaluate(() => document.getElementById('receipt').textContent)

  await page.evaluate(() => {
    window.__harness.forget()
    window.__harness.fire('save')
  })

  const said = await page
    .waitForFunction(() => document.getElementById('receipt').textContent || null)
    .then((held) => held.jsonValue())
    .catch(() => '')

  /* The boundary, for the reason `stub.mjs` keeps the log at all: a page that
     worded its own receipt would look identical in the DOM. */
  const asked = await page.evaluate(() => window.__harness.invokes().map((sent) => sent.name))

  const cleared = await page
    .waitForFunction(() => document.getElementById('receipt').textContent === '')
    .then(() => true)
    .catch(() => false)

  const errors = await drainErrors(page)
  await page.close()

  note(`the cell: ${JSON.stringify(before)} → the save gave ${JSON.stringify(said)} → ${cleared ? 'empty again' : 'still there'}`)
  note(`what it asked Rust for: ${asked.join(', ') || 'nothing'}`)

  ok(
    13,
    'a plain save places the receipt Rust answered, and the cell is empty again after',
    before === '' && said === 'saved' && asked.includes('save') && cleared,
    `opened on ${JSON.stringify(before)}; the save gave ${JSON.stringify(said)} ` +
      `after asking for [${asked.join(', ')}]; it cleared: ${cleared}`
  )
  return errors
}

/* 14. **A drag of the divider resizes the pane and leaves the pane's own
       selection and its own focus alone.**
       `mpdf-003` Phase 20's clause. `setPointerCapture` routes the events of the
       gesture; it never cancelled the default action of the press that opened it,
       and the page had no rule about that default at all.

       **It drags both ways, and neither leg is padding.** The defect is not one
       defect: narrowing moves focus off the textarea onto `body` — so the
       keystroke after the drag is swallowed — and in WebKit leaves a `"\n"`
       selection behind; widening keeps focus and moves the author's caret
       instead. A clause that dragged only left would never see the caret move,
       and one that dragged only right would never see the focus loss.

       **Two assertions carry the mutation, and each carries it alone.** That is
       deliberate rather than belt-and-braces: a clause reading only the selection
       would pass in Chromium against the very page this exists to fix, since a
       leftward drag leaves `"\n"` there and nothing at all here. The focus loss
       and the moved caret both reproduce in both engines.

       **`String(document.getSelection())` and not `rangeCount`**, because the
       guilty WebKit page leaves a one-character selection that has a range.

       **The span is derived from the pane's own geometry at each leg's own
       start**, not chosen: the midpoint between the pane's current width and the
       `room - 160` ceiling the drag clamps to. At `opened()`'s own viewport the
       widening leg therefore lands short of that ceiling, and the pointer stays on
       the divider rather than wandering onto `#pages` — which would be a different
       gesture from the one under test. A pixel span would be the metric literal
       this file forbids, and would also be wrong twice over here: too long reaches
       the clamp, too short is below the span at which a selection appears at all.
       **The construction is not claimed at every viewport**: where `room - 160`
       falls below the pane's width the midpoint degenerates and the widening leg
       narrows instead.

       **It waits for the text layer before either leg.** The pages are not bare
       rasters — `pdf.js` lays transparent, real, selectable text over them — and
       without the layer the selection halves would be asserting against a pane
       with nothing to select. */
const dividerLeavesThePaneAlone = async (browser, url) => {
  const page = await opened(browser, url)
  await page.waitForFunction(() => document.querySelector('.textLayer span') !== null, null, { timeout: 30000 })

  /** One leg. `sign` is -1 to narrow and +1 to widen; everything else is read. */
  const leg = async (sign) => {
    const start = await page.evaluate((towards) => {
      const text = document.getElementById('text')
      text.focus()
      /* Mid-document rather than an offset written down: what is asserted is
         that it did not move, and a literal here would say nothing more. */
      const caret = Math.floor(text.value.length / 2)
      text.setSelectionRange(caret, caret)

      const pane = text.getBoundingClientRect()
      const room = document.body.clientWidth - pane.left
      const bar = document.getElementById('divider').getBoundingClientRect()
      return {
        caret,
        was: pane.width,
        by: (towards * (room - 160 - pane.width)) / 2,
        x: bar.left + bar.width / 2,
        y: bar.top + bar.height / 2
      }
    }, sign)

    await page.mouse.move(start.x, start.y)
    await page.mouse.down()
    await page.mouse.move(start.x + start.by, start.y, { steps: 5 })
    await page.mouse.up()
    await settle(page)

    const after = await page.evaluate(() => {
      const text = document.getElementById('text')
      const held = document.activeElement
      return {
        /* Named rather than compared to a boolean, so a failure says where the
           focus went — `body` is the answer this clause was written for. */
        holding: held === text ? 'text' : held ? held.id || held.tagName.toLowerCase() : 'nothing',
        selection: String(document.getSelection()),
        from: text.selectionStart,
        to: text.selectionEnd,
        now: text.getBoundingClientRect().width
      }
    })
    return { ...start, ...after }
  }

  const narrowed = await leg(-1)
  const widened = await leg(+1)

  const errors = await drainErrors(page)
  await page.close()

  const said = (r) => `${r.was.toFixed(1)} → ${r.now.toFixed(1)} by ${r.by.toFixed(1)}, focus ${r.holding}, ` +
    `caret ${r.caret} → ${r.from}${r.to === r.from ? '' : `–${r.to}`}, selection ${JSON.stringify(r.selection)}`

  note(`narrowing: ${said(narrowed)}`)
  note(`widening:  ${said(widened)}`)

  const moved = (r) => Math.abs(r.now - r.was) > 0.5

  ok(
    14,
    "a drag of the divider resizes the pane and leaves the pane's own selection and its own focus alone",
    moved(narrowed) &&
      narrowed.holding === 'text' &&
      narrowed.selection === '' &&
      moved(widened) &&
      widened.from === widened.caret &&
      widened.to === widened.caret &&
      widened.selection === '',
    [
      moved(narrowed) ? null : 'the narrowing drag did not resize the pane',
      narrowed.holding === 'text' ? null : `narrowing left the focus on ${narrowed.holding}`,
      narrowed.selection === '' ? null : `narrowing selected ${JSON.stringify(narrowed.selection)}`,
      moved(widened) ? null : 'the widening drag did not resize the pane',
      widened.from === widened.caret && widened.to === widened.caret
        ? null
        : `widening moved the caret from ${widened.caret} to ${widened.from}–${widened.to}`,
      widened.selection === '' ? null : `widening selected ${JSON.stringify(widened.selection)}`
    ]
      .filter(Boolean)
      .join('; ') || 'both legs resized, and neither touched the focus or the caret'
  )
  return errors
}

/* 15. **Each view's menu event works the pane its button works, and the page is
       listening for both.** `mpdf-003` Phase 21 gave the two toggles a `View`
       submenu and an accelerator each; the item emits and the page decides,
       which is `OPEN`'s rule and the reason there is anything here to drive.

       **It asserts the registration and not only the effect**, through
       `window.__harness.listening()` — offered by the stub since Phase 12 and
       used by no clause until this one. `fire` dispatches any name and is
       silent for one nothing registered, so a page that wired the two events to
       the wrong names would fail every reading below and give no account of why.

       **The marks are read as their ARIA attributes and never as their ink**,
       and the difference is a falsify failure rather than a taste: `marks-unlit`
       drops the `[aria-expanded='true']` and `[aria-pressed='true']` selectors,
       so a `getComputedStyle(...).color` reading here would fail under that
       mutation as well as clause 10 and report NOT ISOLATED. `aria-expanded`
       and `aria-pressed` are the branch that works; clause 10 keeps the ink.

       **And the writer's last two statements are read too.** The gutter's state
       moved off the pressed control and into the page so a menu event could
       reach it, and that refactor is exactly what could leave `relines()` and
       `markLine()` behind. So `view-lines` is fired **twice**, read on both
       sides of each: after the show the gutter holds one row per line, and
       after the **hide** the band is gone. The count is spelled because one
       fire would not do — `backgroundImage` is not `''` immediately after a
       show, when the band is painted, so a clause reading it there would fail
       on correct code. Both are properties, not metric literals, and neither
       costs `views-deaf` its isolation: no mutation touches either function.

       **`unbanded` was amended by `mpdf-003` Phase 23 and is still an absence
       assertion.** It read `text.style.backgroundImage`, which that phase makes
       `''` unconditionally by moving the band onto a class; it now reads the
       caret's own mirror row. It stayed an absence assertion only because the
       band stayed a `Lines` affordance — had the ink carried it in both modes
       there would have been no reproducible target after the hide, and
       `ink-band-tiles` would have failed 15 and 22 together.

       **The decline is asserted, because it is the one place the chord and the
       button differ.** With nothing open there is no panel, so `offerFold`
       hides the button and the still-enabled item's event must be declined by
       the page. A clause that only ever drove an open page would pass on a page
       that folded a panel that is not there. */
const viewsTakeTheirMenuEvents = async (browser, url) => {
  const page = await opened(browser, url)

  const heard = await page.evaluate(() => window.__harness.listening())

  const read = () =>
    page.evaluate(() => {
      const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
      return {
        filesMark: document.getElementById('views-files').getAttribute('aria-expanded'),
        linesMark: document.getElementById('views-lines').getAttribute('aria-pressed'),
        folded: document.getElementById('files').classList.contains('collapsed'),
        absent: document.getElementById('files').hidden,
        gutter: !document.getElementById('lines').hidden,
        rows: document.getElementById('lines').children.length,
        wanted: text.value.split('\n').length,
        /* **The caret's own mirror row, and not any marked row.** Since
           `mpdf-003` Phase 23 the band is a class on that row rather than a
           gradient on the textarea, whose `backgroundImage` this used to read
           and which is now `''` unconditionally — so the old reading went
           vacuously true and stopped testing that `markLine` ran on the hide.

           **The caret's row and not a walk over marked rows**: `markLine`'s
           clear is the one-element `marked` bookkeeping the page chose over a
           walk, so an assertion reading *any* marked row would also fail under
           `ink-band-tiles` and cost that mutation its isolation. Indexed the way
           `markLine` indexes it, off `selectionStart`, so the two are one
           answer to one question rather than a literal that is right by
           accident. */
        band: document.getElementById('mirror').children[
          text.value.slice(0, text.selectionStart).split('\n').length - 1
        ]?.className ?? ''
      }
    })

  const fire = async (name) => {
    await page.evaluate((n) => window.__harness.fire(n), name)
    await settle(page)
    return read()
  }

  /* One fire for the fold, read on both sides, and two for the gutter. */
  const openFold = [await read()]
  openFold.push(await fire('view-files'))

  const gutter = [await read()]
  gutter.push(await fire('view-lines'))
  gutter.push(await fire('view-lines'))

  /* The empty state, and the decline. `reset()` does not redraw on its own —
     `open()`'s caller fires `rendered` too — and it is that pass through
     `report` and `parts` which hides the panel and withdraws the button. */
  await page.evaluate(() => {
    window.__harness.reset()
    window.__harness.fire('rendered')
  })
  await settle(page)
  const empty = [await read()]
  empty.push(await fire('view-files'))

  const errors = await drainErrors(page)
  await page.close()

  const listening = heard.includes('view-files') && heard.includes('view-lines')

  /* The panel moved and the mark followed it. `Files` is expanded-when-open, so
     the mark is compared against the box it works rather than against a
     literal. */
  const [wasFolded, nowFolded] = openFold
  const foldMoved = wasFolded.folded !== nowFolded.folded
  const foldMarked = openFold.every((r) => r.filesMark === String(!r.folded))

  /* The gutter moved, its mark followed, and the two statements the writer ends
     with ran: the rows exist after the show, the band is gone after the hide. */
  const [wasShown, afterShow, afterHide] = gutter
  const linesMoved = !wasShown.gutter && afterShow.gutter && !afterHide.gutter
  const linesMarked = gutter.every((r) => r.linesMark === String(r.gutter))
  const relined = afterShow.rows === afterShow.wanted && afterShow.wanted > 1
  const unbanded = afterHide.band === ''

  /* Nothing to fold, and nothing folded: the panel stays away and the page left
     the mark and the class exactly where they were. */
  const [wasEmpty, stillEmpty] = empty
  const declined =
    wasEmpty.absent &&
    stillEmpty.absent &&
    stillEmpty.filesMark === wasEmpty.filesMark &&
    stillEmpty.folded === wasEmpty.folded

  note(`the page is listening for: ${heard.join(', ')}`)
  note(
    `view-files: the panel ${wasFolded.folded ? 'folded' : 'open'} → ${nowFolded.folded ? 'folded' : 'open'}, ` +
      `the mark ${wasFolded.filesMark} → ${nowFolded.filesMark}`
  )
  note(
    `view-lines: the gutter ${gutter.map((r) => r.gutter).join(' → ')}, ` +
      `the mark ${gutter.map((r) => r.linesMark).join(' → ')}, ` +
      `${afterShow.rows} rows for ${afterShow.wanted} lines, ` +
      `the caret's own ink row ${JSON.stringify(afterHide.band)} after the hide`
  )
  note(
    `with nothing open: #files hidden ${wasEmpty.absent} → ${stillEmpty.absent}, ` +
      `the mark ${wasEmpty.filesMark} → ${stillEmpty.filesMark}`
  )

  ok(
    15,
    "each view's menu event works the pane its button works, and the page is listening for both",
    listening && foldMoved && foldMarked && linesMoved && linesMarked && relined && unbanded && declined,
    [
      listening ? null : `the page registered [${heard.join(', ')}] and neither view event is among them`,
      foldMoved ? null : 'view-files left the panel where it was',
      foldMarked ? null : `the Files mark disagreed with the panel: ${JSON.stringify(openFold)}`,
      linesMoved ? null : `view-lines did not show the gutter and hide it again: ${gutter.map((r) => r.gutter).join(' → ')}`,
      linesMarked ? null : `the Lines mark disagreed with the gutter: ${JSON.stringify(gutter)}`,
      relined ? null : `the show left ${afterShow.rows} rows for ${afterShow.wanted} lines — relines() did not run`,
      unbanded
        ? null
        : `the hide left the caret's own ink row at ${JSON.stringify(afterHide.band)} — markLine() did not run`,
      declined
        ? null
        : `with nothing open the event was taken: #files hidden ${wasEmpty.absent} → ${stillEmpty.absent}, ` +
          `mark ${wasEmpty.filesMark} → ${stillEmpty.filesMark}, collapsed ${wasEmpty.folded} → ${stillEmpty.folded}`
    ]
      .filter(Boolean)
      .join('; ') ||
      'both events registered and taken, each mark following its pane, and the one with nothing open declined'
  )
  return errors
}

/* 16. **The one destructive gesture in this window is a mark, and a mark owes a
       reader two things a word gave for free.** It must say its own name, since
       there is no text to be one; and since the only visible difference between
       "about to delete something" and "quiet" is its ink, the ink has to reach
       the drawing. **Both are read here, and the second is the phase's one
       novel risk**: the mark is a `<use>` of a `<symbol>`, so the paint has to
       cross into a shadow tree. Measured with the `.trash svg` rule removed and
       nothing else changed, the `<use>` computes `fill: rgb(0,0,0)` and
       `stroke: none` — a black blob that never reddens — while every other
       reading below still passes.

       **The `<use>`'s own box and not the `<svg>`'s.** A `<use>` pointing at
       nothing leaves the `<svg>` at the size it declares with `fill` still
       `none`, so a clause reading either of those alone passes on a page whose
       mark is not there. The `<use>` is the ink's bounding box — non-zero when
       the reference resolves and 0×0 when it is dead — which is why it is
       asserted as a property and not against the 12 the `<svg>` declares. */
const theDeleteIsADrawnMark = async (browser, url) => {
  const page = await opened(browser, url)

  /* An image row carries the delete and no `main` button, which is what makes
     it the row to read the mark on; the width comparison at the end needs a row
     that carries both, and every such row gives the same pair, the controls
     being content-sized and name-independent. */
  const ROW = '#parts li[title="Look at sections/mark.svg"]'
  const BOTH = '#parts li[title="Edit other.md"]'

  const read = () =>
    page.evaluate(
      ([row, both]) => {
        const button = document.querySelector(`${row} .trash`)
        const svg = button?.querySelector('svg')
        const use = svg?.querySelector('use')
        const set = document.querySelector(`${both} .set`)
        const box = (el) => {
          const r = el?.getBoundingClientRect()
          return { width: r?.width ?? 0, height: r?.height ?? 0 }
        }
        return {
          text: button?.textContent ?? null,
          label: button?.getAttribute('aria-label') ?? null,
          /* The size it *declares*, read off the element rather than written
             down here: this file forbids a metric literal. */
          declared: { width: Number(svg?.getAttribute('width')), height: Number(svg?.getAttribute('height')) },
          svg: box(svg),
          use: box(use),
          ink: button ? getComputedStyle(button).color : null,
          stroke: use ? getComputedStyle(use).stroke : null,
          trash: box(button).width,
          set: box(set).width
        }
      },
      [ROW, BOTH]
    )

  /* The pointer starts nowhere in particular, so it is put somewhere the row is
     not before the quiet ink is read — clause 10's own lesson about a click
     leaving the pointer on the control it pressed. */
  await page.mouse.move(0, 0)
  await settle(page)
  const quiet = await read()

  /* **The row before the button, and that ordering is part of the clause.** The
     controls are `visibility: hidden` until the row is hovered, and a hidden
     element is not hoverable — a direct hover on the button times out in both
     engines. */
  await page.hover(ROW)
  await page.hover(`${ROW} .trash`)
  await settle(page)
  const lit = await read()

  const errors = await drainErrors(page)
  await page.close()

  const named = quiet.text === '' && !!quiet.label && quiet.label.includes('sections/mark.svg')
  const resolves = quiet.use.width > 0 && quiet.use.height > 0
  const sized = quiet.svg.width === quiet.declared.width && quiet.svg.height === quiet.declared.height
  const inks = new Set([quiet.ink, lit.ink]).size === 2
  const painted = quiet.stroke === quiet.ink && lit.stroke === lit.ink
  const narrower = quiet.trash < quiet.set

  note(`the button: text ${JSON.stringify(quiet.text)}, aria-label ${JSON.stringify(quiet.label)}`)
  note(
    `the svg ${quiet.svg.width}x${quiet.svg.height} against the ${quiet.declared.width}x${quiet.declared.height} it declares, ` +
      `the use ${quiet.use.width}x${quiet.use.height}`
  )
  note(`the ink: ${quiet.ink} off the row, ${lit.ink} on the button; the stroke ${quiet.stroke} → ${lit.stroke}`)
  note(`the controls: .trash ${quiet.trash} against .set ${quiet.set}`)

  ok(
    16,
    "the row's delete is a drawn mark that names itself and wears the alarm ink only under the pointer",
    named && resolves && sized && inks && painted && narrower,
    [
      named ? null : `it says ${JSON.stringify(quiet.text)} and is named ${JSON.stringify(quiet.label)}`,
      resolves ? null : `the use resolved to ${quiet.use.width}x${quiet.use.height} — the reference is dead`,
      sized ? null : `the svg is ${quiet.svg.width}x${quiet.svg.height} against ${quiet.declared.width}x${quiet.declared.height}`,
      inks ? null : `both states read ${quiet.ink}`,
      painted ? null : `the stroke is ${quiet.stroke} / ${lit.stroke} where the button is ${quiet.ink} / ${lit.ink}`,
      narrower ? null : `.trash is ${quiet.trash} against .set's ${quiet.set}`
    ]
      .filter(Boolean)
      .join('; ') ||
      `named ${JSON.stringify(quiet.label)}, the use ${quiet.use.width}x${quiet.use.height} in ` +
        `${quiet.ink} then ${lit.ink}, ${quiet.trash} against .set's ${quiet.set}`
  )
  return errors
}

/* 17. **A folder row folds what is under it, the fold survives the rebuild, and
       the panel gives the width back.** The first is the gesture, the second is
       what `parts` rebuilding the panel whole on every status puts at risk, and
       the third is the whole reason the phase exists — the panel is
       content-sized under a 40% cap, so folding away the rows that size it
       gives that width to the pages.

       **The expectation is derived here from the entries**, as clause 6's is,
       so this compares two derivations and not a page with itself. The rule is
       re-stated rather than read off the page: an entry is skipped when a
       collapsed folder is a *proper* path prefix of it, so `parts` collapsed
       takes `parts/ch1/deep.md` **and the `ch1` heading** while leaving the
       `parts` row itself standing — or there is no way back. Asserted as the
       whole ordered set, so it cannot pass on a page that dropped the right
       number of wrong rows.

       **All three folders and not one**, and the one-folder form was measured
       and refused: folding `parts` alone changes the width by nothing at all,
       because a content-sized panel is as wide as its *widest* row and that row
       is `loose/orphan.md`. Folding `loose` alone does narrow it, by the 0.45px
       that name beats `parts/ch1/deep.md`'s extra indent by — too fine to
       assert across two engines with different font metrics. Stated as
       narrower, never as a number, per this file's one rule. */
const theFolderRowFolds = async (browser, url) => {
  const page = await opened(browser, url)

  /* A row's identity, for both derivations. The folder mark is a `::before` and
     the trailing `/` an `::after`, so neither reaches `textContent` and a
     heading reads as the bare segment it did before this phase. */
  const read = () =>
    page.evaluate(() => ({
      rows: [...document.getElementById('parts').children].map(
        (li) =>
          `${li.classList.contains('folder') ? '[dir] ' : ''}${
            li.querySelector('.name')?.textContent ?? ''
          }@${li.dataset.depth}`
      ),
      width: document.getElementById('files').getBoundingClientRect().width
    }))

  const press = async (name) => {
    await page.evaluate((n) => {
      const row = [...document.getElementById('parts').children].find(
        (li) => li.classList.contains('folder') && li.querySelector('.name')?.textContent === n
      )
      row?.querySelector('button.name')?.click()
    }, name)
    await settle(page)
    return read()
  }

  const entries = await page.evaluate(() => window.__harness.config.entries)

  /* The heading's own root-relative path is what the fold is keyed by, so the
     derivation carries it beside the label the page draws. */
  const derive = (folded) => {
    const rows = []
    let folder = []
    for (const entry of entries) {
      const segments = entry.path.split('/')
      const here = segments.slice(0, -1)
      let shared = 0
      while (shared < here.length && here[shared] === folder[shared]) shared++
      for (let at = shared; at < here.length; at++) {
        const path = here.slice(0, at + 1).join('/')
        if (!folded.some((f) => path.startsWith(`${f}/`)))
          rows.push(`[dir] ${here[at]}@${Math.min(at, 5)}`)
      }
      folder = here
      if (!folded.some((f) => entry.path.startsWith(`${f}/`)))
        rows.push(`${segments[segments.length - 1]}@${Math.min(here.length, 5)}`)
    }
    return rows
  }

  /* The row is a button and it names its folder, where today it is a `<span>`
     and names nothing. Playwright computes the accessible name itself, so this
     reading does not fork by engine. */
  const heading = await page.evaluate(
    () =>
      [...document.getElementById('parts').children].find(
        (li) => li.classList.contains('folder') && li.querySelector('.name')?.textContent === 'parts'
      )?.querySelector('.name')?.tagName ?? 'none'
  )
  const named = await page.getByRole('button', { name: 'parts' }).count()

  const open = await read()
  const collapsed = await press('parts')
  const survived = await page.evaluate(() => window.__harness.fire('rendered')).then(async () => {
    await settle(page)
    return read()
  })
  const restored = await press('parts')

  await press('loose')
  await press('parts')
  const all = await press('sections')

  const errors = await drainErrors(page)
  await page.close()

  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b)
  const isButton = heading === 'BUTTON' && named === 1
  const drawsAll = same(open.rows, derive([]))
  const hides = same(collapsed.rows, derive(['parts']))
  const holds = same(survived.rows, collapsed.rows)
  const back = same(restored.rows, open.rows)
  const narrower = open.width - all.width > 0.5

  note(`the heading is a <${heading.toLowerCase()}>, named ${named} time(s)`)
  note(`open ${open.rows.length} rows, parts folded ${collapsed.rows.length}, back ${restored.rows.length}`)
  note(`#files ${open.width} open, ${all.width} with all three folded`)

  ok(
    17,
    'a folder row folds what is under it, the fold survives the rebuild, and the panel gives the width back',
    isButton && drawsAll && hides && holds && back && narrower,
    [
      isButton ? null : `the parts heading is a <${heading.toLowerCase()}> the role query found ${named} of`,
      drawsAll ? null : `unfolded  ${JSON.stringify(open.rows)}\n          wanted    ${JSON.stringify(derive([]))}`,
      hides ? null : `folded    ${JSON.stringify(collapsed.rows)}\n          wanted    ${JSON.stringify(derive(['parts']))}`,
      holds ? null : `a status redrew it as ${JSON.stringify(survived.rows)}`,
      back ? null : `pressing again left ${JSON.stringify(restored.rows)}`,
      narrower ? null : `#files is ${all.width} with all three folded against ${open.width} open`
    ]
      .filter(Boolean)
      .join('; ') ||
      `${open.rows.length} rows became ${collapsed.rows.length} and came back, ` +
        `${open.width} → ${all.width} with all three folded`
  )
  return errors
}

/* 18. **A bibliography row opens in the pane, and an image row still does
       not.** `mpdf-010` OQ-2 resolved on the finding that the reason for
       keeping a `.bib` shut stopped being true when Phase 2 separated `edited`
       from `main`: what compiles is the master, so the pane's own file may be
       anything the closure can be handed.

       **Three readings, and the last two are what stop this widening too far.**
       The first is the gesture — `edited` moves and `main` does not, read off
       `invoke('status')` rather than off the row, since a page that redrew a
       row without asking Rust would pass a DOM-only reading. The second is the
       row's `title`, which carries a kind test of its own: widening `opens`
       alone would leave `— not edited here` on exactly one row, the `.bib` the
       pane is holding, where the sentence is false and where the gesture lands.
       The third pins Phase 5 — an image still shows over the pane and leaves
       `edited` where it was — because **one term of one boolean is all that
       separates the two behaviours**, and a widening that swallowed images
       would otherwise pass everything above.

       The row is found by its `.name` text and clicked through the button a
       reader presses, clause 5's own idiom: `refs.bib` and `cover.jpg` are both
       root-level entries of `tests/fixtures/panel/`, so neither reading depends
       on a fold.

       **`#edited` is noted and deliberately not asserted**, and the isolation
       rule is what found that rather than a reading of the file. The cell
       following `edited` is clause 5's own property and `cell-main` is the
       mutation that owns it; a second clause asserting the same thing made that
       mutation fail two clauses and stop isolating. So this one is about what
       Rust was told — `invoke('status')` rather than the DOM, which is also the
       stronger reading, a page that redrew a row without asking Rust passing a
       DOM-only one. */
const theBibliographyRowOpens = async (browser, url) => {
  const page = await opened(browser, url)

  /* **The body being a `<button>` is what "opens" means here**, since that is
     the one difference `fileRow` draws between a row that goes in the pane and
     a row that does not — the click below is on the same element. */
  const read = (name) =>
    page.evaluate((n) => {
      const li = [...document.querySelectorAll('#parts li')].find(
        (el) => !el.classList.contains('folder') && el.querySelector('.name')?.textContent === n
      )
      return {
        found: !!li,
        opens: !!li?.querySelector('button.name'),
        title: li?.title ?? null,
        cell: document.getElementById('edited').textContent,
        status: window.__harness.status()
      }
    }, name)

  const click = async (name) => {
    await page.evaluate((n) => {
      const li = [...document.querySelectorAll('#parts li')].find(
        (el) => !el.classList.contains('folder') && el.querySelector('.name')?.textContent === n
      )
      li?.querySelector('button.name')?.click()
    }, name)
    await settle(page)
  }

  const before = await read('refs.bib')
  await click('refs.bib')
  const held = await read('refs.bib')

  /* Phase 5's behaviour, pinned: the figure goes up over the pane and the pane
     keeps the file it was holding — which after the click above is the `.bib`. */
  await click('cover.jpg')
  await settle(page)
  const figure = await page.evaluate(() => ({
    drawn: !document.getElementById('viewer').hidden && !!document.querySelector('#viewer .sheet img'),
    status: window.__harness.status()
  }))

  const errors = await drainErrors(page)
  await page.close()

  const offered = before.found && before.opens && before.title === 'Edit refs.bib'
  const took = held.status.edited === 'refs.bib' && held.status.main !== 'refs.bib'
  const unmoved = held.status.main === before.status.main
  const honest = held.title === 'refs.bib' && !held.title.includes('not edited here')
  const image = figure.drawn && figure.status.edited === held.status.edited

  note(`before the click: opens ${before.opens}, title ${JSON.stringify(before.title)}`)
  note(
    `after it: edited ${held.status.edited}, main ${held.status.main} (was ${before.status.main}), ` +
      `cell ${JSON.stringify(held.cell)}, title ${JSON.stringify(held.title)}`
  )
  note(`then cover.jpg: drawn ${figure.drawn}, edited ${figure.status.edited}`)

  ok(
    18,
    'a bibliography row opens in the pane and an image row still does not',
    offered && took && unmoved && honest && image,
    [
      offered ? null : `the row was found ${before.found}, a button ${before.opens}, titled ${JSON.stringify(before.title)}`,
      took ? null : `the click left edited ${held.status.edited} against main ${held.status.main}`,
      unmoved ? null : `main moved ${before.status.main} → ${held.status.main}`,
      honest ? null : `the held row's title is ${JSON.stringify(held.title)}`,
      image ? null : `cover.jpg drew ${figure.drawn} and left edited ${figure.status.edited}`
    ]
      .filter(Boolean)
      .join('; ') ||
      `refs.bib went from ${JSON.stringify(before.title)} to ${JSON.stringify(held.title)} with main at ${held.status.main}`
  )
  return errors
}

/* ----------------------------------------------------------------- the run */

/* 19. **The ink lays out to the textarea's own height.** `mpdf-003` Phase 23 put
       one element where a hidden ruler was: `#mirror` is the layout the gutter is
       measured from *and* the glyphs the reader sees, over `#text`'s own box,
       under a textarea gone transparent. What has to be true of that arrangement
       is that the two boxes wrap in the same places, and a height is what says so
       for every wrap at once.

       **No padding term**, and subtracting one would fail correct code by 24px:
       the two carry the same `padding: 12px 14px` over the same content width by
       the geometry that phase states, so the two `scrollHeight`s are directly
       comparable. **The one-pixel tolerance is measured, not hoped past** — a
       textarea and a `pre-wrap` div at the same font, padding and border-box
       width over 40 rows both report 1789 in Chromium and WebKit alike.

       **Narrowed by `opened`'s own width and not by clause 14's drag**, which now
       trips the ink's suppression flag; at `WIDTHS[2]` the pane holds about 23
       columns against the fixture's ~75-character lines, so it wraps three or
       four rows deep. **And the wrap is asserted rather than assumed**: the
       equality holds at any width, so a clause that only read it could pass
       without testing the agreement it is named for. A row taller than the
       shortest row is a logical line that took more than one.

       **The gutter is deliberately not the comparison here**: `regutter` assigns
       each gutter row its height from the mirror's own `offsetHeight`, so a
       mirror-versus-gutter reading holds by construction and says nothing about
       the textarea. */
const theInkLaysOutToThePane = async (browser, url) => {
  const page = await opened(browser, url, WIDTHS[2])

  const read = await page.evaluate(() => {
    const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
    const mirror = document.getElementById('mirror')
    const heights = Array.from(mirror.children, (row) => /** @type {HTMLElement} */ (row).offsetHeight)
    return {
      ink: mirror.scrollHeight,
      pane: text.scrollHeight,
      rows: mirror.children.length,
      wanted: text.value.split('\n').length,
      spans: mirror.querySelectorAll('span').length,
      wrapped: heights.some((h) => h > Math.min(...heights))
    }
  })
  const errors = await drainErrors(page)
  await page.close()

  const laid = Math.abs(read.ink - read.pane) <= 1
  const whole = read.rows === read.wanted && read.spans > 0

  note(`the ink ${read.ink} against the pane's ${read.pane}, ${read.rows} rows for ${read.wanted} lines`)
  note(`${read.spans} token spans, and the fixture ${read.wrapped ? 'wraps' : 'DOES NOT WRAP'} at this width`)

  ok(
    19,
    'the ink lays out to the height the textarea lays out to',
    laid && whole && read.wrapped,
    [
      laid ? null : `the ink stands ${read.ink} against the pane's ${read.pane}`,
      whole ? null : `${read.rows} rows for ${read.wanted} lines, ${read.spans} spans`,
      read.wrapped ? null : 'no row is taller than the shortest, so nothing wrapped and the reading is vacuous'
    ]
      .filter(Boolean)
      .join('; ') || `both ${read.ink} over ${read.rows} wrapped rows`
  )
  return errors
}

/* 20. **The ink draws with `Lines` off, and the gutter does not.** The feature is
       the pane the author's hands are in, not a mode: `#lines` ships `hidden` and
       `shown` is `false`, so a highlighting layer gated the way the gutter is
       would colour nothing until `⌘L` and would not be the feature.

       **Both halves, and each is one of the two things the split bought.** In the
       default state the ink is whole and carries tokens while the gutter is
       hidden *and empty*; `⌘L` then fills the gutter without touching the ink.

       **Read as `textContent`, a row count and a span count, never `innerHTML`**:
       the show marks the caret's row on the mirror, so an `innerHTML` comparison
       would fail on correct code. And the span count is compared **across the
       toggle** rather than to a number — a literal would be the metric this file
       forbids, and it would also cost `ink-include-anywhere` its isolation, that
       mutation changing which class a span wears and, with it, how many there
       are. */
const theInkDrawsWithLinesOff = async (browser, url) => {
  const page = await opened(browser, url)

  const read = () =>
    page.evaluate(() => {
      const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
      const mirror = document.getElementById('mirror')
      const lines = document.getElementById('lines')
      return {
        rows: mirror.children.length,
        wanted: text.value.split('\n').length,
        spans: mirror.querySelectorAll('span').length,
        ink: mirror.textContent,
        hidden: lines.hidden,
        gutter: lines.children.length
      }
    })

  const before = await read()
  await pressLines(page)
  const after = await read()

  const errors = await drainErrors(page)
  await page.close()

  const drawn = before.rows === before.wanted && before.wanted > 1 && before.spans > 0
  const gated = before.hidden && before.gutter === 0
  const filled = !after.hidden && after.gutter === after.wanted
  const untouched = after.rows === before.rows && after.spans === before.spans && after.ink === before.ink

  note(`off: ${before.rows} ink rows for ${before.wanted} lines, ${before.spans} spans; #lines hidden ${before.hidden} with ${before.gutter} rows`)
  note(`on:  ${after.rows} ink rows, ${after.spans} spans; #lines hidden ${after.hidden} with ${after.gutter} rows`)

  ok(
    20,
    'the ink draws with the gutter off, and the gutter fills without touching it',
    drawn && gated && filled && untouched,
    [
      drawn ? null : `the ink drew ${before.rows} rows and ${before.spans} spans for ${before.wanted} lines`,
      gated ? null : `#lines was hidden ${before.hidden} with ${before.gutter} rows before the toggle`,
      filled ? null : `the toggle left #lines hidden ${after.hidden} with ${after.gutter} rows for ${after.wanted} lines`,
      untouched ? null : `the toggle moved the ink: ${before.rows}→${after.rows} rows, ${before.spans}→${after.spans} spans`
    ]
      .filter(Boolean)
      .join('; ') ||
      `${before.spans} spans over ${before.rows} rows either way, and #lines went 0 → ${after.gutter}`
  )
  return errors
}

/* 21. **An include marker is not an inline link, and a missing target is
       neither.** `[](sections/three.md)` on its own line is what the compiler
       reads a section in by; the same thing inside a sentence is an inert link.
       The pane draws the difference, and a marker naming a file the panel lists
       `missing` gets the alarm.

       **The class is read off the span whose text is the destination**, so this
       clause encodes no class name — it asserts that the three are distinct,
       which is the property, and would go on holding if the names changed.

       **It types the third case in.** It was the only clause here that wrote into
       `#text` until `mpdf-003` Phase 24 added three that do the same, and the
       idiom below is the one they took. `sections/missing.md` cannot join the fixture: the engine
       raises `Error::MissingSection` and `serve.mjs` would `die` before a clause
       ran, which is why `book.md` deliberately does not name it. So one `fill`,
       the class read, the buffer restored — and the panel already lists that path
       `missing`, `PANEL_ENTRIES` carrying the row no walk can produce. */
const anIncludeIsNotAnInlineLink = async (browser, url) => {
  const page = await opened(browser, url)

  const classOf = (destination) =>
    page.evaluate((want) => {
      for (const span of document.querySelectorAll('#mirror span')) {
        if (span.textContent === want) return span.className
      }
      return null
    }, destination)

  const marker = await classOf('sections/text.md')
  const link = await classOf('other.md')

  const held = await page.evaluate(() => document.getElementById('text').value)
  await page.fill('#text', `${held}\n[](sections/missing.md)\n`)
  await settle(page)
  const absent = await classOf('sections/missing.md')
  await page.fill('#text', held)
  await settle(page)
  const restored = await page.evaluate(() => document.getElementById('text').value)

  const errors = await drainErrors(page)
  await page.close()

  const found = marker !== null && link !== null && absent !== null
  const apart = found && new Set([marker, link, absent]).size === 3

  note(`the marker "${marker}", the inline link "${link}", the missing target "${absent}"`)
  note(`the buffer was ${restored === held ? 'restored' : 'NOT RESTORED'} after the fill`)

  ok(
    21,
    'an include marker, an inline link and a missing target are three different inks',
    apart && restored === held,
    [
      found ? null : `one of them drew no span at all: marker ${marker}, link ${link}, missing ${absent}`,
      !found || apart ? null : `they are not three: ${JSON.stringify([marker, link, absent])}`,
      restored === held ? null : 'the fill left the buffer changed'
    ]
      .filter(Boolean)
      .join('; ') || `${JSON.stringify([marker, link, absent])}`
  )
  return errors
}

/* 22. **The band is on the caret's row, and on one row.** Phase 8 painted it as
       the textarea's own `background-image`, and records that the missing
       `no-repeat` cost a build: a gradient sized to one measured row tiles down
       the whole pane by default and every line wears one. Phase 23 made the band
       a class on a row, which is the element form of the same construct — and so
       the same bug is available again, in a form no `background-repeat` protects
       against.

       **Both columns, because the band is now two marks through one index.** The
       gutter's row and the mirror's are marked and cleared together by `unmark()`
       and the one `marked`, and a clause reading only one of them would not see a
       rebuild of the other strand a mark.

       **And with `Lines` off it marks neither**, which is the affordance staying
       where Phase 8 put it: the mode that pays for the measurement is the mode
       that gets the band, and `README.md` describes it as what `⌘L` adds. */
const theBandIsOnTheCaretsRow = async (browser, url) => {
  const page = await opened(browser, url)

  const marks = () =>
    page.evaluate(() => {
      const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
      const at = (box) =>
        Array.from(box.children).flatMap((row, i) => (row.classList.contains('here') ? [i] : []))
      return {
        ink: at(document.getElementById('mirror')),
        gutter: at(document.getElementById('lines')),
        caret: text.value.slice(0, text.selectionStart).split('\n').length - 1
      }
    })

  /* The caret is put on a line by its own index rather than by a character
     offset, so the reading below compares two answers to the same question. */
  const putCaret = async (line) => {
    await page.evaluate((n) => {
      const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
      text.focus()
      const at = text.value.split('\n').slice(0, n).join('\n').length
      text.setSelectionRange(at, at)
    }, line)
    await settle(page)
    return marks()
  }

  const off = await marks()
  await pressLines(page)
  const first = await putCaret(3)
  const moved = await putCaret(7)
  await pressLines(page)
  const back = await marks()

  const errors = await drainErrors(page)
  await page.close()

  const one = (r) => r.ink.length === 1 && r.gutter.length === 1 && r.ink[0] === r.caret && r.gutter[0] === r.caret
  const none = (r) => r.ink.length === 0 && r.gutter.length === 0

  note(`with Lines off: ink [${off.ink}], gutter [${off.gutter}]`)
  note(`caret on row ${first.caret}: ink [${first.ink}], gutter [${first.gutter}]`)
  note(`caret on row ${moved.caret}: ink [${moved.ink}], gutter [${moved.gutter}]`)
  note(`after the hide: ink [${back.ink}], gutter [${back.gutter}]`)

  ok(
    22,
    "the band is on the caret's row in both columns, on one row, and on neither with Lines off",
    none(off) && one(first) && one(moved) && none(back),
    [
      none(off) ? null : `with Lines off it marked ink [${off.ink}] and gutter [${off.gutter}]`,
      one(first) ? null : `on row ${first.caret} it marked ink [${first.ink}] and gutter [${first.gutter}]`,
      one(moved) ? null : `moving to row ${moved.caret} left ink [${moved.ink}] and gutter [${moved.gutter}]`,
      none(back) ? null : `the hide left ink [${back.ink}] and gutter [${back.gutter}]`
    ]
      .filter(Boolean)
      .join('; ') || `one row each on ${first.caret} then ${moved.caret}, and neither column marked outside Lines mode`
  )
  return errors
}


/* 23. **The bracket family reads five ways, and an unbracketed `@` is none of
       them.** `mpdf-003` Phase 24. Every construct here opens `[`, and before
       that phase one branch answered for all of them — which is how
       `[](#fig:pipeline)` came to draw as an inline link, its destination the ink
       a URL gets, where the dialect records that *it is the empty brackets that
       make a reference*. A construct drawn as a different construct is worse than
       one drawn as prose.

       **The class is read off the span whose text is the destination or the
       label**, so this clause encodes no class name: what it asserts is that the
       five are distinct, which is the property, and it would go on holding if the
       names changed.

       **Three negative terms, and they carry equal weight.** `a@b.com` and a bare
       `@thing` must draw nothing — the dialect records that the brackets are
       required *because an unbracketed `@` is load-bearing in ordinary text*, and
       a lexer without that term paints every email address in a document. And
       `[this one](#fig:x)` — a **texted** anchor link — must read as an ordinary
       link, which the dialect also states; keying on the `#` alone would
       reintroduce this phase's own defect in mirror image.

       **The fill is pinned by three constraints and they are not incidental**:
       the inline link's destination does not end `.md`, or the shipped
       `ink-include-anywhere` claims it and this clause fails under a mutation it
       does not own; no line carries `: ` before a bracket outside a definition,
       for `ink-captions-anywhere`; and no line carries math, which is clause 25's
       to fill. */
const theBracketFamilyReadsFiveWays = async (browser, url) => {
  const page = await opened(browser, url)

  const held = await page.evaluate(() => document.getElementById('text').value)
  await page.fill(
    '#text',
    [
      '[](sections/text.md)',
      'A reference to [](#fig:pipeline) and a link [this one](#fig:pipeline).',
      'A citation [@quill2019] and a footnote[^note].',
      'A link [text](https://typst.app) beside a@b.com and a bare @thing.'
    ].join('\n')
  )
  await settle(page)

  const read = await page.evaluate(() => {
    const classOf = (want) => {
      for (const span of document.querySelectorAll('#mirror span')) {
        if (span.textContent === want) return span.className
      }
      return null
    }
    const text = /** @type {HTMLTextAreaElement} */ (document.getElementById('text'))
    return {
      include: classOf('sections/text.md'),
      ref: classOf('#fig:pipeline'),
      cite: classOf('quill2019'),
      note: classOf('note'),
      link: classOf('https://typst.app'),
      /* The texted anchor's own span, which shares its text with the reference's
         — so it is read as the *last* of the two rather than by text alone. */
      texted: [...document.querySelectorAll('#mirror span')]
        .filter((s) => s.textContent === '#fig:pipeline')
        .map((s) => s.className),
      /* Nothing on the last line may wear a span but the link's own four. */
      bare: [...document.getElementById('mirror').children[3].querySelectorAll('span')].map(
        (s) => s.textContent
      )
    }
  })

  await page.fill('#text', held)
  await settle(page)
  const restored = await page.evaluate(() => document.getElementById('text').value)
  const errors = await drainErrors(page)
  await page.close()

  const five = [read.include, read.ref, read.cite, read.note, read.link]
  const found = five.every((c) => c !== null)
  const apart = found && new Set(five).size === 5
  const texted = read.texted.length === 2 && read.texted[0] !== read.texted[1]
  const prose = !read.bare.some((t) => t.includes('@'))

  note(`the five: ${JSON.stringify(five)}`)
  note(`the anchor twice, as a reference then as a texted link: ${JSON.stringify(read.texted)}`)
  note(`spans on the prose line: ${JSON.stringify(read.bare)}`)

  ok(
    23,
    'the bracket family reads five ways, and an unbracketed @ is none of them',
    apart && texted && prose && restored === held,
    [
      found ? null : `one drew no span at all: ${JSON.stringify(five)}`,
      !found || apart ? null : `they are not five: ${JSON.stringify(five)}`,
      texted ? null : `a texted anchor read the same as a reference: ${JSON.stringify(read.texted)}`,
      prose ? null : `an unbracketed @ was drawn: ${JSON.stringify(read.bare)}`,
      restored === held ? null : 'the fill left the buffer changed'
    ]
      .filter(Boolean)
      .join('; ') || `${JSON.stringify(five)}, and the anchor ${read.texted.join(' then ')}`
  )
  return errors
}

/* 24. **A group, a caption and a name are drawn, and a `:` inside a sentence is
       not a caption.** The `:::` opener's kind word is the one thing the author
       chose; a caption's marker opens a paragraph; a name may fall anywhere.

       **The caption is read with its own emphasis run inside it**, which is the
       one-array claim: names and inline math live in `inlineRuns` rather than the
       block chain precisely so a caption's marker, its emphasis and its name come
       out of one scanner in one ascending array — `paintRow` fills gaps from
       `at = token.to`, so a second array would have to be merged and an
       out-of-order run would draw the text twice.

       **The negative is the dialect's own**: it records that *everywhere but the
       start of a paragraph, `:::` is ordinary text*, and the same holds for the
       colon — a `:` mid-sentence is a colon. */
const aGroupACaptionAndAName = async (browser, url) => {
  const page = await opened(browser, url)

  const held = await page.evaluate(() => document.getElementById('text').value)
  await page.fill(
    '#text',
    [
      '::: abstract',
      '',
      ': A caption with an *emphasis* run. {#fig:pipeline}',
      '',
      'Prose with a : colon mid-sentence and a { brace that opens no name.'
    ].join('\n')
  )
  await settle(page)

  const read = await page.evaluate(() => {
    const rows = document.getElementById('mirror').children
    const spans = (i) =>
      [...rows[i].querySelectorAll('span')].map((s) => `${s.className}:${s.textContent}`)
    return { group: spans(0), caption: spans(2), prose: spans(4) }
  })

  await page.fill('#text', held)
  await settle(page)
  const errors = await drainErrors(page)
  await page.close()

  const kind = read.group.some((s) => s.endsWith(':abstract'))
  const marker = read.caption.some((s) => s.startsWith('quiet:'))
  const inside = read.caption.some((s) => s.endsWith(':emphasis'))
  const named = read.caption.some((s) => s.endsWith(':fig:pipeline'))
  const prose = read.prose.length === 0

  note(`the group: ${JSON.stringify(read.group)}`)
  note(`the caption: ${JSON.stringify(read.caption)}`)
  note(`the prose line drew ${read.prose.length} spans`)

  ok(
    24,
    "a group, a caption and a name are drawn, and a colon inside a sentence is not a caption",
    kind && marker && inside && named && prose,
    [
      kind ? null : `the group's kind drew ${JSON.stringify(read.group)}`,
      marker ? null : 'the caption drew no marker',
      inside ? null : `the caption lost its own emphasis run: ${JSON.stringify(read.caption)}`,
      named ? null : `the caption's name drew nothing: ${JSON.stringify(read.caption)}`,
      prose ? null : `a colon or a brace in prose was drawn: ${JSON.stringify(read.prose)}`
    ]
      .filter(Boolean)
      .join('; ') || 'the kind, the marker, the emphasis and the name, and prose left alone'
  )
  return errors
}

/* 25. **Display math is one ink and is not read as markdown.** A `$$` block is
       block state, and it is checked *before* the fast path — which is the whole
       of this clause. A body line like `a plain body line` holds none of the six
       characters `inert` scans for, so a `$$` state checked after that path would
       let it fall through and draw as nothing at all.

       **The body line this fills is deliberately trigger-free**, because a body
       holding a `*` would be caught by a per-line lexer too and the clause would
       pass on the implementation it exists to reject.

       And `$5` alone is not math: the twin pairs two dollars on one line, so one
       opens nothing. */
const displayMathIsOneInk = async (browser, url) => {
  const page = await opened(browser, url)

  const held = await page.evaluate(() => document.getElementById('text').value)
  await page.fill(
    '#text',
    ['$$', 'a plain body line with no trigger character', '$$', '', 'It cost $5 in prose.'].join(
      '\n'
    )
  )
  await settle(page)

  const read = await page.evaluate(() => {
    const rows = document.getElementById('mirror').children
    const spans = (i) =>
      [...rows[i].querySelectorAll('span')].map((s) => `${s.className}:${s.textContent}`)
    return { open: spans(0), body: spans(1), shut: spans(2), prose: spans(4) }
  })

  await page.fill('#text', held)
  await settle(page)
  const errors = await drainErrors(page)
  await page.close()

  const fenced = read.open.length === 1 && read.shut.length === 1
  const body = read.body.length === 1 && read.body[0].startsWith(read.open[0].split(':')[0])
  const prose = read.prose.length === 0

  note(`the fences: ${JSON.stringify(read.open)} / ${JSON.stringify(read.shut)}`)
  note(`the body: ${JSON.stringify(read.body)}`)
  note(`the prose line drew ${read.prose.length} spans`)

  ok(
    25,
    'a display block is one ink, its body included, and a lone dollar in prose is not math',
    fenced && body && prose,
    [
      fenced ? null : `the fences drew ${JSON.stringify(read.open)} and ${JSON.stringify(read.shut)}`,
      body
        ? null
        : `the body drew ${JSON.stringify(read.body)} — a trigger-free line fell through to prose`,
      prose ? null : `a lone dollar was drawn as math: ${JSON.stringify(read.prose)}`
    ]
      .filter(Boolean)
      .join('; ') || 'both fences and the body in one ink, and $5 left alone'
  )
  return errors
}


const run = async ({ engine, headed, rev, doc, mutate }) => {
  passed = 0
  failed = 0
  owned.length = 0

  const held = await serve({ rev, doc, mutate, quiet: true })
  const browser = await (engine === 'webkit' ? webkit : chromium).launch({ headless: !headed })
  const errors = { total: 0, loops: 0, spoken: [] }
  const gather = (seen) => {
    errors.total += seen.total
    errors.loops += seen.loops
    errors.spoken.push(...seen.spoken)
  }

  console.log(
    `\n${engine}${headed ? ', headed' : ''} | ${rev ?? 'the working tree'} | ${doc}` +
      `${mutate ? ` | MUTATED ${mutate}` : ''}\n`
  )

  try {
    for (const check of [
      elementOrder,
      sweep,
      statusPlaces,
      cellFollowsThePane,
      panelDrawsTheEntries,
      paletteTurnsBothWays,
      cellPlacesAndDoesNotDecide,
      groupSitsBesideTheBrand,
      viewsWorkTheirPanes,
      cellNamesTheFigure,
      theSaveMarkSaysWhatItDoes,
      theSaveSaysWhatItDidAndThenStops,
      dividerLeavesThePaneAlone,
      viewsTakeTheirMenuEvents,
      theDeleteIsADrawnMark,
      theFolderRowFolds,
      theBibliographyRowOpens,
      theInkLaysOutToThePane,
      theInkDrawsWithLinesOff,
      anIncludeIsNotAnInlineLink,
      theBandIsOnTheCaretsRow,
      theBracketFamilyReadsFiveWays,
      aGroupACaptionAndAName,
      displayMathIsOneInk
    ]) {
      gather(await check(browser, held.url))
    }
  } finally {
    await browser.close()
    await held.close()
  }

  /* **Named and not merely counted**, and the `ResizeObserver` class counted
     apart: that is the failure `mpdf-009` Phase 3 found twenty-one of in a
     single run, and an unrelated throw must still be visible beside it.
     **It stays last** — it is the only clause that accumulates across every
     other one, so its number moves as clauses are added and theirs do not. */
  ok(
    26,
    'no uncaught error reached the console through any of it',
    errors.total === 0,
    `${errors.total} uncaught, ${errors.loops} of them ResizeObserver` +
      (errors.spoken.length ? ` — ${errors.spoken.join(' | ')}` : '')
  )

  console.log(`\n${passed + failed} clauses: ${passed} passed, ${failed} failed`)
  return { passed, failed, owned: [...owned] }
}

const engine = has('webkit') ? 'webkit' : 'chromium'
const shared = { engine, headed: has('headed'), rev: flag('rev'), doc: flag('doc', 'tests/fixtures/panel') }

if (has('falsify')) {
  /* The gate's clause 3, judged rather than read: each mutation must fail exactly
     the clause that owns it and no other.

     **One child process per mutation, and that is measured rather than tidy.** A
     second or third `chromium.launch()` in the same process hangs — no browser
     alive, the driver waiting on a promise that never settles — which cost this
     phase two runs and is the same thing the A/B driver forks around. Each child
     is this file with `--mutate`, so the forked path and the single-mutation path
     are one path, and its exit code already means "isolated". */
  const rest = argv.filter((a) => a !== '--falsify')
  const verdicts = Object.keys(OWNS).map((mutate) => {
    const child = spawnSync(process.execPath, [fileURLToPath(import.meta.url), ...rest, '--mutate', mutate], {
      stdio: 'inherit'
    })
    return { mutate, clause: OWNS[mutate], isolated: child.status === 0 }
  })
  console.log(`\nfalsification in ${engine}:`)
  for (const v of verdicts) {
    console.log(`  ${v.isolated ? 'ISOLATED    ' : 'NOT ISOLATED'}  ${v.mutate} owns clause ${v.clause}`)
  }
  process.exit(verdicts.every((v) => v.isolated) ? 0 : 1)
}

const mutate = flag('mutate')
const answer = await run({ ...shared, mutate })

if (mutate) {
  const isolated = answer.owned.length === 1 && answer.owned[0] === OWNS[mutate]
  console.log(
    `${isolated ? 'ISOLATED' : 'NOT ISOLATED'}  ${mutate} owns clause ${OWNS[mutate]}, failed [${answer.owned.join(', ')}]`
  )
  process.exit(isolated ? 0 : 1)
}

process.exit(answer.failed === 0 ? 0 : 1)
