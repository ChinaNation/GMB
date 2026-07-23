// 交易模块类型定义。

export type ColdWallet = {
  id: string;
  name: string;
  /** minerHot 使用本地矿工密钥签名；cold 使用离线 QR 签名。 */
  kind: 'minerHot' | 'cold';
  /** 矿工热钱包不可从钱包管理中删除。 */
  deletable: boolean;
  /** 仅用于钱包界面展示的 SS58 地址（prefix 2027）。 */
  ss58_address: string;
  /** 账户 ID（小写 0x + 64 位十六进制）。 */
  account_id: string;
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

export type TransferDraft = {
  toAddress: string;
  amountYuan: number;
  remark: string;
};
