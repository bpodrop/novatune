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
const readmePath = path.join(projectRoot, 'README.md')
const metadataPath = path.join(projectRoot, 'src/tuner/wasm/pinned-source.json')

function extract(readme, pattern, label) {
  const match = readme.match(pattern)
  if (!match) {
    throw new Error(`Missing README ${label} line in WASM provenance section`)
  }

  return match[1]
}

async function main() {
  const [readme, metadataRaw] = await Promise.all([
    fs.readFile(readmePath, 'utf8'),
    fs.readFile(metadataPath, 'utf8'),
  ])

  const metadata = JSON.parse(metadataRaw)
  const readmeTag = extract(
    readme,
    /- `tuner-libs` release tag: `([^`]+)`/,
    'release tag',
  )
  const readmeRepo = extract(
    readme,
    /- `tuner-libs` repository: `([^`]+)`/,
    'repository',
  )
  const readmeCommit = extract(
    readme,
    /- Pinned commit: `([^`]+)`/,
    'pinned commit',
  )
  const readmeAsset = extract(
    readme,
    /- Asset name: `([^`]+)`/,
    'asset name',
  )

  if (readmeRepo !== PINNED_LIBS_REPO) {
    throw new Error(
      `README repo mismatch: expected ${PINNED_LIBS_REPO}, found ${readmeRepo}`,
    )
  }

  if (readmeTag !== PINNED_LIBS_TAG) {
    throw new Error(
      `README tag mismatch: expected ${PINNED_LIBS_TAG}, found ${readmeTag}`,
    )
  }

  if (readmeCommit !== PINNED_LIBS_COMMIT) {
    throw new Error(
      `README commit mismatch: expected ${PINNED_LIBS_COMMIT}, found ${readmeCommit}`,
    )
  }

  if (readmeAsset !== PINNED_ASSET_NAME) {
    throw new Error(
      `README asset mismatch: expected ${PINNED_ASSET_NAME}, found ${readmeAsset}`,
    )
  }

  if (metadata.pinnedCommit !== PINNED_LIBS_COMMIT) {
    throw new Error(
      `Metadata pinnedCommit mismatch: expected ${PINNED_LIBS_COMMIT}, found ${metadata.pinnedCommit}`,
    )
  }

  if (metadata.pinnedTag !== PINNED_LIBS_TAG) {
    throw new Error(
      `Metadata pinnedTag mismatch: expected ${PINNED_LIBS_TAG}, found ${metadata.pinnedTag}`,
    )
  }

  if (metadata.pinnedRepo !== PINNED_LIBS_REPO) {
    throw new Error(
      `Metadata pinnedRepo mismatch: expected ${PINNED_LIBS_REPO}, found ${metadata.pinnedRepo}`,
    )
  }

  process.stdout.write('README pin information matches pinned WASM metadata.\n')
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
