// 挖矿 tab 类型，对齐后端 src/mining/dashboard 与 src/mining/network-overview。

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
  // 清算节点数量由后端 network-overview 聚合。
  clearingNodes: number;
  fullNodes: number;
  lightNodes: number;
  warning: string | null;
};
