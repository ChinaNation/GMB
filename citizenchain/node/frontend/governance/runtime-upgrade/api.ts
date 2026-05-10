import { invoke } from '../../core/tauri';
import type {
  VoteSignRequestResult,
  VoteSubmitResult,
} from '../types';

export type ProposeUpgradeRequestResult = VoteSignRequestResult;

// 协议升级专用 Tauri API。这里只提交业务提案，投票流程统一交给投票引擎。
export const runtimeUpgradeApi = {
  buildProposeUpgradeRequest: (
    pubkeyHex: string,
    wasmPath: string,
    reason: string,
  ) =>
    invoke<ProposeUpgradeRequestResult>('build_propose_upgrade_request', {
      pubkeyHex,
      wasmPath,
      reason,
    }),
  submitProposeUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    reason: string,
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
