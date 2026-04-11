import fs from 'node:fs/promises'
import path from 'node:path'

import {
  PINNED_ASSET_NAME,
  PINNED_LIBS_COMMIT,
  PINNED_LIBS_REPO,
  PINNED_LIBS_TAG,
  projectRootFrom,
} from './wasm-pinned-utils.mjs'

const projectRoot = projectRootFrom(import.meta.url)
const metadataPath = path.join(projectRoot, 'src/tuner/wasm/pinned-source.json')
const pkgDir = path.join(projectRoot, 'src/tuner/wasm/pkg')

async function main() {
  const metadataRaw = await fs.readFile(metadataPath, 'utf8')
  const metadata = JSON.parse(metadataRaw)

  if (metadata.pinnedRepo !== PINNED_LIBS_REPO) {
    throw new Error(
      `Pinned repo mismatch: expected ${PINNED_LIBS_REPO}, found ${metadata.pinnedRepo}`,
    )
  }

  if (metadata.pinnedTag !== PINNED_LIBS_TAG) {
    throw new Error(
      `Pinned tag mismatch: expected ${PINNED_LIBS_TAG}, found ${metadata.pinnedTag}`,
    )
  }

  if (metadata.pinnedCommit !== PINNED_LIBS_COMMIT) {
    throw new Error(
      `Pinned commit mismatch: expected ${PINNED_LIBS_COMMIT}, found ${metadata.pinnedCommit}`,
    )
  }

  if (metadata.pinnedAsset !== PINNED_ASSET_NAME) {
    throw new Error(
      `Pinned asset mismatch: expected ${PINNED_ASSET_NAME}, found ${metadata.pinnedAsset}`,
    )
  }

  const required = [
    'package.json',
    'tuner_web_bridge.js',
    'tuner_web_bridge.d.ts',
    'tuner_web_bridge_bg.wasm',
  ]

  for (const file of required) {
    await fs.access(path.join(pkgDir, file))
  }

  process.stdout.write('Pinned WASM source verification passed.\n')
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
