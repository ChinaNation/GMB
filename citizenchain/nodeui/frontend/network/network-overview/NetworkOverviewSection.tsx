import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api';
import type { NetworkOverview } from '../../types';

export function NetworkOverviewSection() {
  const [network, setNetwork] = useState<NetworkOverview>({
    totalNodes: 0,
    onlineNodes: 0,
    guochuhuiNodes: 0,
    shengchuhuiNodes: 0,
    shengchuhangNodes: 0,
    fullNodes: 0,
    lightNodes: 0,
    warning: null,
  });
  const [error, setError] = useState<string | null>(null);

  const loadNetwork = useCallback(async () => {
    const data = await api.getNetworkOverview();
    setNetwork(data);
    setError(null);
  }, []);

  useEffect(() => {
    void loadNetwork().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    const timer = globalThis.setInterval(() => {
      void loadNetwork().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    }, 5000);
    return () => globalThis.clearInterval(timer);
  }, [loadNetwork]);

  return (
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
      {network.warning ? <pre className="error">{network.warning}</pre> : null}
      {error ? <pre className="error">{error}</pre> : null}
    </section>
  );
}
