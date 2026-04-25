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
  finalizedHeight: number | null;
  syncing: boolean | null;
  specVersion: number | null;
  nodeVersion: string;
};

// 节点身份信息
export type NodeIdentity = {
  peerId: string | null;
  role: string | null;
};

// 全链发行总额
export type TotalIssuance = {
  totalIssuance: string | null;
};

// 永久质押金额
export type TotalStake = {
  totalStake: string | null;
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
  cpuHashrateMhs: number | null;
  gpuHashrateMhs: number | null;
  memoryMb: number | null;
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
  // 清算节点数量（当前阶段由后端硬编码 0 占位，待清算节点识别规则落地后填充）。
  clearingNodes: number;
  fullNodes: number;
  lightNodes: number;
  warning: string | null;
};

type OtherIframeTabItem = {
  key: string;
  title: string;
  contentType: 'iframe';
  url: string;
};

type OtherTextTabItem = {
  key: string;
  title: string;
  contentType: 'text';
  text: string;
};

export type OtherTabItem = OtherIframeTabItem | OtherTextTabItem;

export type OtherTabsPayload = {
  tabs: OtherTabItem[];
};
