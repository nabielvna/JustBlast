import { spawn } from 'node:child_process'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { createInterface } from 'node:readline'
import path from 'node:path'
import { test as base, expect } from '@playwright/test'

type Response = { ok: true; value: unknown } | { ok: false; error: unknown }

export const test = base.extend<{ native: void; fakePhone: boolean | 'uncertain' }>({
  fakePhone: [false, { option: true }],
  native: [
    async ({ page, fakePhone }, use) => {
      const storage = await mkdtemp(path.join(tmpdir(), 'justblast-browser-'))
      const host = spawn(
        path.resolve(
          'target/debug',
          process.platform === 'win32' ? 'ipc-test-host.exe' : 'ipc-test-host',
        ),
        fakePhone === 'uncertain'
          ? ['--uncertain-phone', `--store=${storage}`]
          : fakePhone
            ? ['--fake-phone', `--store=${storage}`]
            : [],
        { stdio: ['pipe', 'pipe', 'inherit'] },
      )
      const pending: { resolve: (value: unknown) => void; reject: (reason: unknown) => void }[] = []
      createInterface({ input: host.stdout }).on('line', (line) => {
        const response = JSON.parse(line) as Response
        const request = pending.shift()
        if (response.ok) request?.resolve(response.value)
        else request?.reject(response.error)
      })
      host.on('error', (error) => {
        for (const request of pending.splice(0)) request.reject(error)
      })
      host.on('exit', () => {
        for (const request of pending.splice(0)) request.reject(new Error('Rust test host exited'))
      })
      await page.exposeBinding(
        '__justblastInvoke',
        (_, command: string, args: unknown) =>
          new Promise((resolve, reject) => {
            pending.push({ resolve, reject })
            host.stdin.write(JSON.stringify({ command, args }) + '\n')
          }),
      )
      await page.addInitScript(() => {
        const invoke = (
          window as unknown as {
            __justblastInvoke: (command: string, args: unknown) => Promise<unknown>
          }
        ).__justblastInvoke
        Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke } })
        Object.defineProperty(window, 'isTauri', { value: true })
      })
      try {
        await use()
      } finally {
        host.stdin.end()
        host.kill()
        await rm(storage, { recursive: true, force: true })
      }
    },
    { auto: true },
  ],
})

export { expect }
