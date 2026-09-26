/*
  The worker the web session lives in — `ltr-001` Phase 2.

  **Everything that compiles runs here**, because a compile is `md2pdf-core`
  calling Typst, and on the main thread that is a frozen caret every 300 ms of
  typing. `host.mjs` holds the `__TAURI__` surface and posts each command here
  as `{id, method, args}`; this answers `{id, value}` or `{id, error}`.

  **Messages are answered in the order they arrive**, which the host relies on:
  an `edit` posted before a `compile` is taken before it. Every call into the
  session is synchronous, and the one `await` below is the same promise for all
  of them, so nothing overtakes anything.

  **A refusal crosses as the sentence**, `String(problem)`: the session throws
  plain strings, and the window's `fail` prints exactly what arrives.
*/

import init, { Session } from '../pkg/letur_web.js'

const ready = init().then(() => new Session())

self.onmessage = async ({ data: { id, method, args } }) => {
  const session = await ready
  let value
  try {
    value = session[method](...args)
  } catch (problem) {
    self.postMessage({ id, error: String(problem) })
    return
  }

  // A `Download` is a wasm object with getters; the host wants its two fields.
  if (value !== null && typeof value === 'object' && 'receipt' in value) {
    const download = { bytes: value.bytes, receipt: value.receipt }
    value.free()
    value = download
  }
  const bytes = value instanceof Uint8Array ? value : value?.bytes
  self.postMessage({ id, value }, bytes instanceof Uint8Array ? [bytes.buffer] : [])
}
