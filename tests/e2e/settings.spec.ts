import { test, expect } from './native'
test.use({ fakePhone: true })

test('settings separate wireless pairing and connection addresses and isolate sections', async ({
  page,
}) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Settings', exact: true })
  await expect(dialog.getByRole('tab')).toHaveCount(5)
  await dialog.getByText('Device setup', { exact: true }).click()
  const calibration = dialog.getByRole('combobox', { name: 'Calibration', exact: true })
  await expect(calibration).toHaveValue('default')
  await expect(calibration.locator('option')).toHaveText(['Default · 1080 × 2400'])
  await dialog.getByText('Device setup', { exact: true }).click()
  await dialog.getByRole('button', { name: 'Wireless', exact: true }).click()
  const connection = dialog.getByRole('textbox', { name: 'Connection address', exact: true })
  await connection.fill('192.168.1.2:40567')
  await dialog.getByText('Pair a new device', { exact: true }).click()
  const pairing = dialog.getByRole('textbox', { name: 'Pairing address', exact: true })
  await pairing.fill('192.168.1.2:37123')
  await dialog.getByRole('textbox', { name: 'Pairing code', exact: true }).fill('12345')
  await expect(dialog.getByRole('button', { name: 'Pair device', exact: true })).toBeDisabled()
  await dialog.getByRole('textbox', { name: 'Pairing code', exact: true }).fill('123456')
  await expect(dialog.getByRole('button', { name: 'Pair device', exact: true })).toBeEnabled()
  await expect(connection).toHaveValue('192.168.1.2:40567')
  await expect(pairing).toHaveValue('192.168.1.2:37123')
  await expect(dialog.getByRole('spinbutton', { name: 'points weight' })).toBeHidden()
  await dialog.getByRole('tab', { name: 'Algorithm scoring', exact: true }).click()
  await expect(dialog.getByRole('spinbutton', { name: 'points weight' })).toBeVisible()
  await expect(connection).toBeHidden()
  await dialog.getByRole('tab', { name: 'Algorithm scoring', exact: true }).press('ArrowDown')
  await expect(dialog.getByRole('tab', { name: 'Cycle history', exact: true })).toBeFocused()
  await expect(dialog.getByText('No completed cycles yet.', { exact: true })).toBeVisible()
  await dialog.getByRole('tab', { name: 'Phone connection', exact: true }).click()
  await page.screenshot({ path: 'test-results/settings-wireless-desktop.png' })
  await page.setViewportSize({ width: 390, height: 844 })
  await expect(dialog).toBeVisible()
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  await page.screenshot({ path: 'test-results/settings-wireless-mobile.png' })
  await dialog.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await dialog.getByText('Pair a new device', { exact: true }).click()
  await expect(dialog.getByRole('textbox', { name: 'Pairing code', exact: true })).toHaveValue('')
})

test('wireless buttons send the correct endpoint and never reuse the pairing code', async ({
  page,
}) => {
  const actions: unknown[] = []
  await page.exposeFunction('recordWireless', (action: unknown) => {
    actions.push(action)
  })
  await page.goto('/')
  await page.evaluate(() => {
    const bridge = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          command: string,
          args?: { request?: { action?: { type: string } } },
        ) => Promise<unknown>
      }
      recordWireless: (action: unknown) => Promise<void>
    }
    const invoke = bridge.__TAURI_INTERNALS__.invoke
    bridge.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'device_request' && args?.request?.action?.type === 'wireless') {
        await bridge.recordWireless(args.request.action)
      }
      return invoke(command, args)
    }
  })
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Settings', exact: true })
  await dialog.getByRole('button', { name: 'Wireless', exact: true }).click()
  await dialog.getByText('Pair a new device', { exact: true }).click()
  await dialog
    .getByRole('textbox', { name: 'Pairing address', exact: true })
    .fill('192.168.1.2:37123')
  await dialog.getByRole('textbox', { name: 'Pairing code', exact: true }).fill('123456')
  await dialog.getByRole('button', { name: 'Pair device', exact: true }).click()
  await expect.poll(() => actions.length).toBe(1)
  await expect(dialog.getByRole('textbox', { name: 'Pairing code', exact: true })).toHaveValue('')
  await dialog
    .getByRole('textbox', { name: 'Connection address', exact: true })
    .fill('192.168.1.2:40567')
  await dialog.getByRole('button', { name: 'Connect wireless', exact: true }).click()
  await expect.poll(() => actions.length).toBe(2)
  expect(actions).toEqual([
    { type: 'wireless', address: '192.168.1.2:37123', code: '123456' },
    { type: 'wireless', address: '192.168.1.2:40567', code: null },
  ])
})
