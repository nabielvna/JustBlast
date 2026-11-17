import { existsSync, readFileSync, readdirSync } from 'node:fs'
import path from 'node:path'
import { spawn, spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('..', import.meta.url))
const tools = path.join(root, '.tools')
const executable = (name) => (process.platform === 'win32' ? `${name}.exe` : name)
const env = { ...process.env }
const bins = []
for (const folder of ['node/bin', 'bun/bin', 'cargo/bin', 'sysroot/usr/bin']) {
  const directory = path.join(tools, folder)
  if (existsSync(directory)) bins.push(directory)
}
env.PATH = [...bins, env.PATH].join(path.delimiter)
const requiredBun = readFileSync(path.join(root, '.bun-version'), 'utf8').trim()
const localBun = path.join(tools, 'bun/bin', executable('bun'))
if (process.versions.bun && process.versions.bun !== requiredBun) {
  if (!existsSync(localBun)) {
    console.error(`Install Bun ${requiredBun}. See ../just-blast-documentation/README.md.`)
    process.exit(1)
  }
  const result = spawnSync(localBun, [fileURLToPath(import.meta.url), ...process.argv.slice(2)], {
    env,
    stdio: 'inherit',
  })
  if (result.error) console.error(result.error.message)
  process.exit(result.status ?? 1)
}
if (existsSync(path.join(tools, 'rustup'))) {
  env.RUSTUP_HOME = path.join(tools, 'rustup')
  env.CARGO_HOME = path.join(tools, 'cargo')
}
if (process.platform === 'linux' && existsSync(path.join(tools, 'sysroot'))) {
  env.PKG_CONFIG_PATH = [
    path.join(tools, 'sysroot/usr/lib/x86_64-linux-gnu/pkgconfig'),
    path.join(tools, 'sysroot/usr/share/pkgconfig'),
    env.PKG_CONFIG_PATH,
  ]
    .filter(Boolean)
    .join(path.delimiter)
}

const node = existsSync(path.join(tools, 'node/bin', executable('node')))
  ? path.join(tools, 'node/bin', executable('node'))
  : executable('node')
const version = spawnSync(node, ['--version'], { env, encoding: 'utf8' })
const requiredNode = readFileSync(path.join(root, '.node-version'), 'utf8').trim()
if (version.error || version.stdout.trim() !== `v${requiredNode}`) {
  console.error(
    `Install Node.js ${requiredNode} and add it to PATH. See ../just-blast-documentation/README.md.`,
  )
  process.exit(1)
}

let child
let interrupted = false
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    interrupted = true
    child?.kill(signal)
  })
}

async function run(command, args) {
  if (interrupted) throw new Error('Interrupted')
  await new Promise((resolve, reject) => {
    child = spawn(command, args, { cwd: root, env, stdio: 'inherit' })
    child.once('error', reject)
    child.once('exit', (code, signal) => {
      child = undefined
      if (code === 0) resolve()
      else reject(new Error(`${command} exited with ${signal ?? code}`))
    })
  })
}

const js = (script, args = []) => run(node, [script, ...args])
const cargo = (args) => run(executable('cargo'), args)
const tauri = (args) => js('node_modules/@tauri-apps/cli/tauri.js', args)
const contracts = (check = false) =>
  cargo([
    'run',
    '--quiet',
    '--locked',
    '--package',
    'justblast-desktop',
    '--no-default-features',
    '--features',
    'export-contracts',
    '--bin',
    'export-contracts',
    ...(check ? ['--', '--check'] : []),
  ])

async function task(name) {
  switch (name) {
    case 'dev':
      return tauri(['dev'])
    case 'ui':
      return js('node_modules/vite/bin/vite.js')
    case 'build-ui':
      return js('node_modules/vite/bin/vite.js', ['build'])
    case 'build':
      return tauri([
        'build',
        '--bundles',
        process.platform === 'win32' ? 'nsis' : 'deb',
        '--',
        '--locked',
      ])
    case 'typecheck':
      await js('node_modules/svelte-check-native/bin/svelte-check-native.js', [
        '--workspace',
        'ui',
        '--tsconfig',
        'tsconfig.json',
        '--svelte-warnings',
        'bridge',
        '--fail-on-warnings',
      ])
      return js('node_modules/typescript/bin/tsc', ['-p', 'tsconfig.tools.json'])
    case 'lint':
      await js('node_modules/oxlint/bin/oxlint', ['--deny-warnings', '.'])
      await js('scripts/check-architecture.mjs')
      return cargo([
        'clippy',
        '--workspace',
        '--no-default-features',
        '--locked',
        '--',
        '-D',
        'warnings',
      ])
    case 'format':
    case 'format-check':
      await js('node_modules/prettier/bin/prettier.cjs', [
        name === 'format' ? '--write' : '--check',
        '.',
      ])
      return cargo(['fmt', '--all', ...(name === 'format-check' ? ['--', '--check'] : [])])
    case 'contracts':
      return contracts()
    case 'contracts-check':
      return contracts(true)
    case 'benchmark':
      await cargo([
        'build',
        '--locked',
        '--release',
        '--package',
        'justblast-core',
        '--example',
        'benchmark',
      ])
      return js('scripts/benchmark.mjs')
    case 'test':
      await run(node, [
        '--test',
        '--test-isolation=none',
        ...readdirSync(path.join(root, 'tests/unit'))
          .filter((name) => /\.test\.(?:ts|mjs)$/.test(name))
          .map((name) => `tests/unit/${name}`),
      ])
      return cargo(['test', '--workspace', '--no-default-features', '--locked'])
    case 'e2e':
      await cargo([
        'build',
        '--package',
        'justblast-desktop',
        '--no-default-features',
        '--features',
        'test-host',
        '--bin',
        'ipc-test-host',
        '--locked',
      ])
      return js('node_modules/@playwright/test/cli.js', ['test'])
    case 'check':
      for (const name of ['typecheck', 'lint', 'format-check', 'contracts-check', 'test'])
        await task(name)
      return
    case 'desktop-test':
      return js('tests/desktop/smoke.mjs')
    default:
      throw new Error(`Unknown task: ${name}`)
  }
}

try {
  await task(process.argv[2])
} catch (error) {
  console.error(error.message)
  process.exitCode = 1
}
