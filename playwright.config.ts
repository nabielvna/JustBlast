import { existsSync } from 'node:fs'
import { defineConfig } from '@playwright/test'

const executablePath =
  process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE ??
  (existsSync('/opt/google/chrome/chrome') ? '/opt/google/chrome/chrome' : undefined)

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 30_000,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 2,
  reporter: 'list',
  snapshotPathTemplate: './test-results/reference/{testFileName}/{arg}{ext}',
  use: {
    baseURL: 'http://127.0.0.1:1420',
    viewport: { width: 1440, height: 1000 },
    trace: 'retain-on-failure',
    launchOptions: { executablePath, args: ['--no-sandbox'] },
  },
  webServer: {
    command: 'bun run dev:ui',
    url: 'http://127.0.0.1:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
})
