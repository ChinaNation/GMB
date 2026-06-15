import { invoke } from '../core/tauri';
import type {
  BootnodeKey,
  BootnodeOption,
  CommunicationNodeState,
  GrandpaKey,
  NodeMode,
  NodeModeState,
  RewardWallet,
} from './types';

// 设置页专用 Tauri API。
export const settingsApi = {
  getNodeMode: () => invoke<NodeModeState>('get_node_mode'),
  setNodeMode: (mode: NodeMode) => invoke<NodeModeState>('set_node_mode', { mode }),
  getCommunicationNode: () => invoke<CommunicationNodeState>('get_communication_node'),
  setCommunicationNodeEnabled: (enabled: boolean) =>
    invoke<CommunicationNodeState>('set_communication_node_enabled', { enabled }),
  getRewardWallet: () => invoke<RewardWallet>('get_reward_wallet'),
  setRewardWallet: (address: string, unlockPassword: string) =>
    invoke<RewardWallet>('set_reward_wallet', { address, unlockPassword }),
  getLocalMinerAddress: () => invoke<string | null>('get_local_miner_address'),
  getBootnodeKey: () => invoke<BootnodeKey>('get_bootnode_key'),
  getGrandpaKey: () => invoke<GrandpaKey>('get_grandpa_key'),
  setBootnodeKey: (nodeKey: string, unlockPassword: string) =>
    invoke<BootnodeKey>('set_bootnode_key', { nodeKey, unlockPassword }),
  setGrandpaKey: (key: string, unlockPassword: string) =>
    invoke<GrandpaKey>('set_grandpa_key', { key, unlockPassword }),
  getGenesisBootnodeOptions: () =>
    invoke<BootnodeOption[]>('get_genesis_bootnode_options'),
  prepareDesktopUpdate: () => invoke<void>('prepare_desktop_update'),
};
