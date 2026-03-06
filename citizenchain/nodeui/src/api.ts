import { invoke } from '@tauri-apps/api/core';
import type {
  BootnodeKey,
  BootnodeOption,
  ChainStatus,
  GrandpaKey,
  MiningDashboard,
  NetworkOverview,
  NodeIdentity,
  NodeStatus,
  RewardWallet,
} from './types';

// 统一封装所有 Tauri 命令调用，避免组件里散落 invoke 字符串。
export const api = {
  getNodeStatus: () => invoke<NodeStatus>('get_node_status'),
  startNode: (unlockPassword: string) =>
    invoke<NodeStatus>('start_node', { unlockPassword }),
  stopNode: () => invoke<NodeStatus>('stop_node'),

  getRewardWallet: () => invoke<RewardWallet>('get_reward_wallet'),
  setRewardWallet: (address: string, unlockPassword: string) =>
    invoke<RewardWallet>('set_reward_wallet', { address, unlockPassword }),

  getBootnodeKey: () => invoke<BootnodeKey>('get_bootnode_key'),
  getGrandpaKey: () => invoke<GrandpaKey>('get_grandpa_key'),
  setBootnodeKey: (nodeKey: string, unlockPassword: string) =>
    invoke<BootnodeKey>('set_bootnode_key', { nodeKey, unlockPassword }),
  setGrandpaKey: (key: string, unlockPassword: string) =>
    invoke<GrandpaKey>('set_grandpa_key', { key, unlockPassword }),
  getGenesisBootnodeOptions: () =>
    invoke<BootnodeOption[]>('get_genesis_bootnode_options'),

  getChainStatus: () => invoke<ChainStatus>('get_chain_status'),
  getNodeIdentity: () => invoke<NodeIdentity>('get_node_identity'),
  setNodeName: (nodeName: string) => invoke<NodeIdentity>('set_node_name', { nodeName }),
  getMiningDashboard: () => invoke<MiningDashboard>('get_mining_dashboard'),
  getNetworkOverview: () => invoke<NetworkOverview>('get_network_overview'),
};
