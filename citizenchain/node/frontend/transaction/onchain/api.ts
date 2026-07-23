import { invoke } from '../../tauri';
import type { ColdWallet, TransferSignRequestResult, TransferSubmitResult, WalletStore } from './types';

// 首页交易面板专用 Tauri API，对齐后端 src/transaction/onchain_transaction。
export const transactionApi = {
  getWallets: () => invoke<WalletStore>('get_wallets'),
  addWallet: (name: string, ss58_address: string) =>
    invoke<ColdWallet>('add_wallet', { name, ss58_address }),
  removeWallet: (walletId: string) =>
    invoke<WalletStore>('remove_wallet', { wallet_id: walletId }),
  setActiveWallet: (walletId: string) =>
    invoke<WalletStore>('set_active_wallet', { wallet_id: walletId }),
  getWalletBalance: (account_id: string) =>
    invoke<string | null>('get_wallet_balance', { account_id }),
  buildTransferRequest: (signer_public_key: string, toSs58Address: string, amountYuan: number, remark: string) =>
    invoke<TransferSignRequestResult>('build_transfer_request', {
      signer_public_key,
      to_ss58_address: toSs58Address,
      amount_yuan: amountYuan,
      remark,
    }),
  submitMinerTransfer: (toAddress: string, amountYuan: number, remark: string, unlockPassword: string) =>
    invoke<TransferSubmitResult>('submit_miner_transfer', {
      to_ss58_address: toAddress,
      amount_yuan: amountYuan,
      remark,
      unlock_password: unlockPassword,
    }),
  submitTransfer: (
    requestId: string,
    expected_signer_public_key: string,
    expectedPayloadHash: string,
    callDataHex: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<TransferSubmitResult>('submit_transfer', {
      request_id: requestId,
      expected_signer_public_key,
      expected_payload_hash: expectedPayloadHash,
      call_data_hex: callDataHex,
      sign_nonce: signNonce,
      sign_block_number: signBlockNumber,
      response_json: responseJson,
    }),
};
