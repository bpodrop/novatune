import fs from 'node:fs/promises'
import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const projectRoot = fileURLToPath(new URL('..', import.meta.url))
const crateDir = path.resolve(projectRoot, '../../crates/tuner-dsp-web')
const outputDir = path.resolve(projectRoot, 'src/tuner/wasm/pkg')

function runOrThrow(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    stdio: 'inherit',
    ...options,
  })

  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed`)
  }
}

async function main() {
  await fs.rm(outputDir, { recursive: true, force: true })
  await fs.mkdir(outputDir, { recursive: true })

  runOrThrow('wasm-pack', [
    'build',
    crateDir,
    '--target',
    'web',
    '--out-dir',
    outputDir,
    '--out-name',
    'tuner_web_bridge',
  ])
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
  process.exitCode = 1
})
