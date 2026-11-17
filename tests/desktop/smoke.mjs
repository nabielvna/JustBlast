import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync, existsSync, readFileSync } from 'node:fs'
import path from 'node:path'

if (process.platform !== 'linux') throw new Error('This smoke test currently supports Linux only')
const driver = process.env.JUSTBLAST_WEBDRIVER_URL ?? 'http://127.0.0.1:4445'
const application = process.env.JUSTBLAST_NATIVE_BINARY ?? path.resolve('target/release/justblast')
let session

async function request(method, endpoint, body) {
  const response = await fetch(`${driver}${endpoint}`, {
    method,
    headers: { 'content-type': 'application/json' },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    signal: AbortSignal.timeout(30_000),
  })
  const payload = await response.json()
  if (!response.ok) throw new Error(JSON.stringify(payload))
  return payload.value
}

const evaluate = (script) =>
  request('POST', `/session/${session}/execute/sync`, { script, args: [] })
const evaluateAsync = (script) =>
  request('POST', `/session/${session}/execute/async`, { script, args: [] })

async function waitFor(script) {
  const deadline = Date.now() + 10_000
  while (Date.now() < deadline) {
    if (await evaluate(script)) return
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
  throw new Error(`Native UI did not reach expected state: ${script}`)
}

async function click(selector) {
  const element = await request('POST', `/session/${session}/element`, {
    using: 'css selector',
    value: selector,
  })
  const id = element['element-6066-11e4-a52e-4f735466cecf']
  await request('POST', `/session/${session}/element/${id}/click`, {})
}

async function clickText(text) {
  await evaluate(
    `const button=[...document.querySelectorAll('button')].find(b=>(b.textContent.trim()===${JSON.stringify(text)}||b.getAttribute('aria-label')===${JSON.stringify(text)}));if(!button)throw new Error('Button missing');button.click();`,
  )
}

try {
  const opened = await request('POST', '/session', {
    capabilities: {
      alwaysMatch: { 'webkitgtk:browserOptions': { binary: application, args: [] } },
    },
  })
  session = opened.sessionId
  await waitFor(
    'return document.querySelector("button[aria-label=Settings]")?.getAttribute("aria-disabled") === "false"',
  )
  const startup = await evaluate(`return {
    url:location.href,
    native:window.isTauri,
    cells:document.querySelectorAll('.board-cell').length,
    occupied:document.querySelectorAll('.board-cell.is-filled').length,
    error:document.querySelector('[role=alert]')?.textContent ?? null
  }`)
  assert.equal(startup.url, 'tauri://localhost')
  assert.equal(startup.native, true)
  assert.equal(startup.cells, 64)
  assert.equal(startup.occupied, 0)
  assert.equal(startup.error, null)

  await click('button[aria-label="Settings"]')
  await clickText('Phone calibration')
  await evaluate(
    `const summary=[...document.querySelectorAll('summary')].find(el=>el.textContent.trim()==='Use a saved profile');summary.click();`,
  )
  await clickText('Load current profile')
  await waitFor('return !!document.querySelector(".calibration-resolution, .calibration-error")')
  const calibrationError = await evaluate(
    'return document.querySelector(".calibration-error")?.textContent ?? null',
  )
  assert.equal(calibrationError, null)
  await clickText('3. Pieces')
  await waitFor(`return !!document.querySelector('select[aria-label="Grid columns"]')`)
  assert.equal(await evaluate(`return !!document.querySelector('.tray-grid-controls')`), true)
  await clickText('4. Test')
  await waitFor(`return !!document.querySelector('input[aria-label="Drag gain"]')`)
  const hardware = await evaluateAsync(`
    const done=arguments[arguments.length-1];
    (async()=>{
      const snapshot=await window.__TAURI_INTERNALS__.invoke('workspace_snapshot');
      const result=await window.__TAURI_INTERNALS__.invoke('calibration_request',{request:{commandId:'native-profile-check',expectedRevision:snapshot.device.revision,action:{type:'read'}}});
      done({sections:Object.keys(result.profile).sort(),toml:result.toml});
    })().catch(error=>done({error:String(error)}));
  `)
  assert.deepEqual(hardware.sections, ['runtime', 'screen', 'touch'])
  assert(!/weights|combo|rule_context|wireless/.test(hardware.toml))
  await clickText('5. Save')
  if (process.env.JUSTBLAST_NATIVE_EXPORT_DIR) {
    await evaluate(
      `document.querySelector('summary') && [...document.querySelectorAll('summary')].find(el=>el.textContent.trim()==='Profile options').click()`,
    )
    const exported = path.join(process.env.JUSTBLAST_NATIVE_EXPORT_DIR, 'my-phone.toml')
    assert(!existsSync(exported), 'Use a fresh export directory for native download validation')
    await clickText('Export HP profile')
    const deadline = Date.now() + 10000
    while (!existsSync(exported) && Date.now() < deadline)
      await new Promise((resolve) => setTimeout(resolve, 50))
    assert(existsSync(exported), 'Native hardware profile download did not complete')
    const toml = readFileSync(exported, 'utf8')
    assert(toml.includes('[screen]') && toml.includes('[touch]') && toml.includes('[runtime]'))
    assert(!/weights|combo|rule_context|automation|wireless|serial/.test(toml))
  }
  await clickText('Save calibration')
  await waitFor(`return !!document.querySelector('.calibration-complete')`)
  const savedHardware = await evaluateAsync(`
    const done=arguments[arguments.length-1];
    window.__TAURI_INTERNALS__.invoke('workspace_snapshot').then(value=>done({profileId:value.device.profileId,weights:value.weights}),error=>done({error:String(error)}));
  `)
  assert.equal(savedHardware.profileId, 'my-phone')
  await clickText('Done')

  const cell = 'button[aria-label="Board row 8, column 8"]'
  await click(cell)
  await waitFor(
    `return document.querySelector(${JSON.stringify(cell)})?.getAttribute('aria-pressed') === 'true'`,
  )
  await waitFor('return !document.querySelector(".input-draft-panel")')
  await request('POST', `/session/${session}/refresh`, {})
  await waitFor(
    `return document.querySelector(${JSON.stringify(cell)})?.getAttribute('aria-pressed') === 'true'`,
  )
  assert.equal(
    await evaluate('return document.querySelectorAll("select[aria-label=Mode]").length'),
    0,
  )

  const invalid = await evaluateAsync(`
    const done=arguments[arguments.length-1];
    window.__TAURI_INTERNALS__.invoke('replace_workspace_board',{request:{expectedRevision:1,rows:Array(8).fill('invalid!')}})
      .then(()=>done({accepted:true}),error=>done(error));
  `)
  assert.equal(invalid.code, 'INVALID_INPUT')

  await waitFor(
    `return getComputedStyle(document.querySelector('.board-cell.is-filled'), '::before').opacity === '1'`,
  )

  const screenshot = await request('GET', `/session/${session}/screenshot`)
  mkdirSync('test-results/native', { recursive: true })
  writeFileSync('test-results/native/workspace.png', Buffer.from(screenshot, 'base64'))
  writeFileSync('test-results/native/startup.json', JSON.stringify(startup, null, 2) + '\n')
  await click('button[aria-label="Clear board"]')
  await waitFor('return document.querySelectorAll(".board-cell.is-filled").length === 0')
  for (const [slot, preset, occupied] of [
    [1, 'Line 2', 2],
    [2, 'Single', 3],
  ]) {
    await click(`button.piece-tile[aria-label^="Edit Slot ${slot},"]`)
    await waitFor('return document.querySelector("dialog.piece-editor")?.open')
    await click(`button[aria-label="Use ${preset}"]`)
    await click('dialog.piece-editor .piece-editor-actions .button-primary')
    await waitFor('return !document.querySelector("dialog.piece-editor")')
    await waitFor('return document.querySelectorAll(".plan-row").length === 1')
    await click('button[aria-label="Preview"]')
    await waitFor('return document.querySelectorAll(".plan-row.is-done").length === 1')
    await waitFor(
      `return [...document.querySelectorAll('button')].some(b=>b.getAttribute('aria-label')==='Continue simulation' && !b.disabled)`,
    )
    await click('button[aria-label="Continue simulation"]')
    await waitFor(
      `return document.querySelectorAll('.board-cell.is-filled').length === ${occupied} && [...document.querySelectorAll('.piece-name')].every(node=>node.textContent==='Empty slot')`,
    )
  }
  const finalScreenshot = await request('GET', `/session/${session}/screenshot`)
  writeFileSync('test-results/native/two-cycles.png', Buffer.from(finalScreenshot, 'base64'))
  console.log(
    'Native release passed: bundled assets, real IPC, calibration profile load/save/export, edits, validation, editor/presets, solver, preview and two unified simulation cycles.',
  )
} finally {
  if (session) await request('DELETE', `/session/${session}`)
}
