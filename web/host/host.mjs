/*
  Letur's window, answered in a browser tab — `ltr-001` Phase 2.

  `app/dist/index.html` talks to its backend through one object,
  `window['__TAURI__']`, read at module top level. This module supplies it.
  `web/assemble.sh` puts this script in `<head>` of a copy of that page, where
  `app/harness/serve.mjs` puts its stub, so it runs before the window's own
  module does. **The page is not edited for the browser's sake**: a behaviour
  the browser cannot offer is refused here, in words of its own.

  **What is here and what is not.** Every rule — the panel's order, the main,
  the compile, the refusals, the receipts, the counters — is `letur-project`'s,
  run by `web/src/lib.rs` in `worker.mjs`. What is here is what only a page can
  do: the surface itself, the typing debounce, the `rendered` and `opened`
  events the desktop's loops would send, the keyboard the desktop's menu
  carries, the file input, the downloads, the title, and the seed that opens a
  landing-page example as a project.
*/

const PROJECT = '/project/'
const DOCUMENT = `${PROJECT}document.md`
/** The desktop's `TYPING_DEBOUNCE`, `app/src/watch.rs`. */
const TYPING_DEBOUNCE = 300
/** The one global key the appearance is kept under. */
const APPEARANCE = 'letur.appearance'

/* ------------------------------------------------------------------ worker */

const worker = new Worker(new URL('./worker.mjs', import.meta.url), { type: 'module' })
const waiting = new Map()
let nextId = 0

worker.onmessage = ({ data: { id, value, error } }) => {
  const { resolve, reject } = waiting.get(id)
  waiting.delete(id)
  if (error !== undefined) reject(error)
  else resolve(value)
}

/** One session method, answered in the order it was asked. */
function call(method, ...args) {
  return new Promise((resolve, reject) => {
    const id = nextId++
    waiting.set(id, { resolve, reject })
    worker.postMessage({ id, method, args })
  })
}

/* ------------------------------------------------------------------ events */

const listeners = {}

/** Announce, as the desktop's `handle.emit` does: `{event, id, payload}`. */
function emit(name) {
  for (const fn of listeners[name] ?? []) fn({ event: name, id: 0, payload: null })
}

/* ----------------------------------------------------------------- storage */

function stored() {
  try {
    return localStorage.getItem(APPEARANCE)
  } catch {
    return null
  }
}

function store(appearance) {
  try {
    localStorage.setItem(APPEARANCE, appearance)
  } catch {
    // A private window or blocked site data: the choice lasts the tab.
  }
}

/* The stored appearance goes over first, so the window's first `status` —
   posted after this, and answered in order — already wears it. */
const appearance = stored()
if (appearance !== null) call('set_appearance', appearance).catch(() => {})

/* ---------------------------------------------------------------- debounce */

/*
  **The desktop's typing loop, as a timer.** `edit` stores the text and
  compiles nothing; the compile falls due one quiet interval after the last
  keystroke, and `rendered` follows it. A command that reads what the pane
  holds — a save, a Save-as, an export — flushes a compile still waiting first,
  since `saveDocument` sends `edit` and `save` back to back.
*/
let typing = null

async function compile() {
  await call('compile')
  emit('rendered')
}

function flush() {
  if (typing === null) return Promise.resolve()
  clearTimeout(typing)
  typing = null
  return compile()
}

/* --------------------------------------------------------------- downloads */

/** The name a path ends in, `/`-separated as every path here is. */
function basename(path) {
  return String(path ?? '').split('/').pop()
}

/** The file name a download token carries. */
function tokened(path) {
  return String(path).replace(/^download:/, '')
}

/** Hand the reader a file: the browser's only way to write outside the tab. */
function download(name, bytes, type) {
  const url = URL.createObjectURL(new Blob([bytes], { type }))
  const link = document.createElement('a')
  link.href = url
  link.download = name
  link.hidden = true
  document.body.append(link)
  link.click()
  link.remove()
  setTimeout(() => URL.revokeObjectURL(url), 60_000)
}

/** The bytes a `Uint8Array` holds, as the `ArrayBuffer` Tauri's `Response` hands the page. */
function buffer(bytes) {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
}

/* ---------------------------------------------------------------- commands */

/* The session calls that change something the window draws. Each announces
   after it succeeds, where the desktop's command or its watch loop did; a
   refusal reaches the window's `fail` and announces nothing, as on desktop. */
const ANNOUNCED = new Set([
  'open_document',
  'set_main',
  'set_edited',
  'trash_file',
  'discard',
  'save',
  'create_file'
])

/* The session calls that change nothing and go straight through. */
const READS = new Set(['document_text', 'asset_bytes', 'save_as_path'])

async function invoke(command, args = {}) {
  if (READS.has(command)) {
    const value = await call(command, ...Object.values(args))
    return value instanceof Uint8Array ? buffer(value) : value
  }
  if (ANNOUNCED.has(command)) {
    if (command === 'save') await flush()
    const value = await call(command, ...Object.values(args))
    emit('rendered')
    return value ?? null
  }

  switch (command) {
    case 'status': {
      const status = JSON.parse(await call('status'))
      // The desktop sets the title from Rust, to the pane's file name.
      if (status.edited !== null) document.title = basename(status.edited)
      return status
    }
    case 'current_pdf':
      return buffer(await call('current_pdf'))
    case 'edit':
      await call('edit', args.text)
      clearTimeout(typing)
      typing = setTimeout(() => {
        typing = null
        compile()
      }, TYPING_DEBOUNCE)
      return null
    case 'save_as': {
      await flush()
      const { bytes, receipt } = await call('save_as', args.path)
      download(tokened(args.path), bytes, 'text/plain')
      emit('rendered')
      return receipt
    }
    case 'export_path':
      await flush()
      return call('export_path')
    case 'export': {
      await flush()
      const { bytes } = await call('export', args.path)
      download(tokened(args.path), bytes, 'application/pdf')
      emit('rendered')
      return null
    }
    case 'set_appearance':
      store(args.appearance)
      await call('set_appearance', args.appearance)
      emit('rendered')
      return null
    case 'pending_open': {
      await seeded
      const path = slot
      slot = null
      return path
    }
    case 'fetch_images':
      throw 'images by URL are not fetched in a browser'
    default:
      throw `${command} is not answered in a browser`
  }
}

/* ----------------------------------------------------------------- dialogs */

/*
  **Open… is a file input the host owns.** Its one file is staged alone and
  answered as a path in the project; a cancel answers `null`, as the desktop's
  panel does. `ltr-001` Phase 3 replaces it with the project sheet.
*/
let input = null

function open() {
  if (input === null) {
    input = document.createElement('input')
    input.type = 'file'
    input.accept = '.md'
    input.hidden = true
    document.body.append(input)
  }
  input.value = ''
  return new Promise((resolve) => {
    const settle = async () => {
      input.onchange = input.oncancel = null
      const file = input.files?.[0]
      if (file === undefined) return resolve(null)
      await call('unstage')
      await call('stage', PROJECT + file.name, new Uint8Array(await file.arrayBuffer()))
      resolve(PROJECT + file.name)
    }
    input.onchange = settle
    input.oncancel = settle
    input.click()
  })
}

/*
  **Save-as and export are downloads.** The window's flows are fixed — a path
  from Rust, the dialog, then the command with what the dialog answered — so
  the dialog answers a token, `download:<name>`, and the command that receives
  it produces the bytes. The refusals stay Rust's; only the delivery is here.
*/
async function save({ defaultPath } = {}) {
  const name = prompt('Save a copy as', basename(defaultPath))
  return name === null || name === '' ? null : `download:${name}`
}

/* ---------------------------------------------------------------- keyboard */

/*
  **The desktop menu's accelerators**, since the window keeps no `keydown` of
  its own for them. `metaKey` on macOS and `ctrlKey` elsewhere. ⌘L is the
  browser's address bar, so `view-lines` is not bound; its button is in the
  footer.

  **⇧⌘E is this host's own**, and the one chord here the desktop menu does not
  carry. Export is a menu item on the desktop with no accelerator and no button
  in the window, and a tab has no menu — so without a chord a reader could not
  export at all. `ltr-001` §2's table, corrected.
*/
const MAC = /Mac|iPhone|iPad/.test(navigator.platform)

window.addEventListener(
  'keydown',
  (event) => {
    if (!(MAC ? event.metaKey : event.ctrlKey) || event.altKey) return
    const key = event.key.toLowerCase()
    const name =
      key === 'o' && !event.shiftKey ? 'open'
      : key === 's' && event.shiftKey ? 'save-as'
      : key === 's' ? 'save'
      : key === 'e' && event.shiftKey ? 'export'
      : key === 'b' && !event.shiftKey ? 'view-files'
      : null
    if (name === null) return
    event.preventDefault()
    if (!event.repeat) emit(name)
  },
  true
)

/* -------------------------------------------------------------------- seed */

/*
  **An example opens as a project.** `#example=NAME` names a row of the landing
  page, which stays the one copy of every example: the element is fetched from
  it and opened as `document.md`, beside the page's two files under their own
  `data-asset` names. The bytes are the element's text, encoded — the same
  bytes the landing page's test compiles.

  With no hash the seed is the first example; an unknown name seeds nothing,
  and the window starts empty with Open… offered, as a desktop launch does.
*/
let slot = null
let seeded = seed(location.hash)

function named(hash) {
  const found = /^#example=(.*)$/.exec(hash)
  if (found === null) return hash === '' || hash === '#' ? 'caption-table' : null
  try {
    return decodeURIComponent(found[1])
  } catch {
    return null
  }
}

async function seed(hash) {
  slot = null
  const name = named(hash)
  if (name === null) return
  try {
    const response = await fetch(new URL('../index.html', location))
    if (!response.ok) return
    const page = new DOMParser().parseFromString(await response.text(), 'text/html')
    const example = page.querySelector(`script[data-example="${CSS.escape(name)}"]`)
    if (example === null) return

    const encode = (text) => new TextEncoder().encode(text)
    await call('unstage')
    await call('stage', DOCUMENT, encode(example.textContent))
    for (const asset of page.querySelectorAll('script[data-asset]')) {
      await call('stage', PROJECT + asset.dataset.asset, encode(asset.textContent))
    }
    slot = DOCUMENT
  } catch (problem) {
    console.error('the example could not be read:', problem)
  }
}

window.addEventListener('hashchange', () => {
  seeded = seed(location.hash).then(() => {
    if (slot !== null) emit('opened')
  })
})

/* ------------------------------------------------------------------ window */

window.__TAURI__ = {
  core: { invoke },
  dialog: { open, save },
  event: {
    listen: async (name, fn) => {
      ;(listeners[name] ??= []).push(fn)
      return () => {
        listeners[name] = listeners[name].filter((other) => other !== fn)
      }
    }
  }
}
