import { invoke } from '../../tauri';
import type {
  VoteSignRequestResult,
  VoteSubmitResult,
} from '../types';

export type ProposeUpgradeRequestResult = VoteSignRequestResult;
export type PowDifficultyParams = {
  paramsVersion: number;
  algorithmVersion: number;
  targetBlockTimeMs: number;
  adjustmentInterval: number;
  maxAdjustUpFactor: number;
  maxAdjustDownDivisor: number;
};

// 协议升级专用 Tauri API。这里只提交业务提案，投票流程统一交给投票引擎。
export const runtimeUpgradeApi = {
  getPowDifficultyParams: () =>
    invoke<PowDifficultyParams>('get_pow_difficulty_params'),
  buildProposeUpgradeRequest: (
    pubkeyHex: string,
    wasmPath: string,
    reason: string,
    powParams: PowDifficultyParams,
  ) =>
    invoke<ProposeUpgradeRequestResult>('build_propose_upgrade_request', {
      pubkeyHex,
      wasmPath,
      reason,
      powParams,
    }),
  submitProposeUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    reason: string,
    powParams: PowDifficultyParams,
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
      powParams,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildDeveloperUpgradeRequest: (
    pubkeyHex: string,
    wasmPath: string,
    powParams: PowDifficultyParams,
  ) => invoke<VoteSignRequestResult>('build_developer_upgrade_request', {
    pubkeyHex,
    wasmPath,
    powParams,
  }),
  submitDeveloperUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    powParams: PowDifficultyParams,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_developer_upgrade', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      wasmPath,
      powParams,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
