// 节点基础状态
export type NodeStatus = {
  running: boolean;
  state: string;
  pid: number | null;
};

// 收款钱包设置
export type RewardWallet = {
  address: string | null;
};

// 引导节点 node-key 设置
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

// 链高度与同步状态
export type ChainStatus = {
  blockHeight: number | null;
};

// 节点身份信息
export type NodeIdentity = {
  nodeName: string | null;
  peerId: string | null;
  role: string | null;
};

export type MiningIncome = {
  totalIncome: string;
  totalFeeIncome: string;
  totalRewardIncome: string;
  todayIncome: string;
};

export type MiningBlockRecord = {
  blockHeight: number;
  timestampMs: number | null;
  fee: string;
  blockReward: string;
  author: string;
};

export type ResourceUsage = {
  cpuPercent: number | null;
  memoryMb: number | null;
  diskUsagePercent: number | null;
  nodeDataSizeMb: number | null;
};

export type MiningDashboard = {
  income: MiningIncome;
  records: MiningBlockRecord[];
  resources: ResourceUsage;
  warning: string | null;
};

export type NetworkOverview = {
  totalNodes: number;
  onlineNodes: number;
  guochuhuiNodes: number;
  shengchuhuiNodes: number;
  shengchuhangNodes: number;
  fullNodes: number;
  lightNodes: number;
  warning: string | null;
};

export type OtherTabItem = {
  key: string;
  title: string;
  contentType: string;
  url: string | null;
  text: string | null;
};

export type OtherTabsPayload = {
  tabs: OtherTabItem[];
};
