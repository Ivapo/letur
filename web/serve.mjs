/*
  The site `web/assemble.sh` builds, served on 127.0.0.1 the way Pages serves
  it: `/app` redirected to `/app/`, a directory answered by its `index.html`,
  and `.wasm` as `application/wasm`. `web/check.mjs` drives the app through it
  and `web/hero.mjs` captures the window through it — `mpdf-006` Phase 6 moved
  it here so the two cannot serve the site two ways.
*/

import { createServer } from 'node:http'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { extname, join, normalize } from 'node:path'

export const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.pdf': 'application/pdf',
  '.woff2': 'font/woff2',
  '.png': 'image/png'
}

/** Serve `site` on an unused port; resolves `{ url, server }`. */
export function serve(site) {
  const server = createServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url, 'http://x').pathname)
    if (path === '/app') {
      response.writeHead(301, { location: '/app/' })
      return response.end()
    }
    let file = normalize(join(site, path))
    if (file.startsWith(site) && existsSync(file) && statSync(file).isDirectory()) {
      file = join(file, 'index.html')
    }
    if (!file.startsWith(site) || !existsSync(file) || statSync(file).isDirectory()) {
      response.writeHead(404)
      return response.end()
    }
    response.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' })
    response.end(readFileSync(file))
  })
  return new Promise((done) =>
    server.listen(0, '127.0.0.1', () => done({ url: `http://127.0.0.1:${server.address().port}`, server }))
  )
}
