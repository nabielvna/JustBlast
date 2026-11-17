import { test, expect } from './native'
test.use({ fakePhone: 'uncertain' })

test('an unmatched shape stays visible in red, seeds the editor, and differs from an empty slot', async ({
  page,
}, info) => {
  test.setTimeout(60000)
  await page.goto('/')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByRole('button', { name: 'Discover', exact: true }).click()
  await expect(
    page.getByRole('option', { name: 'Offline phone · device', exact: true }),
  ).toBeAttached()
  await page.getByRole('combobox', { name: 'Device', exact: true }).selectOption('offline-phone')
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
  await expect(page.getByText('Phone connected', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  const uncertain = page.getByRole('button', { name: 'Edit Slot 1, Not in library', exact: true })
  await expect(uncertain).toBeVisible({ timeout: 20000 })
  await expect(uncertain).toHaveClass(/is-uncertain/)
  await expect(uncertain).toHaveCSS('border-top-color', 'rgb(210, 160, 163)')
  await expect(uncertain.locator('.piece-shape > i:not(.piece-hole)')).toHaveCount(12)
  await expect(
    page.getByRole('button', { name: 'Edit Slot 3, Empty slot', exact: true }),
  ).not.toHaveClass(/is-uncertain/)
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeDisabled()
  await expect(page.locator('.plan-row')).toHaveCount(0)
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 900 })
    await page.screenshot({ path: info.outputPath(`uncertain-tray-${width}.png`), fullPage: true })
  }
  await expect(page.getByRole('button', { name: 'Accept slot 1', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Accept slot 1', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(2, { timeout: 20000 })
  await expect(page.locator('.piece-tile.is-uncertain')).toHaveCount(0)
  await expect(page.getByText('Review input', { exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Scan', exact: true }).click()
  await expect(uncertain).toBeVisible({ timeout: 20000 })
  await expect(page.locator('.piece-tile.is-uncertain')).toHaveCount(1)
  await uncertain.click()
  await page.getByRole('button', { name: 'Clear piece', exact: true }).click()
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(
    page.getByRole('button', { name: 'Edit Slot 1, Empty slot', exact: true }),
  ).not.toHaveClass(/is-uncertain/)
  await expect(page.locator('.plan-row')).toHaveCount(1, { timeout: 20000 })
})
