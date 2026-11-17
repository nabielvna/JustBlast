import { test, expect } from './native'
test.use({ fakePhone: true })

async function connect(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('button', { name: 'Discover', exact: true }).click()
  await expect(
    page.getByRole('option', { name: 'Offline phone · device', exact: true }),
  ).toBeAttached()
  await page.getByRole('combobox', { name: 'Device', exact: true }).selectOption('offline-phone')
  await expect(page.getByRole('button', { name: 'Connect', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
  await expect(page.getByText('Phone connected', { exact: true })).toBeVisible()
}
test('Auto scans without moving, then executes acknowledged steps and opens the next plan', async ({
  page,
}) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await connect(page)
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await expect(page.getByRole('switch', { name: 'Confirm each cycle', exact: true })).toHaveCount(0)
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .check()
  await expect(page.getByRole('switch', { name: 'Confirm each cycle', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Done', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Placement plan', exact: true })).toBeVisible({
    timeout: 20000,
  })
  await page.getByRole('button', { name: 'Play on phone', exact: true }).click()
  await expect(page.getByText('Confirmation needed', { exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Next cycle', exact: true })).toBeEnabled({
    timeout: 20000,
  })
  await page.getByRole('button', { name: 'Next cycle', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(3, { timeout: 20000 })
  await expect(page.getByRole('button', { name: 'Play on phone', exact: true })).toBeEnabled()
  expect(errors).toEqual([])
})
test('Move device off keeps Auto playback as a preview', async ({ page }) => {
  await connect(page)
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(2, { timeout: 20000 })
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(2)
  await expect(page.getByRole('button', { name: 'Reset preview', exact: true })).toBeVisible()
  await expect(page.getByText('Confirmation needed', { exact: true })).toHaveCount(0)
})

test('physical progress updates atomically, catches up across cycles and never advances on a timer', async ({
  page,
}) => {
  await connect(page)
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await expect(page.getByRole('switch', { name: 'Animations', exact: true })).toHaveCount(0)
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .check()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(2, { timeout: 20000 })

  await page.evaluate(async () => {
    type View = import('../../ui/src/generated/contracts').WorkspaceView
    const internals = (
      window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> }
      }
    ).__TAURI_INTERNALS__
    const original = internals.invoke
    const initial = (await original('workspace_snapshot')) as View
    const visual = {
      id: 10,
      cycle: initial.cycle,
      revision: initial.revision,
      board: initial.board,
      pieces: initial.pieces,
      plan: initial.plan!,
      completed: 0,
      done: false,
    }
    const current: View = {
      ...initial,
      device: { ...initial.device, busy: true, state: 'running', visual, queuedVisuals: 1 },
    }
    Object.assign(window, { syncTest: { current, failures: [] as string[] } })
    let delivery = 0
    internals.invoke = async (command, args) => {
      if (command !== 'workspace_snapshot' && command !== 'device_request')
        return original(command, args)
      const snapshot = structuredClone(current)
      const currentDelivery = ++delivery
      requestAnimationFrame(() => {
        if (currentDelivery !== delivery) return
        const v = snapshot.device.visual!
        const expected = v.completed ? v.plan.steps[v.completed - 1]!.after : v.board
        const actual = [...document.querySelectorAll('.board .board-cell')]
          .map((cell) => (cell.classList.contains('is-filled') ? '#' : '.'))
          .join('')
        const done = document.querySelectorAll('.plan-row.is-done').length
        const consumed = document.querySelectorAll('.piece-tile.is-consumed').length
        if (actual !== expected.join('') || done !== v.completed || consumed !== v.completed) {
          const probe = (window as unknown as { syncTest: { failures: string[] } }).syncTest
          probe.failures.push(
            `cycle ${v.id}, acknowledged ${v.completed}: board ${actual}, done ${done}, consumed ${consumed}`,
          )
        }
      })
      return snapshot
    }
  })
  await expect(page.locator('.dashboard')).toHaveCSS('--board-motion-duration', '48ms')
  await expect(page.locator('.board-cell').first()).toHaveCSS('transition-duration', '0s')
  await page.getByRole('button', { name: 'Play on phone', exact: true }).click()
  await page.waitForTimeout(180)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(0)

  const acknowledge = async (completed: number, nextCycle = false) =>
    page.evaluate(
      ({ completed, nextCycle }) => {
        const probe = (
          window as unknown as {
            syncTest: { current: import('../../ui/src/generated/contracts').WorkspaceView }
          }
        ).syncTest
        const visual = probe.current.device.visual!
        if (nextCycle) {
          visual.id++
          visual.cycle++
          probe.current.cycle++
        }
        visual.completed = completed
        probe.current.device.completed = completed
        probe.current.device.revision++
      },
      { completed, nextCycle },
    )

  await acknowledge(2)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(2)
  await expect(page.locator('.piece-tile.is-consumed')).toHaveCount(2)
  await acknowledge(0, true)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(0)
  await acknowledge(1)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  await page.waitForTimeout(400)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  const failures = await page.evaluate(
    () => (window as unknown as { syncTest: { failures: string[] } }).syncTest.failures,
  )
  expect(failures).toEqual([])
  await expect(
    page
      .locator('.board-cell:not(.is-filled):not(.is-ghost) .board-cell-label')
      .filter({ hasText: /\d/ }),
  ).toHaveCount(0)
})
