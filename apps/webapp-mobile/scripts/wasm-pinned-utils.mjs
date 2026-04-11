import { spawnSync } from 'node:child_process'

export const PINNED_LIBS_REPO = 'bpodrop/tuner-libs'
export const PINNED_LIBS_COMMIT = 'b16b62ef39a92b05390f4dfac0f95e76ef308533'
export const PINNED_LIBS_TAG = 'libs-v0.2.0'
export const PINNED_ASSET_NAME = `tuner-web-bridge-pkg-${PINNED_LIBS_TAG}.tar.gz`

export function projectRootFrom(importMetaUrl) {
  return new URL('..', importMetaUrl).pathname
}

export function runOrThrow(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    ...options,
  })

  if (result.status !== 0) {
    const stderr = result.stderr?.trim()
    throw new Error(`${command} ${args.join(' ')} failed${stderr ? `: ${stderr}` : ''}`)
  }

  return result
}
