import { test, expect } from './native'

test('compact icon controls fit a full cycle without scrolling and provide accessible custom tooltips', async ({
  page,
}, info) => {
  test.setTimeout(60000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.goto('/')
  const row = page.getByRole('group', { name: 'Workspace actions', exact: true })
  const settings = row.getByRole('button', { name: 'Settings', exact: true })
  await expect(settings).toBeEnabled()
  await row.getByRole('button', { name: 'Scan', exact: true }).hover()
  await expect(page.getByRole('tooltip')).toContainText('Connect your phone')
  const preview = row.getByRole('button', { name: 'Preview', exact: true })
  await expect(preview).toBeDisabled()
  await preview.focus()
  await expect(page.getByRole('tooltip')).toHaveCount(1)
  await expect(page.getByRole('tooltip')).toContainText('Calculate a plan first')
  await expect(preview).toHaveAttribute(
    'aria-describedby',
    (await page.getByRole('tooltip').getAttribute('id')) ?? '',
  )
  await page.keyboard.press('Enter')
  await expect(page.locator('.plan-row')).toHaveCount(0)
  await page.keyboard.press('Escape')
  await expect(page.getByRole('tooltip')).toHaveCount(0)
  await settings.focus()
  await expect(page.getByRole('tooltip')).toContainText('Move device')
  await page.keyboard.press('Tab')
  await expect(page.getByRole('tooltip')).toHaveCount(0)
  await page.getByRole('button', { name: /^Edit Slot 1,/ }).click()
  await page.getByRole('button', { name: 'Use Line 2', exact: true }).click()
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(preview).toBeEnabled()
  await preview.click()
  await expect(row.getByRole('button', { name: 'Continue simulation', exact: true })).toBeEnabled()
  await expect(row.getByRole('button')).toHaveCount(6)
  for (const width of [1440, 1024, 760, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await page.getByRole('button', { name: 'Settings', exact: true }).blur()
    await page.mouse.move(0, 0)
    await row.evaluate(async (el) => {
      await Promise.all(el.getAnimations({ subtree: true }).map((a) => a.finished))
    })
    const layout = await row.evaluate((el) => ({
      fits: el.scrollWidth <= el.clientWidth,
      buttons: [...el.querySelectorAll('button')].map((b) => {
        const r = b.getBoundingClientRect()
        return {
          x: r.x,
          y: r.y,
          width: r.width,
          height: r.height,
          text: b.textContent?.trim(),
          title: b.getAttribute('title'),
        }
      }),
    }))
    expect(layout.fits).toBe(true)
    for (const b of layout.buttons) {
      expect(b.text).toBe('')
      expect(b.title).toBeNull()
      expect(b.width).toBeLessThanOrEqual(36)
      expect(b.height).toBe(36)
      expect(b.y).toBe(layout.buttons[0]!.y)
      expect(b.x).toBeGreaterThanOrEqual(0)
      expect(b.x + b.width).toBeLessThanOrEqual(width)
    }
    await page.getByRole('button', { name: 'Settings', exact: true }).blur()
    await page.mouse.move(0, 0)
    await settings.hover()
    const tooltip = page.getByRole('tooltip')
    await expect(tooltip).toBeVisible()
    const bounds = await tooltip.boundingBox()
    expect(bounds!.x).toBeGreaterThanOrEqual(8)
    expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width - 8)
    await page.screenshot({ path: info.outputPath(`icon-controls-${width}.png`), fullPage: true })
  }
  expect(errors).toEqual([])
})

test.describe('connected device shortcut', () => {
  test.use({ fakePhone: true })

  test('dashboard Move device shortcut persists and stays in sync with Settings', async ({
    page,
  }, info) => {
    await page.goto('/')
    const shortcut = page
      .getByRole('region', { name: 'Controls', exact: true })
      .getByRole('switch', { name: 'Move device', exact: true })
    await expect(shortcut).toHaveCount(0)
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await page.getByRole('button', { name: 'Discover', exact: true }).click()
    await expect(
      page.getByRole('option', { name: 'Offline phone · device', exact: true }),
    ).toBeAttached()
    await page.getByRole('combobox', { name: 'Device', exact: true }).selectOption('offline-phone')
    await page.getByRole('button', { name: 'Connect', exact: true }).click()
    await expect(page.getByText('Phone connected', { exact: true })).toBeVisible()
    await page.getByRole('button', { name: 'Done', exact: true }).click()
    await expect(shortcut).toBeEnabled()
    await expect(shortcut).not.toBeChecked()
    await shortcut.click()
    await expect(shortcut).toBeChecked()
    await page.reload()
    await expect(shortcut).toBeChecked()
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await page.getByRole('tab', { name: 'Automation', exact: true }).click()
    const setting = page
      .getByRole('dialog')
      .getByRole('switch', { name: 'Move device', exact: true })
    await expect(setting).toBeChecked()
    await setting.uncheck()
    await page.getByRole('button', { name: 'Done', exact: true }).click()
    await expect(shortcut).not.toBeChecked()
    await page.getByRole('button', { name: 'Settings', exact: true }).blur()
    await page.mouse.move(0, 0)
    for (const width of [1024, 320]) {
      await page.setViewportSize({ width, height: 1000 })
      await page
        .getByRole('region', { name: 'Controls', exact: true })
        .screenshot({ path: info.outputPath(`move-device-${width}.png`) })
    }
  })
})

test('desktop controller aligns with the left panel bottom while input, placement and alternatives change', async ({
  page,
}) => {
  await page.setViewportSize({ width: 1180, height: 820 })
  await page.goto('/')
  const consolePanel = page.getByRole('region', { name: 'Controls', exact: true })
  const settings = consolePanel.getByRole('button', { name: 'Settings', exact: true })
  await expect(settings).toBeEnabled()
  for (const [width, height] of [
    [1440, 1000],
    [1024, 900],
    [820, 700],
    [1180, 540],
    [1180, 820],
  ] as const) {
    await page.setViewportSize({ width, height })
    await expect
      .poll(async () => {
        const panel = (await consolePanel.boundingBox())!
        const board = (await page.locator('.board-section').boundingBox())!
        return Math.abs(panel.y + panel.height - (board.y + board.height))
      })
      .toBeLessThanOrEqual(1)
    const panel = (await consolePanel.boundingBox())!
    expect(panel.y + panel.height).toBeLessThanOrEqual(height)
  }
  const initial = (await settings.boundingBox())!
  async function staysAnchored() {
    const activity = page.getByRole('region', { name: 'Current activity', exact: true })
    await expect(activity).toBeVisible()
    const statusBounds = (await activity.boundingBox())!
    const controlBounds = (await consolePanel.boundingBox())!
    expect(
      Math.abs(controlBounds.y - (statusBounds.y + statusBounds.height) - 8),
    ).toBeLessThanOrEqual(1)
    await expect(consolePanel.locator('p')).toHaveCount(0)
    await expect
      .poll(async () => {
        const panel = (await consolePanel.boundingBox())!
        const board = (await page.locator('.board-section').boundingBox())!
        return Math.abs(panel.y + panel.height - (board.y + board.height))
      })
      .toBeLessThanOrEqual(1)
    expect(Math.abs((await settings.boundingBox())!.y - initial.y)).toBeLessThanOrEqual(1)
  }
  await staysAnchored()
  for (const [slot, shape] of [
    [1, 'Line 2'],
    [2, 'Line 3'],
  ] as const) {
    await page.getByRole('button', { name: new RegExp(`^Edit Slot ${slot},`) }).click()
    await page.getByRole('button', { name: `Use ${shape}`, exact: true }).click()
    await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
    await expect(page.locator('.plan-row')).toHaveCount(slot)
    await staysAnchored()
  }
  await page.getByRole('button', { name: 'Explore alternative solutions', exact: true }).click()
  await expect(page.getByRole('button', { name: /^View .*solutions/ })).toBeEnabled()
  await staysAnchored()
  await page.getByRole('button', { name: /^View .*solutions/ }).click()
  await expect(page.locator('.candidate-row')).toHaveCount(10)
  await staysAnchored()
  await page.getByRole('combobox', { name: 'Solutions per page', exact: true }).selectOption('20')
  await expect(page.locator('.candidate-row')).toHaveCount(20)
  await staysAnchored()
})
