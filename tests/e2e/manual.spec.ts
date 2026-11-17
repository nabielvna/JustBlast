import { test, expect } from './native'

async function manual(page: import('@playwright/test').Page) {
  await page.goto('/')
}
async function preset(page: import('@playwright/test').Page, slot: number, name: string) {
  await page.getByRole('button', { name: new RegExp(`^Edit Slot ${slot},`) }).click()
  await page.getByRole('button', { name: `Use ${name}`, exact: true }).click()
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(page.getByRole('dialog')).not.toBeVisible()
}
test('One workspace completes two simulated cycles with an authoritative plan, prefix-preserving alternatives and final board', async ({
  page,
}) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await manual(page)
  await preset(page, 1, 'Line 2')
  await preset(page, 3, 'Corner 3')
  await expect(page.getByRole('heading', { name: 'Placement plan', exact: true })).toBeVisible()
  await expect(page.locator('.plan-row')).toHaveCount(2)
  const first = await page.locator('.plan-row').first().innerText()
  await page.getByRole('button', { name: 'Preview through step 1', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  await expect(page.locator('.plan-row').first().locator('.plan-row-select')).toBeDisabled()
  await expect(page.locator('.plan-row').first().locator('.plan-row-action')).toBeDisabled()
  await expect(page.locator('.plan-row').nth(1)).toHaveClass(/is-selected/)
  await expect(
    page.getByRole('button', { name: 'Explore alternative solutions', exact: true }),
  ).toBeEnabled()
  await page.getByRole('button', { name: 'Explore alternative solutions', exact: true }).click()
  await expect(page.getByRole('button', { name: /^View .*solutions/ })).toBeEnabled({
    timeout: 20_000,
  })
  await page.getByRole('button', { name: /^View .*solutions/ }).click()
  await expect(page.locator('.candidate-row')).toHaveCount(10)
  await page.locator('.candidate-row').nth(1).click()
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Replace', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  const coordinate = first.match(/R\d+ C\d+/)![0]
  await expect(page.locator('.plan-row').first()).toContainText(coordinate)
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(2)
  await expect(page.getByRole('button', { name: 'Continue simulation', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Continue simulation', exact: true }).click()
  await expect(page.locator('.piece-name', { hasText: 'Empty slot' })).toHaveCount(3)
  await expect(page.locator('.board-cell.is-filled')).toHaveCount(5)
  await preset(page, 2, 'Single')
  await expect(page.locator('.plan-row')).toHaveCount(1)
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Continue simulation', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Continue simulation', exact: true }).click()
  await expect(page.locator('.board-cell.is-filled')).toHaveCount(6)
  expect(errors).toEqual([])
})
test('editor transforms, saves and reloads custom shapes without changing accepted input until Apply', async ({
  page,
}) => {
  await manual(page)
  await page.getByRole('button', { name: /^Edit Slot 2,/ }).click()
  await page.getByRole('button', { name: 'Use Corner 3', exact: true }).click()
  await page.getByRole('button', { name: 'Rotate ↻', exact: true }).click()
  await page.getByRole('button', { name: 'Flip ↔', exact: true }).click()
  await page.getByPlaceholder('Shape name', { exact: true }).fill('My corner')
  await page.getByRole('button', { name: 'Save shape', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Use My corner', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  await expect(page.locator('.piece-name', { hasText: 'Empty slot' })).toHaveCount(3)
  await page.reload()
  await page.getByRole('button', { name: /^Edit Slot 2,/ }).click()
  await page.getByRole('button', { name: 'My shapes', exact: true }).click()
  await page.getByRole('button', { name: 'Use My corner', exact: true }).click()
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(
    page.locator('.piece-tile').nth(1).locator('.piece-shape i:not(.piece-hole)'),
  ).toHaveCount(3)
})
test('piece editor remains bounded at all supported widths', async ({ page }) => {
  await manual(page)
  for (const width of [1440, 1024, 760, 390]) {
    await page.setViewportSize({ width, height: 900 })
    await page.getByRole('button', { name: /^Edit Slot 1,/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Edit Slot 1', exact: true })
    await expect(dialog).toBeVisible()
    const box = await dialog.boundingBox()
    expect(box!.x).toBeGreaterThanOrEqual(0)
    expect(box!.x + box!.width).toBeLessThanOrEqual(width)
    await page.keyboard.press('Escape')
    await expect(dialog).not.toBeVisible()
  }
})

test('pause and reset reject a pending next-placement timer', async ({ page }) => {
  await manual(page)
  await preset(page, 1, 'Line 2')
  await preset(page, 3, 'Corner 3')
  await expect(page.locator('.plan-row')).toHaveCount(2)
  await page.clock.install()
  await page.clock.pauseAt(new Date())
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await page.clock.runFor(48)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  await page.getByRole('button', { name: 'Pause preview', exact: true }).click()
  await page.clock.runFor(850)
  await expect(page.locator('.plan-row.is-done')).toHaveCount(1)
  await page.getByRole('button', { name: 'Reset preview', exact: true }).click()
  await expect(page.locator('.plan-row.is-done')).toHaveCount(0)
  await expect(page.locator('.plan-row').first()).toHaveClass(/is-selected/)
  await expect(page.locator('.plan-row').first().locator('.plan-row-select')).toBeEnabled()
  await page.clock.runFor(500)
  await expect(page.locator('.board-cell.is-filled')).toHaveCount(0)
})

test('slot panels fit without container scrolling and preserve the drawing across views', async ({
  page,
}, info) => {
  await page.goto('/')
  await page.getByRole('button', { name: /^Edit Slot 1,/ }).click()
  for (const size of [
    { width: 1440, height: 900 },
    { width: 1024, height: 600 },
    { width: 390, height: 700 },
    { width: 390, height: 560 },
  ]) {
    await page.setViewportSize(size)
    const narrow = size.width <= 600
    if (narrow) await page.getByRole('button', { name: 'Library', exact: true }).click()
    await page.getByRole('button', { name: 'Use Corner 3', exact: true }).click()
    for (const selector of ['.piece-editor', '.piece-editor-layout', '.shape-library']) {
      expect(
        await page.locator(selector).evaluate((el) => el.scrollHeight <= el.clientHeight + 1),
      ).toBe(true)
    }
    const grid = page.locator('.shape-library-grid')
    expect(await grid.evaluate((el) => el.scrollHeight > el.clientHeight)).toBe(true)
    await page.screenshot({ path: info.outputPath(`library-${size.width}-${size.height}.png`) })
    if (narrow) await page.getByRole('button', { name: 'Shape', exact: true }).click()
    await expect(page.locator('.manual-piece-cell[aria-pressed="true"]')).toHaveCount(3)
    expect(
      await page
        .locator('.piece-editor-drawing')
        .evaluate((el) => el.scrollHeight <= el.clientHeight + 1),
    ).toBe(true)
    await expect(page.getByRole('button', { name: 'Flip ↔', exact: true })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Apply to slot', exact: true })).toBeVisible()
    await page.screenshot({ path: info.outputPath(`drawing-${size.width}-${size.height}.png`) })
  }
})
