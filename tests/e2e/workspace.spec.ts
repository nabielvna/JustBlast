import { test, expect } from './native'

test('board edits save immediately and retain accepted input on reload', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await page.goto('/')
  const settings = page.getByRole('button', { name: 'Settings', exact: true })
  await expect(settings).toBeEnabled()
  await expect(page.locator('.board-cell')).toHaveCount(64)
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeDisabled()
  await settings.click()
  await expect(page.getByRole('combobox', { name: 'Mode', exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  const cell = page.getByRole('button', { name: 'Board row 8, column 8', exact: true })
  await cell.click()
  await expect(cell).toHaveAttribute('aria-pressed', 'true')
  await expect(page.getByText('Review input', { exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Edit input', exact: true })).toHaveCount(0)
  await page.reload()
  await expect(cell).toHaveAttribute('aria-pressed', 'true')
  await page.getByRole('button', { name: 'Clear board', exact: true }).click()
  await expect(page.locator('.board-cell.is-filled')).toHaveCount(0)
  expect(errors).toEqual([])
})

test('responsive dashboard and Settings stay inside the viewport', async ({ page }) => {
  for (const width of [1440, 1024, 760, 390]) {
    await page.setViewportSize({ width, height: 900 })
    await page.goto('/')
    const settings = page.getByRole('button', { name: 'Settings', exact: true })
    await expect(settings).toBeEnabled()
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
    await settings.click()
    const dialog = page.getByRole('dialog', { name: 'Settings', exact: true })
    await expect(dialog).toBeVisible()
    const bounds = await dialog.boundingBox()
    expect(bounds!.x).toBeGreaterThanOrEqual(0)
    expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width)
    await page.getByRole('tab', { name: 'Automation', exact: true }).click()
    await expect(
      page
        .getByRole('tabpanel', { name: 'Automation' })
        .getByRole('switch', { name: 'Move device', exact: true }),
    ).toBeEnabled()
    await page.keyboard.press('Escape')
    await expect(dialog).not.toBeVisible()
  }
})
