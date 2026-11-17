import type { SavedShape } from '../../generated/contracts'
export type Cell = [number, number]
export function normalize(cells: Cell[]): Cell[] {
  if (!cells.length) return []
  const top = Math.min(...cells.map((c) => c[0])),
    left = Math.min(...cells.map((c) => c[1]))
  return cells.map(([r, c]): Cell => [r - top, c - left]).sort((a, b) => a[0] - b[0] || a[1] - b[1])
}
export const shapeKey = (cells: Cell[]) => JSON.stringify(normalize(cells))
export function transform(cells: Cell[], action: 'rotate' | 'flip'): Cell[] {
  return normalize(cells.map(([r, c]) => (action === 'rotate' ? [c, 4 - r] : [r, 4 - c])))
}
export const presets: SavedShape[] = [
  {
    id: 'game-1',
    name: 'Single',
    cells: [[0, 0]],
  },
  {
    id: 'game-2',
    name: 'Game shape 2',
    cells: [
      [0, 0],
      [1, 0],
    ],
  },
  {
    id: 'game-3',
    name: 'Line 2',
    cells: [
      [0, 0],
      [0, 1],
    ],
  },
  {
    id: 'game-4',
    name: 'Game shape 4',
    cells: [
      [0, 0],
      [1, 0],
      [2, 0],
    ],
  },
  {
    id: 'game-5',
    name: 'Line 3',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
    ],
  },
  {
    id: 'game-6',
    name: 'Game shape 6',
    cells: [
      [0, 0],
      [0, 1],
      [1, 0],
    ],
  },
  {
    id: 'game-7',
    name: 'Game shape 7',
    cells: [
      [0, 0],
      [1, 0],
      [2, 0],
      [3, 0],
    ],
  },
  {
    id: 'game-8',
    name: 'Game shape 8',
    cells: [
      [0, 0],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  },
  {
    id: 'game-9',
    name: 'Game shape 9',
    cells: [
      [0, 0],
      [0, 1],
      [1, 0],
      [1, 1],
    ],
  },
  {
    id: 'game-10',
    name: 'Game shape 10',
    cells: [
      [0, 1],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  },
  {
    id: 'game-11',
    name: 'Line 5',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [0, 3],
      [0, 4],
    ],
  },
  {
    id: 'game-12',
    name: 'Game shape 12',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 2],
      [2, 2],
    ],
  },
  {
    id: 'game-13',
    name: 'Game shape 13',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 0],
      [1, 1],
      [1, 2],
      [2, 0],
      [2, 1],
      [2, 2],
    ],
  },
  {
    id: 'game-14',
    name: 'Game shape 14',
    cells: [
      [0, 1],
      [0, 2],
      [1, 0],
      [1, 1],
    ],
  },
  {
    id: 'game-15',
    name: 'Game shape 15',
    cells: [
      [0, 0],
      [0, 1],
      [1, 1],
    ],
  },
  {
    id: 'game-16',
    name: 'Game shape 16',
    cells: [
      [0, 0],
      [1, 0],
      [1, 1],
      [2, 1],
    ],
  },
  {
    id: 'game-17',
    name: 'Line 4',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [0, 3],
    ],
  },
  {
    id: 'game-18',
    name: 'Game shape 18',
    cells: [
      [0, 0],
      [0, 1],
      [1, 1],
      [1, 2],
    ],
  },
  {
    id: 'game-19',
    name: 'Game shape 19',
    cells: [
      [0, 1],
      [1, 0],
      [1, 1],
      [2, 0],
    ],
  },
  {
    id: 'game-20',
    name: 'Game shape 20',
    cells: [
      [0, 0],
      [1, 0],
      [1, 1],
      [2, 0],
    ],
  },
  {
    id: 'game-21',
    name: 'Game shape 21',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 0],
      [2, 0],
    ],
  },
  {
    id: 'game-22',
    name: 'Game shape 22',
    cells: [
      [0, 0],
      [1, 0],
      [2, 0],
      [3, 0],
      [4, 0],
    ],
  },
  {
    id: 'game-23',
    name: 'Game shape 23',
    cells: [
      [0, 0],
      [1, 0],
      [2, 0],
      [2, 1],
      [2, 2],
    ],
  },
  {
    id: 'game-24',
    name: 'Game shape 24',
    cells: [
      [0, 2],
      [1, 2],
      [2, 0],
      [2, 1],
      [2, 2],
    ],
  },
  {
    id: 'game-25',
    name: 'Game shape 25',
    cells: [
      [0, 1],
      [1, 0],
      [1, 1],
      [2, 1],
    ],
  },
  {
    id: 'game-26',
    name: 'Game shape 26',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 1],
    ],
  },
  {
    id: 'game-27',
    name: 'Corner 3',
    cells: [
      [0, 0],
      [1, 0],
      [1, 1],
    ],
  },
  {
    id: 'game-28',
    name: 'Game shape 28',
    cells: [
      [0, 1],
      [1, 0],
      [1, 1],
    ],
  },
  {
    id: 'game-29',
    name: 'Game shape 29',
    cells: [
      [0, 1],
      [1, 1],
      [2, 0],
      [2, 1],
    ],
  },
  {
    id: 'game-30',
    name: 'Game shape 30',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 2],
    ],
  },
  {
    id: 'game-31',
    name: 'Game shape 31',
    cells: [
      [0, 0],
      [0, 1],
      [1, 0],
      [2, 0],
    ],
  },
  {
    id: 'game-32',
    name: 'Game shape 32',
    cells: [
      [0, 0],
      [0, 1],
      [1, 1],
      [2, 1],
    ],
  },
  {
    id: 'game-33',
    name: 'Game shape 33',
    cells: [
      [0, 2],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  },
  {
    id: 'game-34',
    name: 'Game shape 34',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 0],
    ],
  },
  {
    id: 'game-35',
    name: 'Game shape 35',
    cells: [
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 0],
      [1, 1],
      [1, 2],
    ],
  },
  {
    id: 'game-36',
    name: 'Game shape 36',
    cells: [
      [0, 0],
      [0, 1],
      [1, 0],
      [1, 1],
      [2, 0],
      [2, 1],
    ],
  },
  {
    id: 'game-37',
    name: 'Game shape 37',
    cells: [
      [0, 0],
      [1, 1],
    ],
  },
  {
    id: 'game-38',
    name: 'Game shape 38',
    cells: [
      [0, 1],
      [1, 0],
    ],
  },
  {
    id: 'game-39',
    name: 'Game shape 39',
    cells: [
      [0, 0],
      [1, 1],
      [2, 2],
    ],
  },
  {
    id: 'game-40',
    name: 'Game shape 40',
    cells: [
      [0, 2],
      [1, 1],
      [2, 0],
    ],
  },
  {
    id: 'game-41',
    name: 'Game shape 41',
    cells: [
      [0, 2],
      [1, 1],
      [2, 0],
    ],
  },
  {
    id: 'game-42',
    name: 'Game shape 42',
    cells: [
      [0, 0],
      [1, 0],
      [2, 0],
      [2, 1],
    ],
  },
]
export function parseImportedShapes(raw: string): SavedShape[] {
  const parsed: unknown = JSON.parse(raw)
  const value: unknown =
    typeof parsed === 'object' &&
    parsed !== null &&
    !Array.isArray(parsed) &&
    'schemaVersion' in parsed &&
    (parsed.schemaVersion === 1 || parsed.schemaVersion === 2) &&
    'shapes' in parsed
      ? parsed.shapes
      : parsed
  if (!Array.isArray(value) || value.length > 500)
    throw new Error('Provide a shape collection containing at most 500 shapes.')
  const ids = new Set<string>()
  return value.map((v: unknown) => {
    if (typeof v !== 'object' || v === null) throw new Error('Invalid saved shape')
    const entry = v as Record<string, unknown>
    if (
      typeof entry.id !== 'string' ||
      !entry.id.startsWith('custom-') ||
      entry.id.length > 100 ||
      ids.has(entry.id) ||
      typeof entry.name !== 'string' ||
      !entry.name.trim() ||
      entry.name.length > 40 ||
      !Array.isArray(entry.cells) ||
      !entry.cells.length ||
      entry.cells.length > 25 ||
      !entry.cells.every(
        (cell: unknown) =>
          Array.isArray(cell) &&
          cell.length === 2 &&
          cell.every(
            (n: unknown) => typeof n === 'number' && Number.isInteger(n) && n >= 0 && n < 5,
          ),
      )
    )
      throw new Error('Invalid saved shape')
    const cells = normalize(entry.cells as Cell[])
    if (new Set(cells.map((c) => c.join(':'))).size !== cells.length)
      throw new Error('Duplicate shape cells')
    ids.add(entry.id)
    return { id: entry.id, name: entry.name.trim(), cells }
  })
}
