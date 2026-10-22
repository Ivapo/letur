/* mpdf-010 Phase 10 exit gate — the window half (clause 9). Paste into the Web
   Inspector console of a `cargo tauri dev` window.

   The Rust half is `cargo test --workspace`, which holds the rule: a create
   makes its folders and nothing else, a `..` in a folder still to be made is
   refused before anything is made, a link out of the project is refused
   wherever it sits, a dangling link is refused as outside, and the create lists
   its file before the pane moves to it. The page half is `app/harness/checks.mjs`
   clause 27. What only a window can say is that **all of it meets**: the real
   command makes the folders, the real relist puts the row under them, the real
   `set_edited` moves the pane — and over unsaved work, the create still happens
   while the switch is refused in `SWITCHING`'s words.

   ONE SETUP STEP, in a terminal, before you start:

     rm -rf "$TMPDIR/mpdf-010-phase10"
     cp -R tests/fixtures/panel "$TMPDIR/mpdf-010-phase10"
     echo "$TMPDIR/mpdf-010-phase10"

   **It runs over a copy and never over the repository**, for Phase 3's
   reason: `tests/fixtures/panel/` is enumerated row by row by
   `app/src/document.rs:the_listing_is_the_disk_and_what_the_master_names`, so
   any create in it fails that test on the next `cargo test`. Remaking the copy
   is what makes the run re-runnable. `git status` is clean before and after.

   **It has no preconditions**: every clause is about the DOM and about
   `invoke('status')`'s own answer.

   ORDER:
     __gate.arm()              <- BEFORE opening anything, from the empty state
     open <copy>/sections/text.md
     await __gate.create()     <- clauses 1, 2
     await __gate.dirty()      <- clauses 3, 4
     __gate.report()

   Run this against the build before this phase and clause 1 fails on the
   field's own sentence: `appendix/notes/extra.md would land outside this
   project`, because the folder did not exist.                                */
;(() => {
  const list = document.getElementById('parts')
  const text = document.getElementById('text')
  const divergence = document.getElementById('divergence')
  const said = document.getElementById('divergence-text')
  const files = document.getElementById('files')
  const newToggle = document.getElementById('new-toggle')
  const newFile = document.getElementById('new-file')
  const newName = document.getElementById('new-name')
  const newRefusal = document.getElementById('new-refusal')
  const { invoke } = window['__TAURI__'].core

  /* `project/src/preview.rs:SWITCHING`'s opening words. Enough to say which
     refusal it is, and short of the clause that names the way out. */
  const SWITCHING = 'the pane holds unsaved edits, so it is still holding this file.'

  let noise = 0
  const spoken = []
  const wait = (ms) => new Promise((r) => setTimeout(r, ms))

  /* The command relists before it returns and the pane's move then loads the
     file, so a second is many times what either takes; this script is read by a
     person rather than by a timer. */
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

  /* The panel as a reader sees it, Phase 3's reading: every row's text plus its
     depth, a folder heading spelled `name/`. */
  const drawn = () =>
    [...list.children].map((li) => {
      const name = li.querySelector('.name')?.textContent ?? ''
      return `${li.classList.contains('folder') ? name + '/' : name}@${li.dataset.depth}`
    })

  /* Phase 3's `askFor`: reveal the field, type the whole root-relative path,
     submit the form with `requestSubmit` so the input's own `required` runs. */
  const askFor = async (path) => {
    if (newFile.hidden) newToggle.click()
    newName.value = path
    newFile.requestSubmit()
    await settled()
  }

  /* Phase 2's `type`: the page listens for `input`, which `value = …` alone
     does not raise. */
  const type = (what) => {
    const at = text.value.length
    text.focus()
    text.setRangeText(what, at, at, 'end')
    text.dispatchEvent(new Event('input', { bubbles: true }))
  }

  let before = null

  window.__gate = {
    report() {
      console.log(`mpdf-010 Phase 10 gate\n${transcript.join('\n')}`)
      return `${transcript.length} lines`
    },

    arm() {
      noise = 0
      spoken.length = 0
      before = null
      // **Named and not merely counted**, for Phase 3's reason.
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

      ok(0, 'the empty state holds no panel', files.hidden, `files.hidden ${files.hidden}`)

      console.log(
        '%carmed%c  — now open $TMPDIR/mpdf-010-phase10/sections/text.md, then run: await __gate.create()',
        'font-weight:bold;color:#1a73e8',
        'color:inherit'
      )
    },

    async create() {
      heading('a create that names two folders that do not exist')
      await settled()

      before = await invoke('status')
      note(`main is ${before.main}, the pane holds ${before.edited}`)

      const PATH = 'appendix/notes/extra.md'
      await askFor(PATH)
      const now = drawn()

      /* Clause 1. **The folders were made and the row is under them.** No
         `appendix/` existed in the copy, so a row under two headings is the
         command having made both. The refusal beside the field is read too:
         the pre-phase build answers with it, and the clause should say so. */
      const at = now.indexOf('extra.md@2')
      ok(
        1,
        'the create made appendix/notes and drew extra.md under both headings',
        newRefusal.hidden && at >= 2 && now[at - 1] === 'notes/@1' && now[at - 2] === 'appendix/@0',
        newRefusal.hidden
          ? `around it: ${now.slice(Math.max(at - 2, 0), at + 1).join(' ')}`
          : `refused: ${newRefusal.textContent}`
      )

      /* Clause 2. **The pane holds what was made, and the main is unmoved.**
         Read off Rust's own status, not off the marked row. */
      const after = await invoke('status')
      ok(
        2,
        'the pane holds the new file and the main still compiles',
        after.edited === PATH && after.main === before.main,
        `edited ${after.edited}, main ${after.main} (was ${before.main})`
      )

      note('now run: await __gate.dirty()')
      return tally('create')
    },

    async dirty() {
      heading('a create over unsaved work')

      const held = (await invoke('status')).edited
      type('x')
      await settled()

      const PATH = 'appendix/more.md'
      await askFor(PATH)
      const now = drawn()
      const after = await invoke('status')
      const sentence = said.textContent ?? ''

      /* Clause 3. **The create happens and the switch is refused.** The row is
         there — files before folders, so `more.md` sits under `appendix/`
         ahead of `notes/` — the pane is still on the file with the work in it,
         and the divergence bar carries `SWITCHING`'s sentence, as a row click
         over unsaved work does. */
      const at = now.indexOf('more.md@1')
      ok(
        3,
        'the row appears, the pane stays on the unsaved file, and the bar says SWITCHING',
        newRefusal.hidden &&
          at > 0 &&
          now[at - 1] === 'appendix/@0' &&
          after.edited === held &&
          !divergence.hidden &&
          sentence.includes(SWITCHING),
        `row at ${at} (${now.slice(Math.max(at - 1, 0), at + 2).join(' ')}), ` +
          `edited ${after.edited} (was ${held}), divergence hidden ${divergence.hidden}, said "${sentence}"`
      )

      ok(4, 'no error reached the console', noise === 0,
        `${noise} uncaught${spoken.length ? ' — ' + spoken.join(' | ') : ''}`)

      note('run __gate.report() to copy the whole transcript back, then check `git status` is clean.')
      note('the pane still holds unsaved work in extra.md: discard it, and remake the copy before a second run.')
      return tally('dirty')
    }
  }

  console.log(
    '%c__gate ready%c  —  run __gate.arm() now, before opening anything.\n' +
      'Then: open $TMPDIR/mpdf-010-phase10/sections/text.md → create() → dirty() → report().',
    'font-weight:bold;color:#1a73e8',
    'color:inherit'
  )
})()
