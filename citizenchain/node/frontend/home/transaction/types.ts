// 交易模块类型定义。

export type ColdWallet = {
  id: string;
  name: string;
  /** minerHot 使用本地矿工密钥签名；cold 使用离线 QR 签名。 */
  kind: 'minerHot' | 'cold';
  /** 矿工热钱包不可从钱包管理中删除。 */
  deletable: boolean;
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
