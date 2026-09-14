/* mpdf-010 Phase 9 exit gate — the window half. Paste into the Web Inspector
   console of a `cargo tauri dev` window.

   There is no Rust half. This phase touches no `.rs` file: the bytes already
   crossed for every other kind, and `document::asset_bytes` does not know what
   it is reading. What only a window can say is that **the PDF is drawn rather
   than described** — a canvas with pixels in it, inside the column and the box
   Phase 5 built for an `<img>` — and that **the row the pane is holding is a way
   back** without being a way to lose unsaved work.

   **It has no preconditions**, as Phase 5's had none: every clause is about the
   DOM and about `invoke('status')`'s own answer, so no failure it reports can be
   about the size of your window or its pixel ratio.

   **It writes nothing.** Clause 5 types one character into the pane and takes it
   out again; no `⌘S` is pressed, so no file is touched. `git status` is clean
   before and after.

   ORDER:
     __gate.arm()              <- BEFORE opening anything, from the empty state
     open tests/fixtures/panel/sections/text.md
     await __gate.draws()      <- clauses 1, 2, 3, 9
     await __gate.back()       <- clauses 4, 5, 6
     await __gate.ways()       <- clause 7
     await __gate.release()    <- clause 8
     __gate.report()

   The fixture is the project because `tests/fixtures/panel/plan.pdf` is the only
   tracked `.pdf` in this repository — `samples/showcase/showcase.pdf` is built
   and `.gitignore`d, so a second person on a fresh clone would have nothing to
   click. **No second `.pdf` was added for the parse-failure path**: the phase
   prices that at six edits across four files, four of them silent, and refuses
   it. Clause 1 is what covers the failure that reads as the app having done
   nothing.

   Run this against the build before this phase and clause 1 fails at once:
   `plan.pdf` shows a sentence and the sheet holds no canvas at all.          */

;(() => {
  const list = document.getElementById('parts')
  const text = document.getElementById('text')
  const pages = document.getElementById('pages')
  const viewer = document.getElementById('viewer')
  const problem = document.getElementById('error')
  const { invoke } = window['__TAURI__'].core

  let noise = 0
  const spoken = []
  const wait = (ms) => new Promise((r) => setTimeout(r, ms))

  /* A parse and a raster, both inside one task. 600 ms is some hundreds of times
     what either costs on the one-page fixture, and this script is read by a
     person rather than by a timer. */
  const drawn = () => wait(600)

  let pass = 0
  let fail = 0
  const transcript = []
  const ok = (n, name, good, detail) => {
    good ? pass++ : fail++
    transcript.push(
      `${good ? 'PASS' : 'FAIL'}  ${String(n).padStart(2)}. ${name}${detail ? '  —  ' + detail : ''}`
    )
    console.log(
      `%c${good ? 'PASS' : 'FAIL'}%c  ${String(n).padStart(2)}. ${name}${detail ? '  —  ' + detail : ''}`,
      `font-weight:bold;color:${good ? '#137333' : '#c5221f'}`,
      'color:inherit'
    )
  }
  const note = (s) => {
    transcript.push(`····  ${s}`)
    console.log(`%c····%c  ${s}`, 'color:#888', 'color:#888')
  }
  const tally = (what) => {
    transcript.push(`${what}: ${pass} passed, ${fail} failed`)
    console.log(
      `%c${what}: ${pass} passed, ${fail} failed`,
      `font-weight:bold;color:${fail ? '#c5221f' : '#137333'}`
    )
    const answer = { passed: pass, failed: fail }
    pass = 0
    fail = 0
    return answer
  }

  const row = (name) =>
    [...list.children].find((li) => li.querySelector('.name')?.textContent === name)
  const clickRow = (name) => {
    const button = row(name)?.querySelector('button.name')
    button?.click()
    return button ?? null
  }

  /* **The held row is found by its own mark and never by name.** Which file the
     pane holds after an open is Phase 1's climb talking, and a clause that
     hard-codes `text.md` fails for a reason that is not this phase's. */
  const heldRow = () => list.querySelector('li.holding')
  const clickHeld = () => {
    const button = heldRow()?.querySelector('button.name')
    button?.click()
    return button ?? null
  }

  const sheet = () => viewer.querySelector('.sheet')
  const canvas = () => viewer.querySelector('.sheet canvas')

  /* The sheet's content box, which is what `max-height: 100%` resolves against.
     Phase 5's own reading, unchanged. */
  const sheetBox = () => {
    const it = sheet()
    const style = getComputedStyle(it)
    return {
      width: it.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
      height: it.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom),
      overflow: it.scrollHeight - it.clientHeight
    }
  }

  /* **Two pixels that differ, not a comparison with a named colour.** Nothing
     in this phase says what the canvas is cleared to, so a clause keyed to a
     background would be keyed to an implementation choice the spec does not
     make. A canvas of the right size drawing nothing is what this catches, and
     the size alone would pass without it. */
  const notUniform = (c) => {
    const px = c.getContext('2d').getImageData(0, 0, c.width, c.height).data
    for (let i = 4; i < px.length; i += 4) {
      if (px[i] !== px[0] || px[i + 1] !== px[1] || px[i + 2] !== px[2] || px[i + 3] !== px[3]) {
        return true
      }
    }
    return false
  }

  const overText = () =>
    viewer.offsetLeft === text.offsetLeft && viewer.offsetWidth === text.offsetWidth

  const type = (ch) => {
    text.value += ch
    text.dispatchEvent(new Event('input', { bubbles: true }))
  }

  /* **A plateau alone cannot say a compile has finished**, because it reads
     identically before one has *started*: the debounce is Rust's and
     `watch::TYPING_DEBOUNCE` is 300 ms, so two reads 250 ms apart can both land
     inside the wait and agree on the old number. This waits the debounce out
     first and then asks for quiet across three reads — 750 ms, against a compile
     `mpdf-003` §2 puts at 8.5 and 28.7 ms.

     **This was clause 6's own first failure and is written down as one.** The
     first run of this gate reported `revision settled at 2, 2 → 3`: the restore
     below dispatched an edit, the plateau-detector answered before the debounce
     fired, and the compile landed during the click clause 6 was measuring. The
     app moved nothing — `hideAsset` invokes nothing at all — and the instrument
     was the thing that was wrong. */
  const settledRevision = async () => {
    await wait(800)
    let last = -1
    let same = 0
    for (let i = 0; i < 40; i++) {
      const now = (await invoke('status')).revision
      same = now === last ? same + 1 : 0
      last = now
      if (same >= 2) return now
      await wait(250)
    }
    return last
  }

  window.__gate = {
    report() {
      console.log(`mpdf-010 Phase 9 gate\n${transcript.join('\n')}`)
      return `${transcript.length} lines`
    },

    arm() {
      noise = 0
      spoken.length = 0
      const say = (what) => {
        noise++
        if (spoken.length < 8) spoken.push(what)
      }
      addEventListener('error', (e) =>
        say(`error: ${e.message || e.error} @ ${e.filename || '?'}:${e.lineno || '?'}`)
      )
      addEventListener('unhandledrejection', (e) =>
        say(`rejection: ${(e.reason && (e.reason.message || e.reason.name)) || String(e.reason)}`)
      )

      ok(0, 'the empty state holds no figure', viewer.hidden, `viewer.hidden ${viewer.hidden}`)
      console.log(
        '%carmed%c  — now open tests/fixtures/panel/sections/text.md, then run: await __gate.draws()',
        'font-weight:bold;color:#1a73e8',
        'color:inherit'
      )
    },

    async draws() {
      const was = await invoke('status')
      const held = text.value

      clickRow('plan.pdf')
      await drawn()

      const c = canvas()
      const img = viewer.querySelector('.sheet img')
      ok(
        1,
        'a PDF row draws a canvas with something on it, and no <img>',
        !!c && !img && notUniform(c),
        c
          ? `canvas ${c.width}×${c.height} backing store, ${img ? 'an <img> beside it, ' : 'no <img>, '}` +
            `pixels ${notUniform(c) ? 'differ' : 'are uniform — nothing was drawn'}`
          : `no canvas in the sheet; it holds ${sheet().firstElementChild?.tagName ?? 'nothing'}`
      )

      const now = await invoke('status')
      ok(
        2,
        'it is a view: edited, main and revision are unmoved and the pane holds its text',
        now.edited === was.edited && now.main === was.main && now.revision === was.revision &&
          text.value === held,
        `edited ${was.edited} → ${now.edited}, main ${was.main} → ${now.main}, ` +
          `revision ${was.revision} → ${now.revision}, pane text ${text.value === held ? 'same' : 'CHANGED'}`
      )

      const box = sheetBox()
      const drawnBox = c ? c.getBoundingClientRect() : { width: 0, height: 0 }
      ok(
        3,
        'the surface is over the text column and the canvas fits the sheet',
        overText() && !!c &&
          drawnBox.width <= box.width + 1 && drawnBox.height <= box.height + 1 && box.overflow === 0,
        `viewer ${viewer.offsetLeft}+${viewer.offsetWidth} vs text ${text.offsetLeft}+${text.offsetWidth}; ` +
          `canvas CSS box ${Math.round(drawnBox.width)}×${Math.round(drawnBox.height)} ` +
          `(store ${c?.width}×${c?.height}, expected larger at a ratio above 1) ` +
          `in a sheet of ${Math.round(box.width)}×${Math.round(box.height)}, overflow ${box.overflow}`
      )

      ok(
        9,
        'a click that compiled nothing has not marked the page stale',
        !pages.classList.contains('stale'),
        `#pages classes: ${pages.className || '(none)'}`
      )

      note('now run: await __gate.back()')
      return tally('draws')
    },

    async back() {
      /* Clause 4 wants a figure up. `draws()` leaves one, but this raises its
         own when there is none, so the step can be re-run by itself after a
         failure without replaying the whole gate. */
      if (viewer.hidden) {
        clickRow('plan.pdf')
        await drawn()
      }
      const was = await invoke('status')
      const button = clickHeld()
      await wait(200)
      ok(
        4,
        'clicking the row the pane holds puts the text back',
        !!button && viewer.hidden,
        button
          ? `viewer.hidden ${viewer.hidden}`
          : 'the held row has no button — it is still a <span>'
      )
      const after = await invoke('status')
      ok(
        4.1,
        '…and moves nothing while doing it',
        after.edited === was.edited && after.divergence === was.divergence,
        `edited ${was.edited} → ${after.edited}, divergence ` +
          `${JSON.stringify(was.divergence)} → ${JSON.stringify(after.divergence)}`
      )

      // Clause 5 — the same gesture with unsaved work in the pane. This is the
      // clause that fails if the row is ever routed through `openInPane`.
      const held = text.value
      type('x')
      clickRow('plan.pdf')
      await drawn()
      const dirty = await invoke('status')
      clickHeld()
      await wait(200)
      const back = await invoke('status')
      ok(
        5,
        'the same click with unsaved work keeps the work and asks nothing',
        viewer.hidden && text.value === held + 'x' &&
          back.edited === dirty.edited && back.divergence === dirty.divergence,
        `viewer.hidden ${viewer.hidden}, pane ${text.value === held + 'x' ? 'kept the keystroke' : 'LOST it'}, ` +
          `edited ${dirty.edited} → ${back.edited}, divergence ` +
          `${JSON.stringify(dirty.divergence)} → ${JSON.stringify(back.divergence)}`
      )

      // Put the character back where it was found, then let the compile it
      // armed finish before clause 6 reads `revision`.
      text.value = held
      text.dispatchEvent(new Event('input', { bubbles: true }))
      const rest = await settledRevision()

      const quiet = await invoke('status')
      clickHeld()
      await wait(300)
      const still = await invoke('status')
      ok(
        6,
        'with no figure up the same click changes nothing',
        viewer.hidden && still.revision === quiet.revision && still.edited === quiet.edited &&
          text.value === held && problem.hidden,
        `revision settled at ${rest}, ${quiet.revision} → ${still.revision}; ` +
          `edited ${quiet.edited} → ${still.edited}; error bar hidden ${problem.hidden}`
      )

      note('now run: await __gate.ways()')
      return tally('back')
    },

    async ways() {
      clickRow('plan.pdf')
      await drawn()
      const upOnce = !viewer.hidden && !!canvas()
      dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
      await wait(200)
      const afterEscape = viewer.hidden

      clickRow('plan.pdf')
      await drawn()
      const upTwice = !viewer.hidden && !!canvas()
      document.getElementById('close-viewer').click()
      await wait(200)
      const afterButton = viewer.hidden

      ok(
        7,
        'Escape and Back to the text both put a PDF figure away',
        upOnce && afterEscape && upTwice && afterButton,
        `up ${upOnce} → Escape hid it ${afterEscape}; up again ${upTwice} → the control hid it ${afterButton}`
      )
      note('now run: await __gate.release()')
      return tally('ways')
    },

    async release() {
      /* **The middle step is the point.** A PDF followed by an image is the one
         sequence that reaches `showAsset`'s own release site rather than either
         of the two a reader can see, and it is the sequence that leaked a worker
         before the three became one function. What a page can observe is the
         sheet; the single call site is what carries the rest. */
      clickRow('plan.pdf')
      await drawn()
      const one = sheet().childElementCount
      clickRow('cover.jpg')
      await drawn()
      const two = sheet().childElementCount
      const isImage = !!viewer.querySelector('.sheet img')
      clickRow('plan.pdf')
      await drawn()
      const three = sheet().childElementCount
      const isCanvas = !!canvas()
      dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
      await wait(200)

      ok(
        8,
        'PDF → image → PDF → Escape leaves one child at each step and an empty sheet',
        one === 1 && two === 1 && three === 1 && isImage && isCanvas &&
          viewer.hidden && sheet().childElementCount === 0,
        `children ${one} → ${two} (an <img> ${isImage}) → ${three} (a canvas ${isCanvas}) → ` +
          `${sheet().childElementCount}; viewer.hidden ${viewer.hidden}`
      )

      ok(
        10,
        'no error reached the console',
        noise === 0,
        `${noise} uncaught${spoken.length ? ' — ' + spoken.join(' | ') : ''}`
      )

      note('run __gate.report() to copy the whole transcript back, then check `git status` is clean.')
      return tally('release')
    }
  }

  console.log(
    '%c__gate ready%c  —  run __gate.arm() now, before opening anything.\n' +
      'Then: open tests/fixtures/panel/sections/text.md → draws() → back() → ways() → release() → report().',
    'font-weight:bold;color:#1a73e8',
    'color:inherit'
  )
})()
