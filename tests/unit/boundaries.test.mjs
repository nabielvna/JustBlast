import assert from 'node:assert/strict'
import { test } from 'node:test'
import { checkImports } from '../../scripts/boundaries.mjs'

test('dependency boundaries reject static, dynamic, re-exported and CommonJS bypasses', () => {
  for (const source of [
    "import { invoke } from '@tauri-apps/api/core'",
    "export { invoke } from '@tauri-apps/api/core'",
    "export * from '@tauri-apps/api/core'",
    "const ipc = import('@tauri-apps/api/core')",
    "const ipc = require('@tauri-apps/api/core')",
    'const ipc = import(somePath)',
    "import fs from 'node:fs'",
    "import { createWorkspace } from '../../features/workspace/controller.svelte'",
  ])
    assert.notDeepEqual(checkImports('ui/src/components/ui/Example.ts', source), [], source)
  assert.deepEqual(
    checkImports('ui/src/lib/native/example.ts', "import { invoke } from '@tauri-apps/api/core'"),
    [],
  )
})

test('Svelte scripts keep import boundaries and templates reject raw HTML', () => {
  assert.notDeepEqual(
    checkImports(
      'ui/src/components/ui/Example.svelte',
      '<script lang="ts">import { invoke } from "@tauri-apps/api/core"</script><p>Hello</p>',
    ),
    [],
  )
  assert.notDeepEqual(
    checkImports(
      'ui/src/components/ui/Example.svelte',
      '<script lang="ts">let html: string = ""</script>{@html html}',
    ),
    [],
  )
  assert.deepEqual(
    checkImports(
      'ui/src/components/ui/Example.svelte',
      '<script lang="ts">let { value }: { value: number } = $props()</script><p>{value}</p>',
    ),
    [],
  )
})
