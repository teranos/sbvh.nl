/**
 * sbvh.nl builder
 *
 * Compiles Svelte 5 components via Bun.build() plugin,
 * outputs bundled JS + static assets to dist/
 */

import { compile } from 'svelte/compiler'
import { join } from 'path'
import { rm, mkdir, cp, readdir } from 'fs/promises'

const srcDir = join(import.meta.dir, 'src')
const outDir = join(import.meta.dir, 'dist')
const staticDir = join(import.meta.dir, 'static')

const sveltePlugin: import('bun').BunPlugin = {
  name: 'svelte',
  setup(build) {
    build.onLoad({ filter: /\.svelte$/ }, async (args) => {
      const source = await Bun.file(args.path).text()
      const result = compile(source, {
        filename: args.path,
        generate: 'client',
      })
      let code = result.js.code
      if (result.css && result.css.code) {
        const escaped = result.css.code
          .replaceAll('\\', '\\\\')
          .replaceAll('`', '\\`')
          .replaceAll('$', '\\$')
        code += `\n;(function(){const s=document.createElement('style');s.textContent=\`${escaped}\`;document.head.appendChild(s)})()\n`
      }
      return { contents: code, loader: 'js' }
    })
  },
}

// Clean + build
await rm(outDir, { recursive: true, force: true }).catch(() => {})
await mkdir(outDir, { recursive: true })

const result = await Bun.build({
  entrypoints: [join(srcDir, 'main.ts')],
  outdir: outDir,
  minify: true,
  sourcemap: 'none',
  plugins: [sveltePlugin],
})

if (!result.success) {
  console.error('Build failed:')
  for (const msg of result.logs) console.error(msg)
  process.exit(1)
}

// Copy tokens.css
await Bun.write(
  join(outDir, 'tokens.css'),
  await Bun.file(join(import.meta.dir, 'tokens.css')).text()
)

// Copy static assets
await cp(staticDir, outDir, { recursive: true })

// Copy index.html, rewrite script src
const html = await Bun.file(join(import.meta.dir, 'index.html')).text()
await Bun.write(join(outDir, 'index.html'), html.replace('/src/main.ts', '/main.js'))

// Sitemap with per-URL <lastmod> pulled from git.
// Each entry lists every path whose change should bump that URL's lastmod;
// we take the newest commit time across those paths. Untracked paths return
// empty and drop out — the URL appears without <lastmod>.
const sitemapBase = 'https://sbvh.nl'
const sitemapPages: Array<{ loc: string; sources: string[] }> = [
  { loc: '/',                  sources: ['index.html', 'src', 'tokens.css'] },
  { loc: '/privacy.html',      sources: ['static/privacy.html'] },
  { loc: '/qntx-tos.html',     sources: ['static/qntx-tos.html'] },
  { loc: '/werf/privacy.html', sources: ['static/werf/privacy.html'] },
]

async function gitLastMod(paths: string[]): Promise<string> {
  let latest = ''
  for (const p of paths) {
    const out = (await Bun.$`git log -1 --format=%aI -- ${p}`.cwd(import.meta.dir).text()).trim()
    if (out && out > latest) latest = out
  }
  return latest
}

const urlBlocks: string[] = []
for (const page of sitemapPages) {
  const lastmod = await gitLastMod(page.sources)
  const lastmodTag = lastmod ? `\n    <lastmod>${lastmod}</lastmod>` : ''
  urlBlocks.push(`  <url>\n    <loc>${sitemapBase}${page.loc}</loc>${lastmodTag}\n  </url>`)
}

await Bun.write(
  join(outDir, 'sitemap.xml'),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urlBlocks.join('\n')}\n</urlset>\n`,
)

console.log('sbvh.nl built -> dist/')
