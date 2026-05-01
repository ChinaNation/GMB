import { invoke } from '../../core/tauri';
import type { ColdWallet, TransferSignRequestResult, TransferSubmitResult, WalletStore } from './types';

// 首页交易面板专用 Tauri API，对齐后端 src/home/transaction。
export const transactionApi = {
  getWallets: () => invoke<WalletStore>('get_wallets'),
  addWallet: (name: string, address: string) =>
    invoke<ColdWallet>('add_wallet', { name, address }),
  removeWallet: (walletId: string) =>
    invoke<WalletStore>('remove_wallet', { walletId }),
  setActiveWallet: (walletId: string) =>
    invoke<WalletStore>('set_active_wallet', { walletId }),
  getWalletBalance: (pubkeyHex: string) =>
    invoke<string | null>('get_wallet_balance', { pubkeyHex }),
  buildTransferRequest: (pubkeyHex: string, toAddress: string, amountYuan: number) =>
    invoke<TransferSignRequestResult>('build_transfer_request', { pubkeyHex, toAddress, amountYuan }),
  submitMinerTransfer: (toAddress: string, amountYuan: number, unlockPassword: string) =>
    invoke<TransferSubmitResult>('submit_miner_transfer', { toAddress, amountYuan, unlockPassword }),
  submitTransfer: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    callDataHex: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<TransferSubmitResult>('submit_transfer', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      callDataHex,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
