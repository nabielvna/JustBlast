import { test, expect } from './native'
import type { Page, Locator } from '@playwright/test'

async function measure(button: Locator) {
  return button.evaluate((el) => {
    const r = el.getBoundingClientRect(),
      s = getComputedStyle(el)
    return {
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
      radius: s.borderRadius,
      borders: [s.borderTopWidth, s.borderRightWidth, s.borderBottomWidth, s.borderLeftWidth],
      transform: s.transform,
      shadow: s.boxShadow,
      opacity: s.opacity,
      outline: s.outlineStyle,
      transitions: s.transitionProperty,
    }
  })
}
async function uniform(page: Page) {
  const states = await page
    .locator(
      'button:visible:not(.board-cell):not(.manual-piece-cell):not(.plan-row-select):not(.candidate-row):not(.piece-tile):not(.shape-library-choice):not(.slot-choice):not(.slot-crop-origin)',
    )
    .evaluateAll((buttons) =>
      buttons.map((b) => {
        const s = getComputedStyle(b)
        return {
          name: b.getAttribute('aria-label') ?? b.textContent,
          radius: s.borderRadius,
          borders: [s.borderTopWidth, s.borderRightWidth, s.borderBottomWidth, s.borderLeftWidth],
          opacity: s.opacity,
          transform: s.transform,
          shadow: s.boxShadow,
        }
      }),
    )
  expect(states.length).toBeGreaterThan(3)
  for (const s of states) {
    expect(s, s.name ?? '').toMatchObject({
      radius: '8px',
      borders: ['1px', '1px', '1px', '1px'],
      opacity: '1',
      transform: 'none',
      shadow: 'none',
    })
  }
}
async function stable(page: Page, button: Locator) {
  await button.scrollIntoViewIfNeeded()
  await page.mouse.move(0, 0)
  const before = await measure(button)
  await button.hover()
  await button.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  )
  expect(await measure(button)).toEqual(before)
  await button.evaluate(async (el) => {
    await Promise.all(el.getAnimations().map((a) => a.finished))
  })
  expect(await measure(button)).toEqual(before)
  await page.evaluate(() =>
    document.addEventListener(
      'click',
      (e) => {
        e.preventDefault()
        e.stopImmediatePropagation()
      },
      { capture: true, once: true },
    ),
  )
  await page.mouse.down()
  expect(await measure(button)).toEqual(before)
  await page.mouse.up()
  await page.mouse.move(0, 0)
  await button.focus()
  const focused = await measure(button)
  expect(focused).toEqual(before)
}

test('action buttons keep one shape and intact borders through hover, press, focus, selection and disabled states', async ({
  page,
}, info) => {
  test.setTimeout(60000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await page.goto('/')
  await expect(page.getByRole('button', { name: 'Settings', exact: true })).toBeEnabled()
  await uniform(page)
  await stable(page, page.getByRole('button', { name: 'Scan', exact: true }))
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await uniform(page)
  await stable(page, page.getByRole('tab', { name: 'Automation', exact: true }))
  await page.getByRole('tab', { name: 'Automation', exact: true }).click()
  await uniform(page)
  await page.screenshot({ path: info.outputPath('settings-buttons.png'), fullPage: true })
  await page.getByRole('button', { name: 'Done', exact: true }).click()
  await page.getByRole('button', { name: /^Edit Slot 1,/ }).click()
  await page.getByRole('button', { name: 'Use Line 2', exact: true }).click()
  await uniform(page)
  await stable(page, page.getByRole('button', { name: 'Rotate ↻', exact: true }))
  await page.screenshot({ path: info.outputPath('editor-buttons.png'), fullPage: true })
  await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
  await expect(page.locator('.plan-row')).toHaveCount(1)
  await uniform(page)
  const step = page.locator('.plan-row-select').first()
  await expect(step).toHaveCSS('border-top-width', '0px')
  await expect(step).toHaveCSS('border-left-width', '0px')
  await expect(step).toHaveCSS('border-radius', '0px')
  await expect(step).toHaveCSS('background-color', 'rgba(0, 0, 0, 0)')
  await expect(page.locator('.plan-row').first()).toHaveCSS('padding-top', '0px')
  await page.screenshot({ path: info.outputPath('flat-plan-rows.png'), fullPage: true })
  await stable(page, page.getByRole('button', { name: 'Preview through step 1', exact: true }))
  await page.getByRole('button', { name: 'Explore alternative solutions', exact: true }).click()
  await expect(page.getByRole('button', { name: /^View .*solutions/ })).toBeEnabled()
  await uniform(page)
  await page.getByRole('button', { name: /^View .*solutions/ }).click()
  await expect(page.locator('.candidate-row')).toHaveCount(10)
  await uniform(page)
  await expect(page.locator('.candidate-row').first()).toHaveCSS('border-top-width', '0px')
  await expect(page.locator('.candidate-row').first()).toHaveCSS('border-left-width', '0px')
  await page.locator('.candidate-row').first().click()
  await uniform(page)
  await page.setViewportSize({ width: 390, height: 1000 })
  await page.screenshot({ path: info.outputPath('explorer-buttons-390.png'), fullPage: true })
  expect(errors).toEqual([])
})

test('three plan steps remain a flat list with one selection background', async ({
  page,
}, info) => {
  test.setTimeout(60000)
  await page.goto('/')
  for (const [slot, shape] of [
    [1, 'Line 2'],
    [2, 'Line 3'],
    [3, 'Corner 3'],
  ] as const) {
    await page.getByRole('button', { name: new RegExp(`^Edit Slot ${slot},`) }).click()
    await page.getByRole('button', { name: `Use ${shape}`, exact: true }).click()
    await page.getByRole('button', { name: 'Apply to slot', exact: true }).click()
    await expect(page.locator('.plan-row')).toHaveCount(slot)
  }
  await expect(page.locator('.plan-row')).toHaveCount(3)
  await page.locator('.plan-row-select').nth(1).click()
  for (const width of [1440, 760, 390]) {
    await page.setViewportSize({ width, height: 1000 })
    for (const row of await page.locator('.plan-row').all()) {
      const select = row.locator('.plan-row-select')
      await expect(select).toHaveCSS('border-width', '0px')
      await expect(select).toHaveCSS('border-radius', '0px')
      await expect(select).toHaveCSS('background-color', 'rgba(0, 0, 0, 0)')
      await expect(row).toHaveCSS('padding-top', '0px')
      const geometry = await row.evaluate((el) => {
        const r = el.getBoundingClientRect(),
          d = el.querySelector('.plan-destination')!.getBoundingClientRect(),
          p = el.querySelector('.plan-row-action')!.getBoundingClientRect()
        return {
          height: r.height,
          divider: parseFloat(getComputedStyle(el).borderBottomWidth),
          rowHeight: parseFloat(getComputedStyle(el).getPropertyValue('--row-height')),
          clear: d.right <= p.left,
          aligned: Math.abs(p.y + p.height / 2 - (r.y + r.height / 2)) < 1,
        }
      })
      expect(geometry.height).toBe(geometry.rowHeight + geometry.divider)
      expect(geometry.clear).toBe(true)
      expect(geometry.aligned).toBe(true)
    }
    await expect(page.locator('.plan-row.is-selected')).toHaveCount(1)
    await page
      .locator('.plan-inspector')
      .screenshot({ path: info.outputPath(`flat-plan-${width}.png`) })
  }
})
