import { useCallback, useEffect, useRef, useState } from 'react';
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
  const [loading, setLoading] = useState<boolean>(true);
  const mountedRef = useRef<boolean>(true);
  const requestIdRef = useRef<number>(0);

  const loadNetwork = useCallback(async () => {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    try {
      const data = await api.getNetworkOverview();
      if (!mountedRef.current || requestId !== requestIdRef.current) {
        return;
      }
      setNetwork(data);
      setError(null);
    } catch (e) {
      if (!mountedRef.current || requestId !== requestIdRef.current) {
        return;
      }
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (mountedRef.current && requestId === requestIdRef.current) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    void loadNetwork();
    const timer = globalThis.setInterval(() => {
      void loadNetwork();
    }, 5000);
    return () => {
      mountedRef.current = false;
      globalThis.clearInterval(timer);
    };
  }, [loadNetwork]);

  return (
    <section className="section network-section">
      <h2>网络</h2>
      <div className="network-overview-grid">
        <div className="metric-card">
          <div className="metric-label">总节点数</div>
          <div className="metric-value">{loading ? '加载中...' : network.totalNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">在线节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.onlineNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">国储会节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.guochuhuiNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">省储会节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.shengchuhuiNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">省储行节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.shengchuhangNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">全节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.fullNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">轻节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.lightNodes}</div>
        </div>
      </div>
      {network.warning ? <pre className="warning">{network.warning}</pre> : null}
      {error ? <pre className="error">{error}</pre> : null}
    </section>
  );
}
