import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { test, expect } from './native'

test('migrated dashboard regions match the live legacy reference', async ({
  page,
  context,
}, info) => {
  const referenceUrl = process.env.JUSTBLAST_LEGACY_URL
  test.skip(
    !referenceUrl,
    'Set JUSTBLAST_LEGACY_URL to compare against a separately running legacy app',
  )
  const fontCss = readFileSync('ui/src/styles/fonts.css', 'utf8').replaceAll(
    '/fonts/',
    'https://fonts.gstatic.com/justblast/',
  )
  await context.route('https://fonts.googleapis.com/**', (route) =>
    route.fulfill({ contentType: 'text/css', body: fontCss }),
  )
  await context.route('https://fonts.gstatic.com/justblast/**', (route) =>
    route.fulfill({
      contentType: 'font/woff2',
      path: path.join('ui/public/fonts', path.basename(new URL(route.request().url()).pathname)),
      headers: { 'access-control-allow-origin': '*' },
    }),
  )
  const reference = await context.newPage()
  try {
    for (const width of [1440, 1024, 760, 390]) {
      await page.setViewportSize({ width, height: 1000 })
      await reference.setViewportSize({ width, height: 1000 })
      await Promise.all([page.goto('/'), reference.goto(referenceUrl!)])
      await reference.addStyleTag({ content: readFileSync('ui/src/styles/buttons.css', 'utf8') })
      await expect(page.getByRole('button', { name: 'Settings', exact: true })).toBeEnabled()
      await expect(reference.getByRole('button', { name: 'Settings', exact: true })).toBeEnabled()
      await reference.getByRole('button', { name: 'Settings', exact: true }).click()
      await reference.getByRole('combobox', { name: 'Mode', exact: true }).selectOption('manual')
      for (const target of [page, reference]) {
        await target.evaluate(() => document.fonts.ready)
        await target.mouse.move(0, 0)
      }
      for (const selector of ['.brand', '.board-section', '.piece-tray']) {
        const name = `unified-${width}-${selector.slice(1)}.png`
        const baseline = info.snapshotPath(name)
        mkdirSync(path.dirname(baseline), { recursive: true })
        writeFileSync(baseline, await reference.locator(selector).screenshot())
        expect(await page.locator(selector).screenshot()).toMatchSnapshot(name, {
          maxDiffPixelRatio: 0.001,
          threshold: 0.1,
        })
      }
    }
  } finally {
    await reference.close()
  }
})

test('editor, populated tray, plan and alternatives preserve the legacy presentation', async ({
  page,
  context,
}, info) => {
  test.setTimeout(120_000)
  const referenceUrl = process.env.JUSTBLAST_LEGACY_URL
  test.skip(!referenceUrl, 'Set JUSTBLAST_LEGACY_URL for the independent legacy reference')
  const fontCss = readFileSync('ui/src/styles/fonts.css', 'utf8').replaceAll(
    '/fonts/',
    'https://fonts.gstatic.com/justblast/',
  )
  await context.route('https://fonts.googleapis.com/**', (route) =>
    route.fulfill({ contentType: 'text/css', body: fontCss }),
  )
  await context.route('https://fonts.gstatic.com/justblast/**', (route) =>
    route.fulfill({
      contentType: 'font/woff2',
      path: path.join('ui/public/fonts', path.basename(new URL(route.request().url()).pathname)),
      headers: { 'access-control-allow-origin': '*' },
    }),
  )
  const reference = await context.newPage()
  await reference.addInitScript(() => localStorage.removeItem('justblast.manual.session.v2'))
  async function compare(selector: string, name: string) {
    for (const target of [page, reference]) {
      await target.evaluate(() => document.fonts.ready)
      await target.mouse.move(0, 0)
      await target.locator(selector).evaluate((element) => {
        const box = element.getBoundingClientRect()
        if (element.classList.contains('candidate-pagination')) {
          ;(element as HTMLElement).style.height = `${Math.round(box.height)}px`
        }
        ;(element as HTMLElement).style.transform =
          `translate(${Math.round(box.x) - box.x}px,${Math.round(box.y) - box.y}px)`
      })
    }
    const baseline = info.snapshotPath(`${name}.png`)
    mkdirSync(path.dirname(baseline), { recursive: true })
    writeFileSync(
      baseline,
      await reference.locator(selector).screenshot({ animations: 'disabled' }),
    )
    expect(await page.locator(selector).screenshot({ animations: 'disabled' })).toMatchSnapshot(
      `${name}.png`,
      selector === '.plan-heading'
        ? { maxDiffPixels: 10, threshold: 0.1 }
        : selector === '.candidate-pagination'
          ? { maxDiffPixels: 300, threshold: 0.1 }
          : { maxDiffPixelRatio: 0.001, threshold: 0.1 },
    )
  }
  try {
    for (const width of [1440, 1024, 760, 390]) {
      for (const target of [page, reference]) {
        await target.setViewportSize({ width, height: 1000 })
        await target.goto(target === page ? '/' : referenceUrl!)
        if (target === reference)
          await target.addStyleTag({ content: readFileSync('ui/src/styles/buttons.css', 'utf8') })
        if (target === reference) {
          await target.getByRole('button', { name: 'Settings', exact: true }).click()
          await target.getByRole('combobox', { name: 'Mode', exact: true }).selectOption('manual')
          if (await target.getByRole('dialog', { name: 'Settings', exact: true }).isVisible())
            await target.getByRole('button', { name: 'Done', exact: true }).click()
        }
        const edit = target.getByRole('button', { name: 'Edit input', exact: true })
        if (await edit.isVisible()) await edit.click()
      }
      for (const target of [page, reference]) {
        await target.getByRole('button', { name: /^Edit Slot 1,/ }).click()
        await target.getByRole('button', { name: 'Use Line 2', exact: true }).click()
      }
      for (const [selector, name] of [
        ['.manual-piece-grid', 'drawing-grid'],
        ['.piece-editor-transforms', 'transforms'],
        ['.shape-library-choice[aria-label="Use Single"]', 'preset-tile'],
        ['.control-config-header', 'editor-header'],
      ])
        await compare(selector!, `editor-${width}-${name}`)
      for (const target of [page, reference]) {
        await target.getByRole('button', { name: 'Apply to slot', exact: true }).click()
        await target.getByRole('button', { name: /^Edit Slot 3,/ }).click()
        await target.getByRole('button', { name: 'Use Corner 3', exact: true }).click()
        await target.getByRole('button', { name: 'Apply to slot', exact: true }).click()
      }
      await compare('.piece-tray', `populated-tray-${width}`)
      for (const target of [page, reference]) {
        await target
          .getByRole('button', { name: target === page ? 'Apply input' : 'Search', exact: true })
          .click()
        await expect(
          target.getByRole('heading', { name: 'Placement plan', exact: true }),
        ).toBeVisible()
      }
      for (const [selector, name] of [
        ['.board-section', 'board'],
        ['.piece-tray', 'tray'],
        ['.plan-row:first-child .plan-piece', 'step-piece'],
        ['.plan-heading', 'heading'],
      ])
        await compare(selector!, `plan-${width}-${name}`)
      for (const target of [page, reference]) {
        await target
          .getByRole('button', { name: 'Explore alternative solutions', exact: true })
          .click()
        await expect(target.getByRole('button', { name: /^View .*solutions/ })).toBeEnabled()
      }
      await compare('.move-order', `explorer-${width}-order`)
      for (const target of [page, reference])
        await target.getByRole('button', { name: /^View .*solutions/ }).click()
      await compare('.candidate-columns', `explorer-${width}-columns`)
      await compare('.candidate-row:first-child', `explorer-${width}-candidate`)
      for (const target of [page, reference]) {
        await expect(target.getByLabel('Solutions per page', { exact: true })).toHaveValue('10')
        await expect(target.locator('.pagination-summary')).toContainText('5,040')
      }
      await compare('.candidate-pagination', `explorer-${width}-pagination`)
      for (const target of [page, reference])
        await target.getByRole('button', { name: 'Back to placement plan', exact: true }).click()
    }
  } finally {
    await reference.close()
  }
})
