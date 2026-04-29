// 交易模块类型定义。

export type ColdWallet = {
  id: string;
  name: string;
  /** SS58 地址（prefix 2027）。 */
  address: string;
  /** 32 字节公钥（64 位 hex，无 0x 前缀）。 */
  pubkeyHex: string;
  createdAt: number;
};

export type WalletStore = {
  wallets: ColdWallet[];
  activeId: string | null;
};

export type TransferSignRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  signNonce: number;
  signBlockNumber: number;
  callDataHex: string;
  feeYuan: number;
};

export type TransferSubmitResult = {
  txHash: string;
};
