import { existsSync, readFileSync, mkdirSync, writeFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
const binary = path.resolve(
  'target/release/examples',
  process.platform === 'win32' ? 'benchmark.exe' : 'benchmark',
)
const cases = JSON.parse(readFileSync('tests/fixtures/benchmark-inputs.json', 'utf8'))
const records = []
for (const { name } of cases) {
  const measurements = []
  for (let run = 0; run < 3; run++) {
    const timed = process.platform === 'linux' && existsSync('/usr/bin/time')
    const result = spawnSync(
      timed ? '/usr/bin/time' : binary,
      timed ? ['-f', '%M', binary, name, 'explore'] : [name, 'explore'],
      { encoding: 'utf8' },
    )
    if (result.error || result.status !== 0) throw result.error ?? new Error(result.stderr)
    const [solve, explore] = result.stdout
      .trim()
      .split('\n')
      .map((line) => line.split('\t'))
    measurements.push({
      solveMs: Number(solve[1]),
      solveVisited: Number(solve[2]),
      solveComplete: solve[5] === 'true',
      exploreMs: Number(explore[1]),
      exploreVisited: Number(explore[2]),
      paths: Number(explore[3]),
      finals: Number(explore[4]),
      exploreComplete: explore[5] === 'true',
      peakRssKiB: timed ? Number(result.stderr.trim()) : null,
    })
  }
  records.push({ name, measurements })
}
const output = {
  platform: process.platform,
  arch: process.arch,
  scope:
    'Isolated Rust solver/enumerator processes. This does not measure whole-application memory or phone timing.',
  records,
}
mkdirSync('test-results', { recursive: true })
writeFileSync('test-results/benchmark.json', JSON.stringify(output, null, 2) + '\n')
console.log(JSON.stringify(output, null, 2))
