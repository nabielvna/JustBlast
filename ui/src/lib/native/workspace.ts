import { invoke, isTauri } from '@tauri-apps/api/core'
import type {
  ReplaceBoardRequest,
  WorkspaceAction,
  ExplorerRequest,
  CandidateRequest,
  DeviceRequest,
} from '../../generated/contracts'
import { nativeError, parseWorkspace, parsePlan, parseExplorer } from './schema'

export async function nativeCall<T>(
  command: string,
  request: unknown,
  parse: (value: unknown) => T,
): Promise<T> {
  if (!isTauri()) throw new Error('Open the JustBlast desktop app to use this workspace.')
  try {
    return parse(await invoke<unknown>(command, request === undefined ? undefined : { request }))
  } catch (reason) {
    throw nativeError(reason)
  }
}
export const workspaceApi = {
  device: (r: DeviceRequest) => nativeCall('device_request', r, parseWorkspace),
  snapshot: () => nativeCall('workspace_snapshot', undefined, parseWorkspace),
  replaceBoard: (r: ReplaceBoardRequest) =>
    nativeCall('replace_workspace_board', r, parseWorkspace),
  action: (r: WorkspaceAction) => nativeCall('workspace_action', r, parseWorkspace),
  explorer: (r: ExplorerRequest) => nativeCall('explorer_page', r, parseExplorer),
  preview: (r: CandidateRequest) => nativeCall('preview_candidate', r, parsePlan),
}
