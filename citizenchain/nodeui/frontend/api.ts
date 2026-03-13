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
  OtherTabsPayload,
  RewardWallet,
} from './types';

const ERROR_MAX_LENGTH = 500;

/** 提取错误消息并截断，防止超长或异常内容影响 UI。 */
export function sanitizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  return raw.length > ERROR_MAX_LENGTH
    ? raw.slice(0, ERROR_MAX_LENGTH) + '…'
    : raw;
}

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
  setNodeName: (nodeName: string, unlockPassword: string) =>
    invoke<NodeIdentity>('set_node_name', { nodeName, unlockPassword }),
  getMiningDashboard: () => invoke<MiningDashboard>('get_mining_dashboard'),
  getNetworkOverview: () => invoke<NetworkOverview>('get_network_overview'),
  getOtherTabsContent: () => invoke<OtherTabsPayload>('get_other_tabs_content'),
};
