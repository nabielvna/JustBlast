import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import {
  normalize,
  transform,
  parseImportedShapes,
  presets,
} from '../../ui/src/lib/presentation/shapes.ts'
test('draft transforms preserve holes and return to the original orientation', () => {
  const cells: [number, number][] = [
    [0, 0],
    [0, 2],
    [1, 0],
    [1, 1],
    [1, 2],
  ]
  let rotated = cells
  for (let i = 0; i < 4; i++) rotated = transform(rotated, 'rotate')
  assert.deepEqual(rotated, normalize(cells))
  assert.deepEqual(transform(transform(cells, 'flip'), 'flip'), normalize(cells))
  const catalogue = JSON.parse(readFileSync('resources/shapes-v1.json', 'utf8'))
  assert.deepEqual(
    presets.map(({ id, cells }) => ({ id: Number(id.slice(5)), cells })),
    catalogue.shapes,
  )
})
test('legacy browser arrays and application preference envelopes import the same validated shapes', () => {
  const saved = [
    {
      id: 'custom-old',
      name: 'Old drawing',
      cells: [
        [1, 1],
        [2, 1],
        [2, 2],
      ],
    },
  ]
  const browser = parseImportedShapes(JSON.stringify(saved)),
    application = parseImportedShapes(JSON.stringify({ schemaVersion: 1, shapes: saved }))
  assert.deepEqual(browser, application)
  assert.deepEqual(browser[0]?.cells, [
    [0, 0],
    [1, 0],
    [1, 1],
  ])
  for (const value of [
    [
      {
        ...saved[0],
        cells: [
          [0, 0],
          [0, 0],
        ],
      },
    ],
    [{ ...saved[0], cells: [[5, 0]] }],
    { schemaVersion: 3, shapes: saved },
  ])
    assert.throws(() => parseImportedShapes(JSON.stringify(value)))
})
