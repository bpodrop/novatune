import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = fileURLToPath(new URL('..', import.meta.url))
const manifestPath = path.join(projectRoot, 'public/manifest.webmanifest')
const swPath = path.join(projectRoot, 'public/sw.js')
const indexPath = path.join(projectRoot, 'index.html')
const mainPath = path.join(projectRoot, 'src/main.tsx')

function assertTruthy(value, message) {
  if (!value) {
    throw new Error(message)
  }
}

async function main() {
  const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'))

  assertTruthy(manifest.name, 'manifest.name is required')
  assertTruthy(manifest.short_name, 'manifest.short_name is required')
  assertTruthy(manifest.id, 'manifest.id is required')
  assertTruthy(manifest.start_url, 'manifest.start_url is required')
  assertTruthy(manifest.scope, 'manifest.scope is required')
  assertTruthy(manifest.display, 'manifest.display is required')
  assertTruthy(manifest.theme_color, 'manifest.theme_color is required')
  assertTruthy(manifest.background_color, 'manifest.background_color is required')
  assertTruthy(Array.isArray(manifest.icons) && manifest.icons.length > 0, 'manifest.icons is required')

  const has192Icon = manifest.icons.some((icon) => icon.sizes === '192x192')
  const has512Icon = manifest.icons.some((icon) => icon.sizes === '512x512')
  const hasMaskableIcon = manifest.icons.some((icon) => icon.purpose === 'maskable')
  assertTruthy(has192Icon, 'manifest must include a 192x192 icon')
  assertTruthy(has512Icon, 'manifest must include a 512x512 icon')
  assertTruthy(hasMaskableIcon, 'manifest must include at least one maskable icon')

  for (const icon of manifest.icons) {
    const iconPath = path.join(projectRoot, 'public', icon.src.replace(/^\//, ''))
    await fs.access(iconPath)
  }

  const indexHtml = await fs.readFile(indexPath, 'utf8')
  assertTruthy(indexHtml.includes('rel="manifest"'), 'index.html missing manifest link')
  assertTruthy(indexHtml.includes('name="theme-color"'), 'index.html missing theme-color meta tag')
  assertTruthy(
    indexHtml.includes('name="apple-mobile-web-app-capable"'),
    'index.html missing apple-mobile-web-app-capable meta tag',
  )
  assertTruthy(
    indexHtml.includes('name="apple-mobile-web-app-title"'),
    'index.html missing apple-mobile-web-app-title meta tag',
  )
  assertTruthy(
    indexHtml.includes('name="apple-mobile-web-app-status-bar-style"'),
    'index.html missing apple-mobile-web-app-status-bar-style meta tag',
  )

  const swSource = await fs.readFile(swPath, 'utf8')
  assertTruthy(swSource.includes("self.addEventListener('install'"), 'sw.js missing install handler')
  assertTruthy(swSource.includes("self.addEventListener('activate'"), 'sw.js missing activate handler')
  assertTruthy(swSource.includes("self.addEventListener('fetch'"), 'sw.js missing fetch handler')

  const mainSource = await fs.readFile(mainPath, 'utf8')
  assertTruthy(
    mainSource.includes("navigator.serviceWorker.register('/sw.js')"),
    'src/main.tsx missing service worker registration',
  )

  process.stdout.write('PWA validation passed.\n')
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
