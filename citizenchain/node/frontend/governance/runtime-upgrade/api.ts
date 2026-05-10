import { invoke } from '../../core/tauri';
import type {
  ProposeUpgradeRequestResult,
  VoteSignRequestResult,
  VoteSubmitResult,
} from '../types';

// Runtime 升级专用 Tauri API。创建入口只在 node 端,移动端只展示/签名。
export const runtimeUpgradeApi = {
  buildProposeUpgradeRequest: (pubkeyHex: string, wasmPath: string, reason: string) =>
    invoke<ProposeUpgradeRequestResult>('build_propose_upgrade_request', { pubkeyHex, wasmPath, reason }),
  submitProposeUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    reason: string,
    eligibleTotal: number,
    snapshotNonce: string,
    snapshotSignature: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_upgrade', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      wasmPath,
      reason,
      eligibleTotal,
      snapshotNonce,
      snapshotSignature,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildDeveloperUpgradeRequest: (pubkeyHex: string, wasmPath: string) =>
    invoke<VoteSignRequestResult>('build_developer_upgrade_request', { pubkeyHex, wasmPath }),
  submitDeveloperUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_developer_upgrade', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      wasmPath,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
