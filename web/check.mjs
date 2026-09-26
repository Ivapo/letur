/*
  `ltr-001` Phase 2's exit gate: Letur's window in a browser tab, driven.

    bun web/check.mjs            Chromium
    bun web/check.mjs --webkit   WebKit
    bun web/check.mjs --only c   one clause (a letter), for a failure being chased

  **One engine per process**, as `app/harness/checks.mjs` records: a second
  launch in one process hangs. **Each clause on a fresh page**, in a fresh
  context, so no clause leans on what another left behind — the appearance in
  `localStorage` least of all.

  **It drives the site `web/assemble.sh` builds**, served on 127.0.0.1 the way
  Pages serves it: `/app` redirected to `/app/`, and `.wasm` as
  `application/wasm` — by `web/serve.mjs`, which `web/hero.mjs` serves it
  through too. So `web/pkg/` must be built first —
  `cd web && wasm-pack build --target web --release`.

  **The reference compiler is pinned, and so is the graph it was built from.**
  Clause (c) compares the page's bytes with what `md2pdf` writes, and that
  comparison means something only if both came from one dependency graph: the
  `md2pdf-core` version `web/Cargo.lock` resolves, and every package the two
  lockfiles share at the CLI's version. Typst breaks lines with
  `icu_segmenter`'s compiled data, so the engine's version alone is not enough.
  The check refuses to compare until both hold, and says which did not.
*/

import { chromium, webkit } from 'playwright'
import { spawnSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { serve } from './serve.mjs'

const ROOT = resolve(import.meta.dir, '..')
const SITE = join(ROOT, '_site')
const argv = process.argv.slice(2)
const engine = argv.includes('--webkit') ? 'webkit' : 'chromium'
const only = argv.includes('--only') ? argv[argv.indexOf('--only') + 1] : null
const LOAD = 90_000

function die(message) {
  console.error(`check: ${message}`)
  process.exit(2)
}

/* ------------------------------------------------------------- the pinning */

/** Every `[[package]]` of a lockfile, as name → the set of versions. */
function packages(lock) {
  const found = new Map()
  for (const block of readFileSync(lock, 'utf8').split('[[package]]').slice(1)) {
    const name = /name = "([^"]+)"/.exec(block)[1]
    const version = /version = "([^"]+)"/.exec(block)[1]
    if (!found.has(name)) found.set(name, new Set())
    found.get(name).add(version)
  }
  return found
}

function pinned() {
  const ours = packages(join(ROOT, 'web/Cargo.lock'))
  const core = [...(ours.get('md2pdf-core') ?? [])]
  if (core.length !== 1) die(`web/Cargo.lock resolves md2pdf-core ${core.length} ways`)
  const version = core[0]

  const reported = spawnSync('md2pdf', ['--version'], { encoding: 'utf8' })
  if (reported.error) die('no md2pdf on PATH: cargo install --locked md2pdf-cli --version ' + version)
  const said = reported.stdout.trim().split(/\s+/).pop()
  if (said !== version) {
    die(`md2pdf reports ${said}, and web/Cargo.lock resolves md2pdf-core ${version}: ` +
      `cargo install --locked md2pdf-cli --version ${version}`)
  }

  const registry = join(process.env.CARGO_HOME ?? join(homedir(), '.cargo'), 'registry/src')
  const lock = existsSync(registry)
    ? readdirSync(registry).sort()
        .map((index) => join(registry, index, `md2pdf-cli-${version}`, 'Cargo.lock'))
        .find(existsSync)
    : undefined
  if (lock === undefined) die(`no md2pdf-cli-${version}/Cargo.lock under ${registry}`)

  const theirs = packages(lock)
  const differ = [...ours].filter(
    ([name, versions]) => theirs.has(name) && [...versions].some((v) => !theirs.get(name).has(v))
  )
  if (differ.length > 0) {
    die(`web/Cargo.lock and ${lock} differ on ${differ.length} shared packages, ` +
      `among them ${differ.slice(0, 5).map(([name]) => name).join(', ')}`)
  }
  return version
}

/* ---------------------------------------------------------------- the site */

function assemble() {
  const built = spawnSync('bash', [join(ROOT, 'web/assemble.sh')], { encoding: 'utf8' })
  if (built.status !== 0) die(`web/assemble.sh failed:\n${built.stderr}`)
}

/** Gate 4: the app page is the window's page and one inserted line. */
function copied() {
  const source = readFileSync(join(ROOT, 'app/dist/index.html'), 'utf8').split('\n')
  const copy = readFileSync(join(SITE, 'app/index.html'), 'utf8').split('\n')
  const at = copy.findIndex((line) => line === '    <script type="module" src="host/host.mjs"></script>')
  if (at < 0 || copy.length !== source.length + 1) return false
  copy.splice(at, 1)
  return copy.every((line, i) => line === source[i]) && source[at].trim() === '</head>'
}

/* ------------------------------------------------------------ the examples */

const LANDING = readFileSync(join(ROOT, 'web/index.html'), 'utf8')

function unescape(text) {
  return text.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'").replace(/&amp;/g, '&')
}

/** The same element shape `app/tests/page_examples_test.rs` scans for. */
const EXAMPLES = [...LANDING.matchAll(
  /<script type="text\/markdown" data-example="([^"]+)" data-expect="(ok|error)">([\s\S]*?)<\/script>/g
)].map(([, name, expect, source]) => ({
  name,
  expect,
  source,
  message: expect === 'error'
    ? unescape(new RegExp(`<code data-error-for="${name}">([\\s\\S]*?)</code>`).exec(LANDING)[1])
    : null
}))
if (EXAMPLES.length !== 12) die(`web/index.html carries ${EXAMPLES.length} examples, not 12`)

const ASSETS = [...LANDING.matchAll(/<script type="[^"]+" data-asset="([^"]+)">([\s\S]*?)<\/script>/g)]
  .map(([, name, text]) => ({ name, text }))
if (ASSETS.length !== 2) die(`web/index.html carries ${ASSETS.length} assets, not 2`)

/** What the pinned CLI writes for a source, with the page's two files beside it. */
function reference(source) {
  const dir = mkdtempSync(join(tmpdir(), 'letur-check-'))
  writeFileSync(join(dir, 'document.md'), source)
  for (const { name, text } of ASSETS) writeFileSync(join(dir, name), text)
  const run = spawnSync('md2pdf', ['document.md', '-o', 'document.pdf'], { cwd: dir, encoding: 'utf8' })
  if (run.status !== 0) die(`md2pdf refused an ok example:\n${run.stderr}`)
  return readFileSync(join(dir, 'document.pdf'))
}

/** `SWITCHING`, off `project/src/preview.rs`, with its line continuations joined. */
const SWITCHING = /pub const SWITCHING: &str = "([\s\S]*?)";/
  .exec(readFileSync(join(ROOT, 'project/src/preview.rs'), 'utf8'))[1]
  .replace(/\\\n\s*/g, '')

/* ----------------------------------------------------------------- driving */

/*
  The error counters, installed before any script on every page. **Not
  `page.on('pageerror')` alone**: `app/harness/checks.mjs` records it missing
  errors a page-side listener sees.
*/
function counting() {
  window.__checkErrors = []
  addEventListener('error', (event) => window.__checkErrors.push(String(event.message)))
  addEventListener('unhandledrejection', (event) => window.__checkErrors.push(String(event.reason)))
}

async function fresh(browser, path) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 860 } })
  const page = await context.newPage()
  await page.addInitScript(counting)
  page.errors = []
  page.on('dialog', (dialog) => dialog.accept(dialog.defaultValue()))
  if (path !== null) await page.goto(base + path, { waitUntil: 'load' })
  return page
}

const status = (page) => page.evaluate(() => window.__TAURI__.core.invoke('status'))

async function until(page, predicate, arg, timeout = LOAD) {
  await page.waitForFunction(predicate, arg, { timeout, polling: 50 })
}

/**
 * Poll the session's status from here until `accept` holds.
 *
 * **Not `waitForFunction`**: its predicate is synchronous, and an async one
 * answers a promise, which is truthy, so the wait passes at once.
 */
async function polled(page, accept, timeout = LOAD) {
  const deadline = Date.now() + timeout
  for (;;) {
    const now = await status(page)
    if (accept(now)) return now
    if (Date.now() > deadline) throw new Error(`timed out in state ${now.state}, revision ${now.revision}`)
    await wait(50)
  }
}

/** Wait for the session to hold a document in one of `states`. */
function settled(page, states = ['current', 'failed', 'stale']) {
  return polled(page, (now) => states.includes(now.state))
}

async function pdfBytes(page) {
  return Buffer.from(
    await page.evaluate(async () =>
      Array.from(new Uint8Array(await window.__TAURI__.core.invoke('current_pdf')))
    )
  )
}

/** Put the caret at the end of the text pane and type a line there. */
async function typeLine(page, line) {
  await page.evaluate(() => {
    const text = document.getElementById('text')
    text.focus()
    text.selectionStart = text.selectionEnd = text.value.length
  })
  await page.keyboard.type(`\n${line}`)
}

function wait(ms) {
  return new Promise((done) => setTimeout(done, ms))
}

/* ----------------------------------------------------------------- clauses */

const byName = Object.fromEntries(EXAMPLES.map((example) => [example.name, example]))
const app = (name) => `/app/#example=${encodeURIComponent(name)}`
const textIs = (page, source) =>
  until(page, (source) => document.getElementById('text').value === source, source)

function check(condition, message) {
  if (!condition) throw new Error(message)
}

const CLAUSES = {
  async a(browser) {
    const page = await fresh(browser, null)
    const asked = []
    page.on('request', (request) => asked.push(request.url()))
    await page.goto(base + '/', { waitUntil: 'networkidle' })
    const wasm = asked.filter((url) => url.endsWith('.wasm'))
    check(wasm.length === 0, `the landing page requested ${wasm.join(', ')}`)
    const links = await page.$$eval('a.open', (links) => links.map((a) => a.getAttribute('href')))
    check(links.length === 12, `the landing page carries ${links.length} links into the app`)
    return page
  },

  async b(browser) {
    const pages = []
    for (const { name, source } of EXAMPLES) {
      const page = await fresh(browser, '/')
      pages.push(page)
      const href = await page.$eval(`a.open[href="app/#example=${name}"]`, (a) => a.href)
      await page.goto(href, { waitUntil: 'load' })
      await textIs(page, source)
    }
    // The `hashchange` → `opened` path, on an app page that is already open.
    const page = await fresh(browser, app('caption-table'))
    pages.push(page)
    await textIs(page, byName['caption-table'].source)
    await page.evaluate(() => (location.hash = '#example=footnote'))
    await textIs(page, byName.footnote.source)
    return pages
  },

  async c(browser) {
    const pages = []
    for (const { name, source } of EXAMPLES.filter(({ expect }) => expect === 'ok')) {
      const page = await fresh(browser, app(name))
      pages.push(page)
      await settled(page, ['current'])
      await until(page, () => document.getElementById('pages').childElementCount > 0)
      const ours = await pdfBytes(page)
      const theirs = reference(source)
      check(ours.equals(theirs), `${name}: the page's PDF (${ours.length} bytes) is not md2pdf's (${theirs.length})`)
    }
    return pages
  },

  async d(browser) {
    const pages = []
    for (const { name } of EXAMPLES.filter(({ expect }) => expect === 'ok')) {
      const page = await fresh(browser, app(name))
      pages.push(page)
      await settled(page, ['current'])
      const shown = await page.evaluate(() =>
        ['error', 'divergence', 'web'].filter((id) => !document.getElementById(id).hidden)
      )
      check(shown.length === 0, `${name}: ${shown.join(', ')} shown on an ok example`)
    }
    return pages
  },

  async e(browser) {
    const pages = []
    for (const { name, message } of EXAMPLES.filter(({ expect }) => expect === 'error')) {
      const page = await fresh(browser, app(name))
      pages.push(page)
      await settled(page, ['failed', 'stale'])
      await until(page, () => !document.getElementById('error').hidden)
      const said = await page.$eval('#error', (error) => error.textContent)
      check(said === message, `${name}: the error bar reads ${JSON.stringify(said)}`)
    }
    return pages
  },

  async f(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    const before = (await status(page)).revision
    await typeLine(page, 'More text.')
    const typed = Date.now()
    await polled(page, (now) => now.revision > before, 2000).catch(() => {
      throw new Error(`revision still ${before} ${Date.now() - typed} ms after typing`)
    })
    const risen = (await status(page)).revision
    await wait(2000)
    const still = (await status(page)).revision
    check(still === risen, `revision moved from ${risen} to ${still} with no typing`)
    return page
  },

  async g(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    await page.click('#new-toggle')
    await page.fill('#new-name', 'notes.md')
    await page.press('#new-name', 'Enter')
    const row = '#parts li[title="Edit notes.md"]'
    await page.waitForSelector(row, { timeout: 10_000 })
    // The row's controls are drawn only under the pointer, as a reader meets them.
    await page.hover(row)
    await page.click(`${row} button.trash`)
    await page.waitForSelector(row, { state: 'detached', timeout: 10_000 })
    return page
  },

  async h(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    await typeLine(page, 'Saved text.')
    await page.keyboard.press('ControlOrMeta+S')
    await until(page, () => document.getElementById('receipt').textContent === 'saved', null, 10_000)
    return page
  },

  async i(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    await typeLine(page, 'Unsaved text.')
    const edited = (await status(page)).edited
    const [download] = await Promise.all([
      page.waitForEvent('download', { timeout: 10_000 }),
      page.keyboard.press('Shift+ControlOrMeta+S')
    ])
    const name = download.suggestedFilename()
    check(name === 'document.md', `the download is named ${name}`)
    const bytes = readFileSync(await download.path())
    const text = await page.$eval('#text', (text) => text.value)
    check(bytes.equals(Buffer.from(text)), 'the download is not the pane\'s text')
    await until(page, (name) => document.getElementById('receipt').textContent === `downloaded ${name}`, name, 10_000)
    const after = (await status(page)).edited
    check(after === edited, `the pane moved from ${edited} to ${after}`)
    await page.click('#parts li[title="Edit refs.yml"] button.name')
    await until(page, () => !document.getElementById('divergence').hidden, null, 10_000)
    const said = await page.$eval('#divergence-text', (span) => span.textContent)
    check(said === SWITCHING, `the divergence reads ${JSON.stringify(said)}`)
    return page
  },

  async j(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    const [download] = await Promise.all([
      page.waitForEvent('download', { timeout: 10_000 }),
      page.keyboard.press('Shift+ControlOrMeta+E')
    ])
    const name = download.suggestedFilename()
    check(name === 'document.pdf', `the export is named ${name}`)
    const bytes = readFileSync(await download.path())
    check(bytes.equals(await pdfBytes(page)), 'the export is not the page\'s bytes')
    return page
  },

  async k(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    const mine = '# Mine\n\nWritten somewhere else.\n'
    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.click('#open')])
    await chooser.setFiles({ name: 'mine.md', mimeType: 'text/markdown', buffer: Buffer.from(mine) })
    await textIs(page, mine)
    return page
  },

  async l(browser) {
    const page = await fresh(browser, '/app/')
    await textIs(page, byName['caption-table'].source)
    await until(page, () => document.title === 'document.md', null, 10_000)
    return page
  },

  async m(browser) {
    const page = await fresh(browser, app('caption-table'))
    await settled(page, ['current'])
    await page.evaluate(() => window.__TAURI__.core.invoke('set_appearance', { appearance: 'dark' }))
    page.errors.push(...(await page.evaluate(() => window.__checkErrors)))
    await page.reload({ waitUntil: 'load' })
    await settled(page, ['current'])
    const worn = (await status(page)).appearance
    check(worn === 'dark', `after a reload the appearance is ${worn}`)
    return page
  }
}

/* -------------------------------------------------------------------- main */

const version = pinned()
if (!argv.includes('--no-assemble')) assemble()
if (!copied()) die('_site/app/index.html is not app/dist/index.html and one inserted line')

const { url: base, server } = await serve(SITE)
const browser = await (engine === 'webkit' ? webkit : chromium).launch({ headless: true })
console.log(`${engine}, md2pdf ${version}, ${base}`)

let failed = 0
for (const [letter, clause] of Object.entries(CLAUSES)) {
  if (only !== null && letter !== only) continue
  const started = Date.now()
  let pages = []
  let problem = null
  try {
    pages = [(await clause(browser))].flat()
  } catch (error) {
    problem = error.message.split('\n')[0]
  }
  // (n): no clause records an uncaught page error.
  for (const page of pages) {
    const errors = [...page.errors, ...(await page.evaluate(() => window.__checkErrors).catch(() => []))]
    if (errors.length > 0 && problem === null) problem = `(n) uncaught: ${errors.join(' | ')}`
    await page.context().close()
  }
  const took = ((Date.now() - started) / 1000).toFixed(1)
  if (problem === null) console.log(`PASS ${letter}  ${took}s`)
  else {
    failed++
    console.log(`FAIL ${letter}  ${took}s  ${problem}`)
  }
}

await browser.close()
server.close()
console.log(failed === 0 ? `all clauses pass in ${engine}` : `${failed} clause(s) failed in ${engine}`)
process.exit(failed === 0 ? 0 : 1)
