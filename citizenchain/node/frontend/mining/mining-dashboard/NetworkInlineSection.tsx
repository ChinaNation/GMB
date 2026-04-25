// 挖矿页内嵌的网络统计面板：3×2 卡片网格。
// 位置：由 MiningDashboardSection 注入在"资源监控"与"出块记录"之间。
// 设计决策：
//   - 行 1：总节点数 ｜ 在线节点
//   - 行 2：治理节点（国储会 ｜ 省储会 ｜ 省储行 单卡合并展示）｜ 清算节点（占位，后端当前固定返回 0）
//   - 行 3：全节点 ｜ 轻节点
// 数据轮询：独立 5 秒间隔，与 MiningDashboardSection 的 10 秒收益轮询解耦，
// 因为网络拓扑变化频率高于收益刷新频率。
import { useCallback, useEffect, useRef, useState } from 'react';
import { api, sanitizeError } from '../../api';
import type { NetworkOverview } from '../../types';

export function NetworkInlineSection() {
  const [network, setNetwork] = useState<NetworkOverview>({
    totalNodes: 0,
    onlineNodes: 0,
    guochuhuiNodes: 0,
    shengchuhuiNodes: 0,
    shengchuhangNodes: 0,
    clearingNodes: 0,
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
      setError(sanitizeError(e));
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

  // 治理节点 3 列子网格：每列一个数字 + 其名称，数字和名称上下对齐。
  const governanceCols: Array<{ name: string; value: number }> = [
    { name: '国储会', value: network.guochuhuiNodes },
    { name: '省储会', value: network.shengchuhuiNodes },
    { name: '省储行', value: network.shengchuhangNodes },
  ];

  return (
    <section className="section network-inline-section">
      <h2>网络</h2>
      <div className="network-inline-grid">
        <div className="metric-card">
          <div className="metric-label">总节点数</div>
          <div className="metric-value">{loading ? '加载中...' : network.totalNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">在线节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.onlineNodes}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">治理节点</div>
          <div className="governance-node-grid">
            {governanceCols.map((col) => (
              <div key={col.name} className="governance-node-col">
                <div className="governance-node-value">
                  {loading ? '—' : col.value}
                </div>
                <div className="governance-node-name">{col.name}</div>
              </div>
            ))}
          </div>
        </div>
        <div className="metric-card">
          <div className="metric-label">清算节点</div>
          <div className="metric-value">{loading ? '加载中...' : network.clearingNodes}</div>
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
