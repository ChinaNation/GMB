// 设置页类型，对齐后端 src/settings 返回值。

export type RewardWallet = {
  address: string | null;
};

export type BootnodeKey = {
  nodeKey: string | null;
  peerId: string | null;
  institutionName: string | null;
};

export type GrandpaKey = {
  key: string | null;
  institutionName: string | null;
};

export type BootnodeOption = {
  name: string;
  peerId: string;
};
