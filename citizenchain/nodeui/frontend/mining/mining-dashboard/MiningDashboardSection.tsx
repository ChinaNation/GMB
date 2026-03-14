import { useCallback, useEffect, useRef, useState } from 'react';
import { api, sanitizeError } from '../../api';
import type { MiningDashboard } from '../../types';

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

export function MiningDashboardSection() {
  const [mining, setMining] = useState<MiningDashboard>({
    income: {
      totalIncome: '0.00',
      totalFeeIncome: '0.00',
      totalRewardIncome: '0.00',
      todayIncome: '0.00',
    },
    records: [],
    resources: { cpuPercent: null, memoryMb: null, diskUsagePercent: null, nodeDataSizeMb: null },
    warning: null,
  });
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const mountedRef = useRef<boolean>(true);
  const requestIdRef = useRef<number>(0);

  const loadMining = useCallback(async () => {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    try {
      const data = await api.getMiningDashboard();
      if (!mountedRef.current || requestId !== requestIdRef.current) {
        return;
      }
      setMining(data);
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
    void loadMining();
    const timer = globalThis.setInterval(() => {
      void loadMining();
    }, 10000);
    return () => {
      mountedRef.current = false;
      globalThis.clearInterval(timer);
    };
  }, [loadMining]);

  return (
    <>
      <section className="section mining-section">
        <h2>挖矿收益</h2>
        <div className="mining-income-grid">
          <div className="metric-card">
            <div className="metric-label">收益总额</div>
            <div className="metric-value">
              {loading ? (
                '加载中...'
              ) : (
                <>
                  {formatIncomeDisplay(mining.income.totalIncome)}元
                  <span className="metric-value-currency">（公民币）</span>
                </>
              )}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">累计手续费收益</div>
            <div className="metric-value">
              {loading ? (
                '加载中...'
              ) : (
                <>
                  {formatIncomeDisplay(mining.income.totalFeeIncome)}元
                  <span className="metric-value-currency">（公民币）</span>
                </>
              )}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">累计挖矿奖励</div>
            <div className="metric-value">
              {loading ? (
                '加载中...'
              ) : (
                <>
                  {formatIncomeDisplay(mining.income.totalRewardIncome)}元
                  <span className="metric-value-currency">（公民币）</span>
                </>
              )}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">今日收益</div>
            <div className="metric-value">
              {loading ? (
                '加载中...'
              ) : (
                <>
                  {formatIncomeDisplay(mining.income.todayIncome)}元
                  <span className="metric-value-currency">（公民币）</span>
                </>
              )}
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
              {loading ? '加载中...' : (mining.resources.cpuPercent == null ? '未知' : `${mining.resources.cpuPercent.toFixed(5)}%`)}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">内存占用</div>
            <div className="metric-value">
              {loading ? '加载中...' : (mining.resources.memoryMb == null ? '未知' : `${mining.resources.memoryMb} MB`)}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">磁盘占用</div>
            <div className="metric-value">
              {loading ? '加载中...' : (mining.resources.diskUsagePercent == null ? '未知' : `${mining.resources.diskUsagePercent.toFixed(5)}%`)}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-label">节点数据大小</div>
            <div className="metric-value">
              {loading ? '加载中...' : (mining.resources.nodeDataSizeMb == null ? '未知' : `${mining.resources.nodeDataSizeMb} MB`)}
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
                  <td colSpan={5} className="empty-cell">{loading ? '加载中...' : '暂无数据'}</td>
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
      {mining.warning ? <pre className="error">{mining.warning}</pre> : null}
      {error ? <pre className="error">{error}</pre> : null}
    </>
  );
}
