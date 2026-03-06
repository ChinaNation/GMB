import { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from './api';
import DOMPurify from 'dompurify';
import { marked } from 'marked';
import { WalletSection } from './components/WalletSection';
import { NodeKeySection } from './components/NodeKeySection';
import { ChainSection } from './components/ChainSection';
import { IdentitySection } from './components/IdentitySection';
import type {
  BootnodeKey,
  ChainStatus,
  MiningDashboard,
  NetworkOverview,
  NodeIdentity,
  NodeStatus,
  RewardWallet,
} from './types';

type TabKey =
  | 'home'
  | 'mining'
  | 'network'
  | 'whitepaper'
  | 'party'
  | 'constitution'
  | 'settings';

const WHITEPAPER_HTML_URL =
  'https://chinanation.github.io/GMB/GMB_README.html';
const CONSTITUTION_RAW_URL =
  'https://raw.githubusercontent.com/ChinaNation/FRC/main/README.md';

function formatIncomeDisplay(raw: string): string {
  const normalized = raw.replace(/,/g, '').trim();
  const amount = Number(normalized);
  if (!Number.isFinite(amount)) {
    return raw;
  }
  return amount.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export default function App() {
  const [status, setStatus] = useState<NodeStatus>({ running: false, state: 'stopped', pid: null });
  const [wallet, setWallet] = useState<RewardWallet>({ address: null });
  const [nodeKey, setNodeKey] = useState<BootnodeKey>({
    nodeKey: null,
    peerId: null,
    institutionName: null,
  });
  const [chain, setChain] = useState<ChainStatus>({ blockHeight: null });
  const [identity, setIdentity] = useState<NodeIdentity>({ nodeName: null, peerId: null, role: null });
  const [mining, setMining] = useState<MiningDashboard>({
    income: {
      totalIncome: '0.00',
      totalFeeIncome: '0.00',
      totalRewardIncome: '0.00',
      todayIncome: '0.00',
    },
    records: [],
    resources: { cpuPercent: null, memoryMb: null, diskUsagePercent: null, nodeDataSizeMb: null },
  });
  const [network, setNetwork] = useState<NetworkOverview>({
    totalNodes: 0,
    onlineNodes: 0,
    guochuhuiNodes: 0,
    shengchuhuiNodes: 0,
    shengchuhangNodes: 0,
    fullNodes: 0,
    lightNodes: 0,
  });
  const [constitutionContent, setConstitutionContent] = useState<string>('');
  const [constitutionLoading, setConstitutionLoading] = useState(false);
  const [constitutionError, setConstitutionError] = useState<string | null>(null);
  const [tab, setTab] = useState<TabKey>('home');
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [showStartUnlockDialog, setShowStartUnlockDialog] = useState(false);
  const [startUnlockPassword, setStartUnlockPassword] = useState('');
  const [error, setError] = useState<string | null>(null);

  // 统一刷新页面所需状态，保持显示数据一致。
  const loadAll = useCallback(async () => {
    const [s, w, k, c, i] = await Promise.allSettled([
      api.getNodeStatus(),
      api.getRewardWallet(),
      api.getBootnodeKey(),
      api.getChainStatus(),
      api.getNodeIdentity(),
    ]);
    if (s.status === 'fulfilled') setStatus(s.value);
    if (w.status === 'fulfilled') setWallet(w.value);
    if (k.status === 'fulfilled') setNodeKey(k.value);
    if (c.status === 'fulfilled') setChain(c.value);
    if (i.status === 'fulfilled') setIdentity(i.value);
  }, []);

  useEffect(() => {
    void loadAll().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [loadAll]);

  const loadMining = useCallback(async () => {
    const data = await api.getMiningDashboard();
    setMining(data);
  }, []);

  const loadNetwork = useCallback(async () => {
    const data = await api.getNetworkOverview();
    setNetwork(data);
  }, []);

  const loadConstitution = useCallback(async () => {
    setConstitutionLoading(true);
    try {
      const res = await fetch(`${CONSTITUTION_RAW_URL}?t=${Date.now()}`, {
        cache: 'no-store',
      });
      if (!res.ok) {
        throw new Error(`公民宪法拉取失败: HTTP ${res.status}`);
      }
      const text = await res.text();
      setConstitutionContent(text);
      setConstitutionError(null);
    } catch (e) {
      setConstitutionError(e instanceof Error ? e.message : String(e));
    } finally {
      setConstitutionLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = globalThis.setInterval(() => {
      void Promise.all([api.getNodeStatus(), api.getChainStatus(), api.getNodeIdentity()])
        .then(([s, c, i]) => {
          setStatus(s);
          setChain(c);
          setIdentity(i);
        })
        .catch(() => undefined);
    }, 3000);
    return () => globalThis.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (tab !== 'mining') return;
    void loadMining().catch(() => undefined);
    const timer = globalThis.setInterval(() => {
      void loadMining().catch(() => undefined);
    }, 10000);
    return () => globalThis.clearInterval(timer);
  }, [loadMining, tab]);

  useEffect(() => {
    if (tab !== 'network') return;
    void loadNetwork().catch(() => undefined);
    const timer = globalThis.setInterval(() => {
      void loadNetwork().catch(() => undefined);
    }, 5000);
    return () => globalThis.clearInterval(timer);
  }, [loadNetwork, tab]);

  useEffect(() => {
    if (tab !== 'constitution') return;
    void loadConstitution().catch(() => undefined);
    const timer = globalThis.setInterval(() => {
      void loadConstitution().catch(() => undefined);
    }, 60000);
    return () => globalThis.clearInterval(timer);
  }, [loadConstitution, tab]);

  const constitutionHtml = useMemo(() => {
    if (!constitutionContent) return '';
    const parsed = marked.parse(constitutionContent, {
      gfm: true,
      breaks: false,
    });
    const html = typeof parsed === 'string' ? parsed : '';
    return DOMPurify.sanitize(html);
  }, [constitutionContent]);

  const onStart = useCallback(async (unlockPasswordInput: string) => {
    if (starting || stopping) return;
    const unlockPassword = unlockPasswordInput.trim();
    if (!unlockPassword) {
      setError('请输入设备开机密码');
      return;
    }
    setStarting(true);
    setError(null);
    try {
      const next = await api.startNode(unlockPassword);
      setStatus(next);
      await loadAll();
      setStartUnlockPassword('');
      setShowStartUnlockDialog(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  }, [loadAll, starting, stopping]);

  const onStop = useCallback(async () => {
    if (starting || stopping) return;
    setStopping(true);
    setError(null);
    try {
      const next = await api.stopNode();
      setStatus(next);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setStopping(false);
    }
  }, [loadAll, starting, stopping]);

  return (
    <div className="page">
      <nav className="top-nav">
        <button className={tab === 'home' ? 'active' : ''} onClick={() => setTab('home')}>首页</button>
        <button className={tab === 'mining' ? 'active' : ''} onClick={() => setTab('mining')}>挖矿</button>
        <button className={tab === 'network' ? 'active' : ''} onClick={() => setTab('network')}>网络</button>
        <button className={tab === 'whitepaper' ? 'active' : ''} onClick={() => setTab('whitepaper')}>白皮书</button>
        <button className={tab === 'party' ? 'active' : ''} onClick={() => setTab('party')}>公民党</button>
        <button className={tab === 'constitution' ? 'active' : ''} onClick={() => setTab('constitution')}>公民宪法</button>
        <button className={tab === 'settings' ? 'active' : ''} onClick={() => setTab('settings')}>设置</button>
      </nav>

      <main className="app">
        <section className="content">
        {tab === 'home' ? (
          <>
            <p className="status-line">
              <span className={`status-dot ${status.running ? 'running' : 'stopped'}`} />
              状态: {status.running ? '运行中' : '已停止'}
            </p>
            <div className="actions">
              <button
                onClick={() => {
                  setError(null);
                  setStartUnlockPassword('');
                  setShowStartUnlockDialog(true);
                }}
                disabled={starting || stopping || status.running}
              >
                {starting ? '启动中...' : '启动节点'}
              </button>
              <button onClick={onStop} disabled={starting || stopping || !status.running}>
                {stopping ? '停止中...' : '停止节点'}
              </button>
            </div>
            <ChainSection chain={chain} nodeRunning={status.running} />
            <IdentitySection
              identity={identity}
              onUpdated={setIdentity}
              disabled={starting || stopping}
            />
          </>
        ) : null}

        {tab === 'settings' ? (
          <>
            <WalletSection wallet={wallet} onUpdated={setWallet} disabled={starting || stopping} />
            <NodeKeySection
              nodeKey={nodeKey}
              onUpdated={setNodeKey}
              onApplied={() => {
                void loadAll();
              }}
              disabled={starting || stopping}
            />
          </>
        ) : null}

        {tab === 'mining' ? (
          <>
            <section className="section mining-section">
              <h2>挖矿收益</h2>
              <div className="mining-income-grid">
                <div className="metric-card">
                  <div className="metric-label">收益总额</div>
                  <div className="metric-value">
                    {formatIncomeDisplay(mining.income.totalIncome)}元
                    <span className="metric-value-currency">（公民币）</span>
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">累计手续费收益</div>
                  <div className="metric-value">
                    {formatIncomeDisplay(mining.income.totalFeeIncome)}元
                    <span className="metric-value-currency">（公民币）</span>
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">累计挖矿奖励</div>
                  <div className="metric-value">
                    {formatIncomeDisplay(mining.income.totalRewardIncome)}元
                    <span className="metric-value-currency">（公民币）</span>
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">今日收益</div>
                  <div className="metric-value">
                    {formatIncomeDisplay(mining.income.todayIncome)}元
                    <span className="metric-value-currency">（公民币）</span>
                  </div>
                </div>
              </div>
            </section>

            <section className="section mining-section">
              <h2>资源监控</h2>
              <div className="mining-income-grid">
                <div className="metric-card">
                  <div className="metric-label">CPU 占用</div>
                  <div className="metric-value">
                    {mining.resources.cpuPercent == null ? '未知' : `${mining.resources.cpuPercent.toFixed(1)}%`}
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">内存占用</div>
                  <div className="metric-value">
                    {mining.resources.memoryMb == null ? '未知' : `${mining.resources.memoryMb} MB`}
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">磁盘占用</div>
                  <div className="metric-value">
                    {mining.resources.diskUsagePercent == null ? '未知' : `${mining.resources.diskUsagePercent.toFixed(1)}%`}
                  </div>
                </div>
                <div className="metric-card">
                  <div className="metric-label">节点数据大小</div>
                  <div className="metric-value">
                    {mining.resources.nodeDataSizeMb == null ? '未知' : `${mining.resources.nodeDataSizeMb} MB`}
                  </div>
                </div>
              </div>
            </section>

            <section className="section">
              <h2>出块记录</h2>
              <div className="table-wrap">
                <table className="mining-table">
                  <thead>
                    <tr>
                      <th>区块高度</th>
                      <th>时间</th>
                      <th>手续费</th>
                      <th>铸块奖励</th>
                      <th>区块作者</th>
                    </tr>
                  </thead>
                  <tbody>
                    {mining.records.length === 0 ? (
                      <tr>
                        <td colSpan={5} className="empty-cell">暂无数据</td>
                      </tr>
                    ) : (
                      mining.records.map((row) => (
                        <tr key={row.blockHeight}>
                          <td>{row.blockHeight}</td>
                          <td>{row.timestampMs ? new Date(row.timestampMs).toLocaleString() : '未知'}</td>
                          <td>{row.fee}</td>
                          <td>{row.blockReward}</td>
                          <td>{row.author}</td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </section>
          </>
        ) : null}

        {tab === 'network' ? (
          <section className="section network-section">
            <h2>网络</h2>
            <div className="mining-income-grid">
              <div className="metric-card">
                <div className="metric-label">总节点数</div>
                <div className="metric-value">{network.totalNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">在线节点</div>
                <div className="metric-value">{network.onlineNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">国储会节点</div>
                <div className="metric-value">{network.guochuhuiNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">省储会节点</div>
                <div className="metric-value">{network.shengchuhuiNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">省储行节点</div>
                <div className="metric-value">{network.shengchuhangNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">全节点</div>
                <div className="metric-value">{network.fullNodes}</div>
              </div>
              <div className="metric-card">
                <div className="metric-label">轻节点</div>
                <div className="metric-value">{network.lightNodes}</div>
              </div>
            </div>
          </section>
        ) : null}

        {tab === 'whitepaper' ? (
          <section className="section whitepaper-section">
              <iframe
                className="whitepaper-iframe"
                src={WHITEPAPER_HTML_URL}
                title="白皮书"
              />
          </section>
        ) : null}

        {tab === 'party' ? (
          <section className="section">
            <h2>公民党</h2>
            <p>公民党内容入口（待接入）。</p>
          </section>
        ) : null}

        {tab === 'constitution' ? (
          <section className="section whitepaper-section">
            {constitutionLoading ? <p className="whitepaper-meta">加载中...</p> : null}
            {constitutionError ? <p className="whitepaper-error">{constitutionError}</p> : null}
            {constitutionHtml ? (
              <article
                className="whitepaper-content markdown-body"
                dangerouslySetInnerHTML={{ __html: constitutionHtml }}
              />
            ) : (
              <div className="whitepaper-content">暂无内容</div>
            )}
          </section>
        ) : null}
        </section>

        {showStartUnlockDialog ? (
          <div className="unlock-modal-mask" onClick={() => setShowStartUnlockDialog(false)}>
            <div className="unlock-modal" onClick={(e) => e.stopPropagation()}>
              <h3>启动节点解锁</h3>
              <input
                className="unlock-password-input"
                type="password"
                value={startUnlockPassword}
                onChange={(e) => setStartUnlockPassword(e.target.value)}
                placeholder="请输入设备开机密码"
                disabled={starting || stopping}
              />
              <div className="unlock-modal-actions">
                <button
                  onClick={() => setShowStartUnlockDialog(false)}
                  disabled={starting}
                >
                  取消
                </button>
                <button
                  onClick={() => {
                    void onStart(startUnlockPassword);
                  }}
                  disabled={starting || stopping || status.running}
                >
                  {starting ? '启动中...' : '确认启动'}
                </button>
              </div>
            </div>
          </div>
        ) : null}

        {error ? <pre className="error">{error}</pre> : null}
      </main>
    </div>
  );
}
