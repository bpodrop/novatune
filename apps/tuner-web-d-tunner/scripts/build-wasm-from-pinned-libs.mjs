import fs from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

import {
  PINNED_ASSET_NAME,
  PINNED_LIBS_COMMIT,
  PINNED_LIBS_REPO,
  PINNED_LIBS_TAG,
  projectRootFrom,
  runOrThrow,
} from './wasm-pinned-utils.mjs'

const projectRoot = projectRootFrom(import.meta.url)
const wasmOutDir = path.join(projectRoot, 'src/tuner/wasm/pkg')
const metadataPath = path.join(projectRoot, 'src/tuner/wasm/pinned-source.json')

async function main() {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'tuner-web-bridge-'))
  const archivePath = path.join(tempDir, PINNED_ASSET_NAME)

  await fs.mkdir(path.dirname(metadataPath), { recursive: true })
  await fs.rm(wasmOutDir, { recursive: true, force: true })

  runOrThrow('gh', [
    'release',
    'download',
    PINNED_LIBS_TAG,
    '--repo',
    PINNED_LIBS_REPO,
    '--pattern',
    PINNED_ASSET_NAME,
    '--output',
    archivePath,
  ])

  await fs.mkdir(wasmOutDir, { recursive: true })
  runOrThrow('tar', ['-xzf', archivePath, '-C', wasmOutDir])

  await fs.writeFile(
    metadataPath,
    `${JSON.stringify({
      pinnedRepo: PINNED_LIBS_REPO,
      pinnedTag: PINNED_LIBS_TAG,
      pinnedCommit: PINNED_LIBS_COMMIT,
      pinnedAsset: PINNED_ASSET_NAME,
      generatedAt: new Date().toISOString(),
    }, null, 2)}\n`,
    'utf8',
  )

  await fs.rm(tempDir, { recursive: true, force: true })
  process.stdout.write(`Fetched wasm bridge asset ${PINNED_ASSET_NAME} from ${PINNED_LIBS_REPO}@${PINNED_LIBS_TAG}\n`)
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
