import { readFile } from 'node:fs/promises'
import { test, expect } from './native'
test.use({ fakePhone: true })

test('calibration owns connection, chooses test targets, verifies movement and reconnects on save', async ({
  page,
}) => {
  test.setTimeout(180000)
  test.slow()
  await page.goto('/')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Settings', exact: true })
  await dialog.getByRole('tab', { name: 'Phone calibration', exact: true }).click()
  await expect(dialog.getByText('Use the default game theme', { exact: false })).toBeVisible({
    timeout: 45000,
  })
  const start = dialog.getByRole('button', { name: 'Start calibration', exact: true })
  await expect(start).toBeDisabled()
  await page.screenshot({ path: 'test-results/calibration-start-desktop.png' })
  await dialog.getByRole('button', { name: 'Find phone', exact: true }).click()
  await expect(start).toBeEnabled()
  await dialog.getByText('Change phone or connection', { exact: true }).click()
  await dialog.getByRole('button', { name: 'Phone connection', exact: true }).click()
  await dialog.getByRole('button', { name: 'Connect', exact: true }).click()
  await dialog.getByRole('tab', { name: 'Phone calibration', exact: true }).click()
  await start.click()
  await expect(
    dialog.getByRole('heading', { name: 'Does the board line up?', exact: true }),
  ).toBeVisible({ timeout: 45000 })
  await expect(dialog.getByText('Board edges weren’t clear.', { exact: false })).toBeVisible()
  await dialog.getByRole('button', { name: '5. Save', exact: true }).click()
  await expect(dialog.getByRole('button', { name: 'Save calibration', exact: true })).toBeDisabled()
  await expect(dialog.getByText('Board and pieces aligned', { exact: false })).not.toBeVisible()
  await dialog.getByRole('button', { name: 'Review board', exact: true }).click()
  await dialog.getByRole('button', { name: '4. Test', exact: true }).click()
  await expect(dialog.getByRole('button', { name: 'Test movement', exact: true })).toBeDisabled()
  await dialog.getByRole('button', { name: 'Adjust board alignment', exact: true }).click()
  await expect(dialog.locator('.calibration-wizard .button-primary:visible')).toHaveCount(1)
  await expect(dialog.getByRole('spinbutton', { name: 'Area X', exact: true })).not.toBeVisible()
  await dialog.getByRole('button', { name: 'Adjust', exact: true }).click()
  const boardOptions = dialog.locator('details:has(> summary:text-is("Adjust board"))')
  const x = dialog.getByRole('spinbutton', { name: 'Area X', exact: true })
  await expect(x).toHaveValue('63')
  await x.fill('64')
  await x.fill('63')
  await boardOptions.getByRole('button', { name: 'Re-read board', exact: true }).click()
  await expect(dialog.locator('.calibration-wizard .button-primary:visible')).toHaveCount(1)
  await expect(
    boardOptions.getByRole('button', { name: 'Re-read board', exact: true }),
  ).toBeEnabled({
    timeout: 45000,
  })
  const boardGrid = dialog.getByRole('button', { name: 'Board area', exact: true })
  await expect(dialog.locator('.calibration-board-zoom')).toBeVisible()
  await boardGrid.focus()
  await boardGrid.press('ArrowRight')
  await expect(x).toHaveValue('64')
  await expect(
    boardOptions.getByRole('button', { name: 'Re-read board', exact: true }),
  ).toBeEnabled({
    timeout: 45000,
  })
  await boardGrid.press('ArrowLeft')
  await expect(x).toHaveValue('63')
  await expect(
    boardOptions.getByRole('button', { name: 'Re-read board', exact: true }),
  ).toBeEnabled({
    timeout: 45000,
  })
  const resize = dialog.getByRole('button', { name: 'Resize selected area', exact: true })
  await resize.scrollIntoViewIfNeeded()
  const handle = (await resize.boundingBox())!
  await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2)
  await page.mouse.down()
  await page.mouse.move(handle.x + handle.width / 2 + 5, handle.y + handle.height / 2 + 5, {
    steps: 3,
  })
  await page.mouse.up()
  const width = dialog.getByRole('spinbutton', { name: 'Area Width', exact: true })
  await expect.poll(async () => Number(await width.inputValue())).toBeGreaterThan(954)
  await expect(
    boardOptions.getByRole('button', { name: 'Re-read board', exact: true }),
  ).toBeEnabled({
    timeout: 45000,
  })
  await width.fill('954')
  await dialog.getByRole('spinbutton', { name: 'Area Height', exact: true }).fill('954')
  await boardOptions.getByRole('button', { name: 'Re-read board', exact: true }).click()
  await expect(
    boardOptions.getByRole('button', { name: 'Re-read board', exact: true }),
  ).toBeEnabled({
    timeout: 45000,
  })
  await dialog.getByText('Adjust board', { exact: true }).click()
  await dialog
    .getByRole('heading', { name: 'Does the board line up?', exact: true })
    .scrollIntoViewIfNeeded()
  await page.screenshot({ path: 'test-results/calibration-board-desktop.png' })
  await dialog.getByRole('button', { name: 'Board looks right', exact: true }).click()
  const firstGrid = dialog.getByRole('button', { name: 'Move slot 1 grid', exact: true })
  await expect(firstGrid).toBeVisible({ timeout: 45000 })
  const firstCell = firstGrid.locator('rect').first()
  await expect.poll(async () => Number(await firstCell.getAttribute('x'))).toBeCloseTo(170, -1)
  expect(Math.abs(Number(await firstCell.getAttribute('y')) - 1808)).toBeLessThanOrEqual(3)
  expect(Math.abs(Number(await firstCell.getAttribute('width')) - 54)).toBeLessThanOrEqual(1.6)
  await expect(
    dialog.getByRole('combobox', { name: 'Grid columns', exact: true }),
  ).not.toBeVisible()
  await dialog.getByRole('button', { name: 'Adjust', exact: true }).click()
  await expect(dialog.getByRole('combobox', { name: 'Grid columns', exact: true })).toHaveValue('1')
  await dialog.getByText('Adjust pieces', { exact: true }).click()
  await page.setViewportSize({ width: 390, height: 844 })
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  await page.screenshot({ path: 'test-results/calibration-tray-grids-mobile.png' })
  await page.setViewportSize({ width: 1440, height: 1000 })
  await dialog.getByRole('button', { name: 'Pieces look right', exact: true }).click()
  await expect(dialog.locator('.calibration-wizard .button-primary:visible')).toHaveCount(1)
  await dialog.getByText('Manual test & adjustments', { exact: true }).click()
  await dialog.getByRole('button', { name: 'Adjust board alignment', exact: true }).click()
  await expect(
    dialog.getByRole('heading', { name: 'Does the board line up?', exact: true }),
  ).toBeVisible()
  await expect(dialog.locator('.calibration-board-zoom')).toBeVisible()
  await dialog.getByRole('button', { name: '4. Test', exact: true }).click()
  const check = dialog.getByRole('button', { name: 'Test movement', exact: true })
  await expect(check).toBeEnabled()
  await expect(
    dialog.getByRole('spinbutton', { name: 'Target row', exact: true }),
  ).not.toBeVisible()
  await check.click()
  await expect(dialog.getByText('Movement looks good', { exact: true })).toBeVisible({
    timeout: 45000,
  })
  await page.screenshot({ path: 'test-results/calibration-preview-desktop.png' })
  await dialog.getByRole('button', { name: 'Continue', exact: true }).click()
  await expect(dialog.getByRole('heading', { name: 'Save calibration', exact: true })).toBeVisible()
  await dialog.getByRole('button', { name: 'Back', exact: true }).click()
  await expect(dialog.getByRole('button', { name: 'Continue', exact: true })).toBeEnabled()
  await dialog.getByRole('button', { name: 'Continue', exact: true }).click()
  await dialog.getByText('Profile options', { exact: true }).click()
  const download = page.waitForEvent('download')
  await dialog.getByRole('button', { name: 'Export HP profile', exact: true }).click()
  const artifact = await download
  const toml = await readFile((await artifact.path())!, 'utf8')
  expect(toml).toContain('[screen]')
  expect(toml).not.toMatch(/weights|combo|rule_context|automation|wireless|serial/)
  await dialog.getByText('Profile options', { exact: true }).click()
  await page.screenshot({ path: 'test-results/calibration-save-desktop.png' })
  await dialog.getByRole('button', { name: 'Save calibration', exact: true }).click()
  await expect(dialog.getByRole('heading', { name: 'You’re ready', exact: true })).toBeVisible({
    timeout: 45000,
  })
  await page.setViewportSize({ width: 390, height: 844 })
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  await expect(dialog.getByRole('button', { name: 'Done', exact: true })).toHaveCount(1)
  await page.screenshot({ path: 'test-results/calibration-mobile.png' })
  await dialog.getByRole('button', { name: 'Calibrate again', exact: true }).click()
  await expect(
    dialog.getByRole('heading', { name: 'Prepare your phone', exact: true }),
  ).toBeVisible()
})
