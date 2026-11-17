import test from 'node:test'
import assert from 'node:assert/strict'
import { parseCalibration, validHardwareProfile } from '../../ui/src/lib/native/schema.ts'
const profile = {
  screen: {
    width: 1080,
    height: 2400,
    board: [63, 582, 954, 954],
    slots: [
      [30, 1650, 335, 370],
      [370, 1650, 340, 370],
      [715, 1650, 335, 370],
    ],
    preview_pitch: 54,
    cell_majority: 0.9,
  },
  touch: { gain: 1.4, lift: 269, hold: 0.08, drop: 0.08, gap: 0.08, risk_max: 0.08 },
  runtime: { tray_wait: 0.5, retry_wait: 0.1, tray_attempts: 3, command_timeout: 10 },
}
test('hardware calibration rejects invalid dimensions, rectangles, pitch and touch values', () => {
  assert.equal(validHardwareProfile(profile), true)
  for (const value of [
    { ...profile, screen: { ...profile.screen, width: 1e8 } },
    { ...profile, screen: { ...profile.screen, board: [0, 0, 2000, 500] } },
    { ...profile, screen: { ...profile.screen, preview_pitch: NaN } },
    { ...profile, touch: { ...profile.touch, gain: 0 } },
    { ...profile, touch: { ...profile.touch, risk_max: 0.01 } },
    { ...profile, runtime: { ...profile.runtime, tray_attempts: 0 } },
  ])
    assert.equal(validHardwareProfile(value), false)
})
test('calibration response only accepts bounded screenshot URLs and valid recognition data', () => {
  const view = {
    layoutDetected: null,
    unclearCells: [],
    anchors: [],
    movement: null,
    revision: 1,
    profile,
    toml: '',
    captureId: 1,
    image: 'data:image/png;base64,YQ==',
    board: null,
    boardUnclear: false,
    pieces: [],
    unknownSlots: [],
    message: '',
  }
  assert.equal(parseCalibration(view).captureId, 1)
  for (const value of [
    { ...view, image: 'https://example.com/image.png' },
    { ...view, board: ['bad'] },
    { ...view, captureId: -1 },
    { ...view, layoutDetected: 'yes' },
    { ...view, unclearCells: [64] },
    { ...view, anchors: [{ slot: 0, x: -1, y: 100 }] },
    { ...view, anchors: [{ slot: 0, x: 0, y: 2400 }] },
    { ...view, unknownSlots: [3] },
    { ...view, pieces: [{ slot: 0, cells: [] }] },
  ])
    assert.throws(() => parseCalibration(value))
})
