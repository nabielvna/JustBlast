import type { CalibrationAction } from '../../generated/contracts'
import { parseCalibration } from './schema'
import { nativeCall, workspaceApi } from './workspace'
export { validHardwareProfile } from './schema'

export const calibrationApi = {
  request: async (action: CalibrationAction) => {
    const snapshot = await workspaceApi.snapshot()
    return nativeCall(
      'calibration_request',
      { commandId: crypto.randomUUID(), expectedRevision: snapshot.device.revision, action },
      parseCalibration,
    )
  },
}
