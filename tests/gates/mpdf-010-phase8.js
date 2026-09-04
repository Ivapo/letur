/* mpdf-010 Phase 8 exit gate — the window half. Paste into the Web Inspector
   console of a `cargo tauri dev` window.

   The Rust half is `cargo test --workspace`, which holds the two clauses about
   the bytes: the pane's unsaved bibliography reaches the citation pass, and
   `⌘S` writes the `.bib` and not the master. The harness holds clause 18, which
   says the row opens and an image row still does not. **What only a window can
   say is the phase itself** — that an edit nobody has saved is in the page the
   app draws, because no rig in this repository has a compile behind it:
   `app/harness/serve.mjs` stubs Rust, so its `set_edited` invents the pane's
   text and its `current_pdf` answers one file compiled once.

   **It has no preconditions**, as Phases 1, 2 and 5's had none: every clause is
   about the DOM and about `invoke('status')`'s own answer, so no failure it
   reports can be about the size of your window or its pixel ratio.

   **It writes nothing at all**, which is stronger than Phase 5's "writes
   nothing" and is the phase working rather than a courtesy: the whole claim is
   that the page redraws *without a save*, so `tests/fixtures/panel/refs.bib` is
   never touched. The buffer is left dirty and `finish()` discards it through
   the app's own command, which is `Preview::load`'s path. `git status` is clean
   before and after.

   ORDER:
     __gate.arm()              <- BEFORE opening anything, from the empty state
     open tests/fixtures/panel/book.md
     await __gate.offers()     <- clauses 1, 2
     await __gate.holds()      <- clauses 3, 4
     await __gate.redraws()    <- clause 5, the observable
     await __gate.finish()     <- clause 6, and the pane put back
     __gate.report()

   The fixture and not a sample deliberately: `tests/fixtures/panel/book.md` is
   the one master in this repository that both names a bibliography and cites a
   key out of it, which Phase 8's own commit made it. `samples/showcase/` names
   `refs.bib` too and would serve for clauses 1 to 4, but its reference list is
   long enough that reading one entry back out of the text layer is a worse
   instrument than reading the only entry there is.

   Run this against the build before this phase and it stops at clause 1:
   `refs.bib`'s body was a `<span>`, so `clickRow` answers `null` and there is
   nothing to put in the pane. Clause 5 is unreachable there by construction. */
;(() => {
  const list = document.getElementById('parts')
  const text = document.getElementById('text')
  const pages = document.getElementById('pages')
  const edited = document.getElementById('edited')
  const problem = document.getElementById('error')
  const divergence = document.getElementById('divergence')
  const { invoke } = window['__TAURI__'].core

  /* The one field the reference list prints. **`WAS` is deliberately not the
     needle**: `book.md`'s own frontmatter carries the same title, so it is on
     the page twice and its absence would say nothing. `NOW` is a string no file
     in this repository holds, so finding it in the text layer cannot be an
     accident, and the clauses below turn on that one direction. */
  const WAS = 'A Book the Panel Lists'
  const NOW = 'A Title Nobody Has Saved'

  let noise = 0
  const spoken = []
  const wait = (ms) => new Promise((r) => setTimeout(r, ms))

  /* The compile's, matching Phase 2's `settled`. The redraw after a keystroke
     costs the typing debounce as well, so clause 5 waits twice. */
  const settled = () => wait(1000)

  let pass = 0
  let fail = 0
  /* Every line is kept as well as logged: Safari's Web Inspector copies one
     console entry at a time, and `__gate.report()` gives the whole run back as
     one entry. */
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
  const heading = (s) => {
    transcript.push('', `== ${s}`)
    console.log(`%c\n${s}\n`, 'font-weight:bold')
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
    [...list.children].find(
      (li) => !li.classList.contains('folder') && li.querySelector('.name')?.textContent === name
    )

  /* Click a row's own body, which is what the reader clicks. `null` when the
     row is not there or is not a button, so a clause can say which. */
  const clickRow = (name) => {
    const button = row(name)?.querySelector('button.name')
    button?.click()
    return button ?? null
  }

  /* **The drawn page, read off the text layer `pdf.js` builds.** The canvas is
     pixels and says nothing a clause can assert; the text layer is the same
     stream the reader selects from, so this asks the page what it says rather
     than what it was told to say. */
  const drawn = () =>
    [...pages.querySelectorAll('.textLayer')].map((el) => el.textContent).join(' ')

  /* **Whitespace is squashed on both sides**, because `pdf.js` builds one span
     per text item and a line's own spacing is CSS rather than characters — so a
     needle compared against the raw `textContent` can miss for a reason that is
     about the renderer and not about this phase. */
  const squash = (s) => s.replace(/\s+/g, '')
  const holdsText = (page, needle) => squash(page).includes(squash(needle))
  const countOf = (page, needle) => squash(page).split(squash(needle)).length - 1

  /* One replacement, typed the way a keyboard types it: the page listens for
     `input`, which `value = …` alone does not raise. */
  const retitle = (from, to) => {
    const at = text.value.indexOf(from)
    if (at < 0) return false
    text.focus()
    text.setRangeText(to, at, at + from.length, 'end')
    text.dispatchEvent(new Event('input', { bubbles: true }))
    return true
  }

  /* **The file as the disk holds it**, through the command the figure surface
     already uses — `document::asset_bytes` confines the path and reads it and
     asks nothing about a kind, so a `.bib` comes back like a `.png`. This is
     what lets "nothing was written" be read rather than assumed. */
  const onDisk = async (path) =>
    new TextDecoder().decode(new Uint8Array(await invoke('asset_bytes', { path })))

  let before = null
  let held = ''
  let typed = false

  window.__gate = {
    report() {
      console.log(`mpdf-010 Phase 8 gate\n${transcript.join('\n')}`)
      return `${transcript.length} lines`
    },

    arm() {
      noise = 0
      spoken.length = 0
      before = null
      held = ''
      typed = false
      // **Named and not merely counted.** A run reporting "3 uncaught" and
      // nothing else sends the next round looking for something it cannot see.
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

      ok(0, 'the empty state draws no rows', list.children.length === 0,
        `${list.children.length} rows with nothing open`)

      console.log(
        '%carmed%c  — now open tests/fixtures/panel/book.md, then run: await __gate.offers()',
        'font-weight:bold;color:#1a73e8',
        'color:inherit'
      )
    },

    async offers() {
      heading('tests/fixtures/panel/book.md — the bibliography row offers to open')
      await settled()
      before = await invoke('status')

      /* Clause 1. The row's body is a button and its title says what pressing
         it does — the two halves of `fileRow`'s `opens`, which are separate
         terms in the page and are read separately here. */
      const bib = row('refs.bib')
      const body = bib?.querySelector('button.name') ?? null
      ok(1, 'the bibliography row is a button that offers to edit that file',
        body !== null && bib.title === 'Edit refs.bib',
        `body <${(bib?.querySelector('.name')?.tagName ?? 'none').toLowerCase()}>, title ${JSON.stringify(bib?.title ?? null)}`)

      /* Clause 2. The master is what compiles, and the page it drew already
         holds the bibliography's own entry — so clause 5's comparison has a
         *before* that is not merely "the string is absent". */
      const page = drawn()
      /* Twice: the document's own title and the reference the master cites.
         Noted rather than asserted as a number — what the clause turns on is
         that `NOW` is not there yet. */
      ok(2, 'the master compiles and its reference list is in the drawn page',
        before.main === 'book.md' && before.edited === 'book.md' &&
        before.state === 'current' && holdsText(page, WAS) && !holdsText(page, NOW),
        `main ${before.main}, edited ${before.edited}, state ${before.state}, ` +
        `${page.length} chars drawn, "${WAS}" ${countOf(page, WAS)} time(s), ` +
        `"${NOW}" ${countOf(page, NOW)}`)

      note('now run: await __gate.holds()')
      return tally('offers')
    },

    async holds() {
      heading('the pane takes the bibliography, and the main goes on compiling')
      if (before === null) {
        ok(3, 'the pane holds the bibliography', false, 'run __gate.offers() first')
        return tally('holds')
      }

      if (clickRow('refs.bib') === null) {
        ok(3, 'the pane holds the bibliography', false,
          'no row for refs.bib carries a name button — this is the build before Phase 8')
        return tally('holds')
      }
      await settled()

      const state = await invoke('status')
      held = text.value
      /* Clause 3. The two values move apart, which is Phase 2's separation
         asked of a file that is not markdown, and the pane holds that file's
         own bytes rather than a stand-in. */
      ok(3, 'the pane holds the bibliography and the main is where it was',
        state.edited === 'refs.bib' && state.main === before.main &&
        held.includes(WAS) && held.includes('@book{panel'),
        `edited ${state.edited}, main ${state.main} (was ${before.main}), ` +
        `${held.split('\n').length} lines in the pane, cell ${JSON.stringify(edited.textContent)}`)

      /* Clause 4. **The row's title, which is a second kind test in the page.**
         Widening `opens` without it puts "not edited here" on exactly this row
         — the one the pane is holding — where the sentence is false. Read here
         rather than trusted, which is what Phase 8's round 1 asked for. */
      const title = row('refs.bib')?.title ?? null
      const marks = [...list.children].filter((li) => li.classList.contains('holding'))
      ok(4, 'the held row says its own path and not that it is not edited here',
        title === 'refs.bib' && !String(title).includes('not edited here') &&
        marks.length === 1 && marks[0].querySelector('.name')?.textContent === 'refs.bib',
        `title ${JSON.stringify(title)}, ${marks.length} row(s) marked holding`)

      note('now run: await __gate.redraws()')
      return tally('holds')
    },

    async redraws() {
      heading('the observable — an unsaved bibliography edit is in the drawn page')
      if (!held) {
        ok(5, 'the page redraws with the unsaved title', false, 'run __gate.holds() first')
        return tally('redraws')
      }

      const was = drawn()
      typed = retitle(WAS, NOW)
      if (!typed) {
        ok(5, 'the page redraws with the unsaved title', false,
          `the pane holds no "${WAS}" to retitle`)
        return tally('redraws')
      }
      // The typing debounce, then the compile behind it.
      await settled()
      await settled()

      const state = await invoke('status')
      const page = drawn()
      /* **"Without saving" is read off the disk and not assumed.** The whole
         claim is that the page carries text no file holds, so the file is
         fetched back and asked. `WAS` dropping by exactly one is the other half:
         the reference took the new title and the document's own heading — which
         carries the same string out of `book.md`'s frontmatter — did not. */
      const file = await onDisk('refs.bib')
      ok(5, 'the page redraws with the unsaved title, and nothing was written',
        holdsText(page, NOW) && countOf(page, WAS) === countOf(was, WAS) - 1 &&
        text.value.includes(NOW) && file.includes(WAS) && !file.includes(NOW) &&
        state.state === 'current' && state.main === before.main &&
        state.edited === 'refs.bib' && state.revision > before.revision &&
        problem.hidden && !pages.classList.contains('stale'),
        `"${NOW}" ${countOf(page, NOW)} time(s) where it was ${countOf(was, NOW)}; ` +
        `"${WAS}" ${countOf(page, WAS)} where it was ${countOf(was, WAS)} — the reference took it ` +
        `and the document's own title did not; the file on disk holds "${NOW}" ` +
        `${file.includes(NOW)}; state ${state.state}, revision ${state.revision} ` +
        `(was ${before.revision})`)

      note('the buffer is dirty on purpose — run: await __gate.finish()')
      return tally('redraws')
    },

    async finish() {
      heading('the pane put back, and the console')

      /* **Through the app's own command**, which is what the refusal names and
         is `Preview::load` behind it. Nothing here writes, so this is the whole
         of the restoration. */
      if (typed) {
        await invoke('discard')
        await settled()
      }

      const state = await invoke('status')
      const back = drawn()
      ok(6, 'the discard takes the file again and the page goes back with it',
        text.value.includes(WAS) && !text.value.includes(NOW) &&
        holdsText(back, WAS) && !holdsText(back, NOW) &&
        state.divergence === null && divergence.hidden,
        `the pane holds "${WAS}" ${text.value.includes(WAS)}, the page holds it ` +
        `${countOf(back, WAS)} time(s) and "${NOW}" ${countOf(back, NOW)}; ` +
        `divergence ${JSON.stringify(state.divergence)}`)

      ok(7, 'no error reached the console', noise === 0,
        `${noise} uncaught${spoken.length ? ' — ' + spoken.join(' | ') : ''}`)

      note('run __gate.report() to copy the whole transcript back, then check `git status` is clean.')
      return tally('finish')
    }
  }

  console.log(
    '%c__gate ready%c  —  run __gate.arm() now, before opening anything.\n' +
      'Then: open tests/fixtures/panel/book.md → offers() → holds() → redraws() →\n' +
      'finish() → report().',
    'font-weight:bold;color:#1a73e8',
    'color:inherit'
  )
})()
