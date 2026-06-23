import { invoke } from '../../core/tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from './types';

// 多签转账模块专用 Tauri API，对齐后端 src/transaction/duoqian_transfer。
export const duoqianTransferApi = {
  buildDuoqianTransferRequest: (
    pubkeyHex: string,
    cidNumber: string,
    institutionCode: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_duoqian_transfer_request', {
      pubkeyHex,
      cidNumber,
      institutionCode,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitDuoqianTransfer: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    cidNumber: string,
    institutionCode: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_duoqian_transfer', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      cidNumber,
      institutionCode,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSweepRequest: (pubkeyHex: string, cidNumber: string, amountYuan: number) =>
    invoke<VoteSignRequestResult>('build_duoqian_sweep_request', {
      pubkeyHex,
      cidNumber,
      amountYuan,
    }),
  submitProposeSweep: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    cidNumber: string,
    amountYuan: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_duoqian_sweep', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      cidNumber,
      amountYuan,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSafetyFundRequest: (
    pubkeyHex: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_duoqian_safety_fund_request', {
      pubkeyHex,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitProposeSafetyFund: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_duoqian_safety_fund', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
