import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFileSync, readdirSync } from 'node:fs'
import path from 'node:path'
import { checkImports } from './boundaries.mjs'

const core = readFileSync('crates/justblast-core/Cargo.toml', 'utf8')
assert(!/\[(?:.*\.)?(?:build-)?dependencies\]/.test(core), 'Core must remain dependency-free')

function files(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const name = path.join(directory, entry.name)
    return entry.isDirectory() ? files(name) : [name]
  })
}
for (const file of files('crates/justblast-core/src')) {
  const source = readFileSync(file, 'utf8')
  assert(
    !/\b(?:tauri|serde|tokio)::|std::(?:fs|net|process|thread|path|env)\b/.test(source),
    `${file}: domain/services must stay pure`,
  )
}
for (const file of [...files('ui/src'), ...files('src-tauri/src'), ...files('crates')]) {
  if (!/\.(rs|ts|svelte)$/.test(file)) continue
  const source = readFileSync(file, 'utf8')
  assert(
    !/just-blast-legacy|bb-legacy|block_blaster|python[23]?\b/.test(source),
    `${file}: production code cannot use the old runtime`,
  )
  if (file.startsWith(`ui${path.sep}src${path.sep}`) && /\.(ts|svelte)$/.test(file)) {
    assert.deepEqual(checkImports(file, source), [], `${file}: dependency boundaries failed`)
  }
}
const baseline = {
  ...JSON.parse(readFileSync('contracts/design-baseline.json', 'utf8')),
  ...JSON.parse(readFileSync('contracts/native-assets.json', 'utf8')),
}
for (const [file, expected] of Object.entries(baseline)) {
  const digest = createHash('sha256').update(readFileSync(file)).digest('hex')
  assert.equal(digest, expected.sha256, `${file}: approved presentation styles changed`)
}
console.log('Architecture boundaries and presentation baseline passed')
