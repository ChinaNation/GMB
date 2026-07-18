import type { Env } from '../types';
import { HttpError } from '../shared/http';

/// 稳定币充值购买公民币 · 静态配置真源。
///
/// 首期只两条入金轨:USDC→Base、USDT→Arbitrum(方案 B / WalletConnect)。
/// 后续加币/加链 = 往下面表里加一条配置,零新代码。
/// 合约地址是安全关键项:错地址 = 收假币,任何改动必须二次核对。

/// 支持的稳定币币种(首期两种)。
export type TopupToken = 'USDC' | 'USDT';

/// 部署网络:沙箱期用 testnet,上生产切 mainnet。
export type TopupNetwork = 'mainnet' | 'testnet';

/// 一条「币 + 链」入金轨的解析后配置。
export interface TopupRail {
  token: TopupToken;
  chain_id: number;
  /// ERC-20 代币合约地址(小写 0x),来自网络表默认 + Env 覆盖。
  token_contract: string;
  /// 代币精度(USDC/USDT 均为 6 位)。
  token_decimals: number;
  /// 该链 EVM JSON-RPC 的 Env 变量名(URL 值放 wrangler vars/secret,不硬编码)。
  rpc_env_key: 'TOPUP_BASE_RPC_URL' | 'TOPUP_ARBITRUM_RPC_URL';
  /// Env 中覆盖该币合约地址的变量名(testnet mock / 勘误用)。
  contract_env_key: 'TOPUP_USDC_CONTRACT' | 'TOPUP_USDT_CONTRACT';
  label: string;
}

/// 网络表默认合约地址(mainnet 为官方发行地址;testnet 由 Env 覆盖)。
interface RailTemplate {
  chain_id: number;
  default_contract: string;
  rpc_env_key: TopupRail['rpc_env_key'];
  contract_env_key: TopupRail['contract_env_key'];
  label: string;
}

const MAINNET_TEMPLATES: Readonly<Record<TopupToken, RailTemplate>> = {
  USDC: {
    chain_id: 8453,
    default_contract: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913',
    rpc_env_key: 'TOPUP_BASE_RPC_URL',
    contract_env_key: 'TOPUP_USDC_CONTRACT',
    label: 'USDC · Base',
  },
  USDT: {
    chain_id: 8453,
    // USDC/USDT 同走 Base(一条链、一种 gas)。Base 主网 USDT 合约(用户从钱包核对提供)。
    default_contract: '0xfde4c96c8593536e31f229ea8f37b2ada2699bb2',
    rpc_env_key: 'TOPUP_BASE_RPC_URL',
    contract_env_key: 'TOPUP_USDT_CONTRACT',
    label: 'USDT · Base',
  },
};

const TESTNET_TEMPLATES: Readonly<Record<TopupToken, RailTemplate>> = {
  USDC: {
    chain_id: 84532,
    // Circle 官方 Base Sepolia 测试 USDC;仍允许 Env 覆盖。
    default_contract: '0x036cbd53842c5426634e7929541ec2318f3dcf7e',
    rpc_env_key: 'TOPUP_BASE_RPC_URL',
    contract_env_key: 'TOPUP_USDC_CONTRACT',
    label: 'USDC · Base Sepolia',
  },
  USDT: {
    chain_id: 84532,
    // Base Sepolia 上 USDT 无官方测试币,用自部署 mock,合约地址由 Env 提供。
    default_contract: '',
    rpc_env_key: 'TOPUP_BASE_RPC_URL',
    contract_env_key: 'TOPUP_USDT_CONTRACT',
    label: 'USDT · Base Sepolia',
  },
};

const TOKEN_DECIMALS = 6;

/// 充值套餐(定价真源)。USDC/USDT 均按 1:1 美元、6 位精度,两币共用同一套金额。
/// pay_amount = 应付稳定币最小单位;coin_fen = 应发公民币分额(2 位精度)。
/// 两档单价不同 = 约 7% 批量折扣(已确认有意保留)。
export interface TopupPackage {
  package_id: string;
  pay_display: string;
  pay_amount: string;
  coin_display: string;
  coin_fen: string;
}

const PACKAGES: readonly TopupPackage[] = [
  // 15 USDC/USDT → 10,000.00 公民币:15 × 10^6 = 15000000;10000.00 × 100 = 1000000 分。
  { package_id: 'pkg_15', pay_display: '15', pay_amount: '15000000', coin_display: '10,000.00', coin_fen: '1000000' },
  // 1,400 USDC/USDT → 1,000,000.00 公民币:1400 × 10^6 = 1400000000;1000000.00 × 100 = 100000000 分。
  { package_id: 'pkg_1400', pay_display: '1400', pay_amount: '1400000000', coin_display: '1,000,000.00', coin_fen: '100000000' },
];

export function topupNetwork(env: Env): TopupNetwork {
  return env.TOPUP_NETWORK === 'mainnet' ? 'mainnet' : 'testnet';
}

/// 解析一条币轨:合并网络表默认合约 + Env 覆盖;合约为空视为该轨未配置。
export function topupRail(env: Env, token: TopupToken): TopupRail {
  const template = (topupNetwork(env) === 'mainnet' ? MAINNET_TEMPLATES : TESTNET_TEMPLATES)[token];
  const override = (env[template.contract_env_key] ?? '').trim().toLowerCase();
  // 合约地址:Env 覆盖优先(testnet mock / 勘误),否则用网络表内置默认(mainnet 两币均内置)。
  const contract = override || template.default_contract;
  return {
    token,
    chain_id: template.chain_id,
    token_contract: contract,
    token_decimals: TOKEN_DECIMALS,
    rpc_env_key: template.rpc_env_key,
    contract_env_key: template.contract_env_key,
    label: template.label,
  };
}

/// USDC 与 USDT 两轨**始终同时提供**(不因合约未配置而隐藏)。
/// mainnet 两币合约均内置;testnet 的 USDT mock 合约由 Env `TOPUP_USDT_CONTRACT` 提供。
export function topupRails(env: Env): TopupRail[] {
  return [topupRail(env, 'USDC'), topupRail(env, 'USDT')];
}

export function topupPackages(): readonly TopupPackage[] {
  return PACKAGES;
}

export function findPackage(packageId: string): TopupPackage | null {
  return PACKAGES.find((item) => item.package_id === packageId) ?? null;
}

export function isTopupToken(value: unknown): value is TopupToken {
  return value === 'USDC' || value === 'USDT';
}

/// 平台/国储会 EVM 收款地址(同一 EOA 跨链复用),小写返回。
export function topupRecvAddress(env: Env): string {
  const address = (env.TOPUP_RECV_ADDRESS ?? '').trim().toLowerCase();
  if (!isEvmAddress(address)) {
    throw new HttpError(503, 'topup_recv_unconfigured', 'EVM 收款地址未配置');
  }
  return address;
}

/// 取某条链的 EVM JSON-RPC URL(必须 https)。
export function railRpcUrl(env: Env, rail: TopupRail): string {
  const url = (env[rail.rpc_env_key] ?? '').trim();
  if (!url.startsWith('https://')) {
    throw new HttpError(503, 'topup_rpc_unconfigured', `${rail.label} 的 EVM RPC 未配置`);
  }
  return url;
}

/// 最小确认数:>0 时按 latest 计算确认数,=0 时按 finalized 区块判定。
export function topupMinConfirmations(env: Env): number {
  const parsed = Number.parseInt(env.TOPUP_MIN_CONFIRMATIONS ?? '', 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

export function isEvmAddress(value: string): boolean {
  return /^0x[0-9a-f]{40}$/.test(value);
}

export function isEvmTxHash(value: string): boolean {
  return /^0x[0-9a-f]{64}$/.test(value.toLowerCase());
}
