import { invoke } from '../../tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from './types';

// 多签转账模块专用 Tauri API，对齐后端 src/transaction/multisig_transfer。
export const multisigTransferApi = {
  buildMultisigTransferRequest: (
    pubkeyHex: string,
    actorCidNumber: string,
    institutionAccount: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_multisig_transfer_request', {
      pubkeyHex,
      actorCidNumber,
      institutionAccount,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitMultisigTransfer: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    actorCidNumber: string,
    institutionAccount: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_multisig_transfer', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      actorCidNumber,
      institutionAccount,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSweepRequest: (
    pubkeyHex: string,
    actorCidNumber: string,
    institutionAccount: string,
    amountYuan: number,
  ) =>
    invoke<VoteSignRequestResult>('build_multisig_sweep_request', {
      pubkeyHex,
      actorCidNumber,
      institutionAccount,
      amountYuan,
    }),
  submitProposeSweep: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    actorCidNumber: string,
    institutionAccount: string,
    amountYuan: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_multisig_sweep', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      actorCidNumber,
      institutionAccount,
      amountYuan,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSafetyFundRequest: (
    pubkeyHex: string,
    actorCidNumber: string,
    institutionAccount: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_multisig_safety_fund_request', {
      pubkeyHex,
      actorCidNumber,
      institutionAccount,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitProposeSafetyFund: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    actorCidNumber: string,
    institutionAccount: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_multisig_safety_fund', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      actorCidNumber,
      institutionAccount,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
