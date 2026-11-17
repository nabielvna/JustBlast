import { test, expect } from '@playwright/test'

test('a browser without a native runtime reports the missing connection instead of faking success', async ({
  page,
}) => {
  await page.goto('/')
  await expect(page.getByRole('alert')).toHaveText(
    'Open the JustBlast desktop app to use this workspace.',
  )
  await expect(page.getByRole('button', { name: 'Settings', exact: true })).toBeDisabled()
  await expect(page.locator('.board-cell.is-filled')).toHaveCount(0)
})
