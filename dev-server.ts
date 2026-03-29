#!/usr/bin/env bun
/**
 * sbvh.nl dev server
 *
 * Builds the frontend, serves from dist/, watches for
 * source changes with live reload.
 */

import { watch } from 'fs'
import { exec } from 'child_process'
import { promisify } from 'util'
import { join } from 'path'

const execAsync = promisify(exec)

const DEV_PORT = 5173
const distDir = join(import.meta.dir, 'dist')

let isBuilding = false
let buildTimeout: ReturnType<typeof setTimeout> | null = null
const clients: Set<{ write: (data: string) => void }> = new Set()

async function build() {
  if (isBuilding) return
  isBuilding = true
  console.log('Building...')
  try {
    await execAsync('bun run build.ts', { cwd: import.meta.dir })
    console.log('Build complete')
    const msg = 'data: reload\n\n'
    clients.forEach(c => c.write(msg))
  } catch (e: any) {
    console.error('Build failed:', e.stderr || e.message)
  } finally {
    isBuilding = false
  }
}

// Watch src/ for changes
watch(join(import.meta.dir, 'src'), { recursive: true }, (_event, filename) => {
  if (!filename) return
  console.log(`Changed: ${filename}`)
  if (buildTimeout) clearTimeout(buildTimeout)
  buildTimeout = setTimeout(build, 300)
})

// Watch index.html and tokens.css
watch(join(import.meta.dir, 'index.html'), () => {
  if (buildTimeout) clearTimeout(buildTimeout)
  buildTimeout = setTimeout(build, 300)
})
watch(join(import.meta.dir, 'tokens.css'), () => {
  if (buildTimeout) clearTimeout(buildTimeout)
  buildTimeout = setTimeout(build, 300)
})

// Initial build
await build()

const server = Bun.serve({
  port: DEV_PORT,
  async fetch(req) {
    const url = new URL(req.url)

    // SSE for live reload
    if (url.pathname === '/__dev_reload__') {
      return new Response(
        new ReadableStream({
          start(controller) {
            const client = {
              write: (data: string) => controller.enqueue(new TextEncoder().encode(data)),
            }
            clients.add(client)
            req.signal.addEventListener('abort', () => clients.delete(client))
          },
        }),
        { headers: { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' } },
      )
    }

    // Serve index.html at root (inject live reload)
    if (url.pathname === '/' || url.pathname === '') {
      let html = await Bun.file(join(distDir, 'index.html')).text()
      html = html.replace(
        '</body>',
        `<script>
          const es = new EventSource("/__dev_reload__");
          es.onmessage = () => location.reload();
        </script></body>`,
      )
      return new Response(html, { headers: { 'Content-Type': 'text/html' } })
    }

    // Serve static files from dist/
    const file = Bun.file(join(distDir, url.pathname))
    if (await file.exists()) return new Response(file)

    // Fallback to index
    return new Response(Bun.file(join(distDir, 'index.html')), {
      headers: { 'Content-Type': 'text/html' },
    })
  },
})

console.log(`sbvh.nl dev: http://localhost:${server.port}`)
