import fs from 'node:fs/promises'
import path from 'node:path'

const projectRoot = path.resolve(new URL('.', import.meta.url).pathname, '..')
const manifestPath = path.join(projectRoot, 'public/manifest.webmanifest')
const swPath = path.join(projectRoot, 'public/sw.js')
const indexPath = path.join(projectRoot, 'index.html')

function assertTruthy(value, message) {
  if (!value) {
    throw new Error(message)
  }
}

async function main() {
  const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'))

  assertTruthy(manifest.name, 'manifest.name is required')
  assertTruthy(manifest.short_name, 'manifest.short_name is required')
  assertTruthy(manifest.start_url, 'manifest.start_url is required')
  assertTruthy(manifest.display, 'manifest.display is required')
  assertTruthy(manifest.theme_color, 'manifest.theme_color is required')
  assertTruthy(manifest.background_color, 'manifest.background_color is required')

  const indexHtml = await fs.readFile(indexPath, 'utf8')
  assertTruthy(indexHtml.includes('rel="manifest"'), 'index.html missing manifest link')
  assertTruthy(indexHtml.includes('name="theme-color"'), 'index.html missing theme-color meta tag')

  await fs.access(swPath)

  process.stdout.write('PWA validation passed.\n')
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
