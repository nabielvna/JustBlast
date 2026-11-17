import { test, expect } from './native'
import type { WorkspaceView } from '../../ui/src/generated/contracts'
test.use({ fakePhone: true })
async function snapshot(page: import('@playwright/test').Page) {
  return page.evaluate(async () =>
    (
      window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string) => Promise<WorkspaceView> }
      }
    ).__TAURI_INTERNALS__.invoke('workspace_snapshot'),
  )
}
async function setup(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('button', { name: 'Discover', exact: true }).click()
  await expect(
    page.getByRole('option', { name: 'Offline phone · device', exact: true }),
  ).toBeAttached()
  await page.getByRole('combobox', { name: 'Device', exact: true }).selectOption('offline-phone')
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
  await expect(page.getByText('Phone connected', { exact: true })).toBeVisible()
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .check()
  for (const name of ['Check position after each cycle'])
    await page.getByRole('switch', { name, exact: true }).uncheck()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(2, { timeout: 20000 })
}
test('main and step playback follow Move device and a correction cannot replay the first phone move', async ({
  page,
}) => {
  test.setTimeout(60000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await setup(page)
  await expect(page.locator('.plan-row-action')).toHaveCount(2)
  await expect(page.locator('.plan-row-action svg')).toHaveCount(2)
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .uncheck()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Play on phone', exact: true })).toHaveCount(0)
  await expect(page.locator('.console-controls')).toHaveCount(1)
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(2)
  expect((await snapshot(page)).completed).toBe(0)
  await page.getByRole('button', { name: 'Reset preview', exact: true }).click()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .uncheck()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await expect(page.locator('.plan-row-action')).toHaveCount(2)
  await expect(
    page.getByRole('button', { name: 'Play through step 1 on phone', exact: true }),
  ).toHaveCount(0)
  await page.getByRole('button', { name: 'Preview through step 1', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  expect((await snapshot(page)).completed).toBe(0)
  await page.getByRole('button', { name: 'Reset preview', exact: true }).click()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await page
    .getByRole('tabpanel', { name: 'Automation' })
    .getByRole('switch', { name: 'Move device', exact: true })
    .check()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Play through step 1 on phone', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1, { timeout: 20000 })
  await expect(page.locator('.plan-row').first().locator('.plan-row-select')).toBeDisabled()
  await expect(page.locator('.plan-row').nth(1)).toHaveClass(/is-selected/)
  await expect(
    page.getByRole('button', { name: 'Phone moves locked', exact: true }),
  ).toHaveAttribute('aria-disabled', 'true')
  await expect(page.getByRole('button', { name: 'Reset preview', exact: true })).toHaveCount(0)
  await page.getByRole('switch', { name: 'Move device', exact: true }).click()
  await expect(
    page.getByRole('button', { name: 'Phone moves locked', exact: true }),
  ).toHaveAttribute('aria-disabled', 'true')
  await expect(page.locator('.plan-row').first().locator('.plan-row-select')).toBeDisabled()
  await page.getByRole('switch', { name: 'Move device', exact: true }).click()
  const before = await snapshot(page)
  await page.locator('button.piece-tile:not(.is-consumed)').first().click()
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(page.getByRole('region', { name: 'Input draft', exact: true })).toHaveCount(0)
  await expect(page.locator('.piece-tile.is-consumed')).toHaveCount(1)
  await expect(page.locator('button.piece-tile.is-consumed')).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Play on phone', exact: true })).toBeEnabled()
  const corrected = await snapshot(page)
  expect(corrected.source).toBe('corrected')
  expect(corrected.completed).toBe(1)
  expect(corrected.plan!.steps[0]).toEqual(before.plan!.steps[0])
  await expect(
    page.getByRole('button', { name: 'Play through step 1 on phone', exact: true }),
  ).toBeDisabled()
  await page.getByRole('button', { name: 'Play on phone', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Next cycle', exact: true })).toBeEnabled({
    timeout: 20000,
  })
  expect((await snapshot(page)).completed).toBe(2)
  expect(errors).toEqual([])
})
test('scan directly fills the regular slots, replaces a draft, and keeps correction available', async ({
  page,
}, info) => {
  test.setTimeout(60000)
  await setup(page)
  const cell = page.getByRole('button', { name: 'Board row 8, column 8', exact: true })
  await cell.click()
  const before = await snapshot(page)
  await expect(cell).toHaveAttribute('aria-pressed', 'true')
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect
    .poll(async () => (await snapshot(page)).device.capture?.id, { timeout: 20000 })
    .not.toBe(before.device.capture?.id)
  await expect(page.getByRole('region', { name: 'Input draft', exact: true })).toHaveCount(0)
  await expect(page.locator('.plan-row')).toHaveCount(2)
  await expect(page.getByRole('region', { name: 'Review new scan', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Use new Scan', exact: true })).toHaveCount(0)
  expect((await snapshot(page)).currentBoard[7][7]).toBe('.')
  expect((await snapshot(page)).completed).toBe(0)
  for (const width of [1440, 1024, 760, 390]) {
    await page.setViewportSize({ width, height: 900 })
    await page.getByRole('button', { name: /^Edit Slot 1,/ }).click()
    await page.getByRole('button', { name: 'Phone position', exact: true }).click()
    const dialog = page.getByRole('dialog', { name: 'Edit Slot 1', exact: true })
    await expect(
      page.getByRole('button', { name: 'Set grid origin for slot 1', exact: true }),
    ).toBeVisible()
    const bounds = await dialog.boundingBox()
    expect(bounds!.x).toBeGreaterThanOrEqual(0)
    expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width)
    await expect(page.getByLabel('Piece origin Y', { exact: true })).toBeVisible()
    expect(
      await page
        .locator('.piece-editor-layout')
        .evaluate((el) => el.scrollHeight <= el.clientHeight),
    ).toBe(true)
    await page.screenshot({ path: info.outputPath(`crop-editor-${width}.png`), fullPage: true })
    await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  }
  expect((await snapshot(page)).source).toBe('observed')
})
