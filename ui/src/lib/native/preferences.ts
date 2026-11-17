import type { Preferences, HistoryEntry } from '../../generated/contracts'
import { record } from './schema'
import { nativeCall } from './workspace'
import { parseImportedShapes } from '../presentation/shapes'
export function parsePreferences(v: unknown): Preferences {
  if (!record(v) || v.schemaVersion !== 2 || !record(v.automation) || !record(v.connection))
    throw new Error('Invalid preferences response')
  parseImportedShapes(JSON.stringify(v.shapes))
  if (
    !record(v.weights) ||
    !['points', 'combo', 'flexibility', 'fragmentation'].every(
      (k) =>
        typeof (v.weights as Record<string, unknown>)[k] === 'number' &&
        Number.isFinite((v.weights as Record<string, number>)[k]) &&
        (v.weights as Record<string, number>)[k]! >= 0,
    )
  )
    throw new Error('Invalid saved scoring weights')
  if (
    !['moveDevice', 'continuous', 'checkAfterMove', 'checkAfterCycle'].every(
      (k) => typeof (v.automation as Record<string, unknown>)[k] === 'boolean',
    ) ||
    typeof v.automation.cycleLimit !== 'number' ||
    !Number.isInteger(v.automation.cycleLimit) ||
    v.automation.cycleLimit < 0 ||
    v.automation.cycleLimit > 100000 ||
    typeof v.connection.profileId !== 'string' ||
    typeof v.connection.serial !== 'string' ||
    typeof v.connection.wirelessAddress !== 'string' ||
    !['usb', 'wireless'].includes(String(v.connection.method))
  )
    throw new Error('Invalid automation preferences')
  return v as unknown as Preferences
}
function parseHistory(v: unknown): HistoryEntry[] {
  if (
    !Array.isArray(v) ||
    v.length > 200 ||
    !v.every(
      (e: unknown) =>
        record(e) &&
        typeof e.timestamp === 'string' &&
        typeof e.source === 'string' &&
        typeof e.status === 'string' &&
        typeof e.message === 'string' &&
        Number.isInteger(e.cycle) &&
        Array.isArray(e.board) &&
        e.board.length === 8 &&
        e.board.every((r: unknown) => typeof r === 'string' && /^[.#]{8}$/.test(r)),
    )
  )
    throw new Error('Invalid history response')
  return v as HistoryEntry[]
}
export const preferencesApi = {
  snapshot: () => nativeCall('preferences_snapshot', undefined, parsePreferences),
  save: (prefs: Preferences) => nativeCall('save_preferences', prefs, parsePreferences),
  history: () => nativeCall('history_snapshot', undefined, parseHistory),
}
