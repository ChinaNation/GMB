import { useCallback, useEffect, useRef, useState } from 'react';
import { api, sanitizeError } from '../../api';
import { ChainSection } from './components/ChainSection';
import { IdentitySection } from './components/IdentitySection';
import { IssuanceSection } from './components/IssuanceSection';
import type { ChainStatus, NodeIdentity, NodeStatus, TotalIssuance, TotalStake } from '../../types';

const PARTIAL_REFRESH_ERROR_PREFIX = '部分数据刷新失败：';

// 节点生命周期与 App 进程绑定（开 App = 启节点 / 退 App = 停节点 / 关窗 = 最小化）。
// 此组件只负责展示节点运行状态与链信息，不再持有启动/停止控制。
export function HomeNodeSection() {
  const [status, setStatus] = useState<NodeStatus>({ running: false, state: 'stopped', pid: null });
  const [chain, setChain] = useState<ChainStatus>({ blockHeight: null, finalizedHeight: null, syncing: null, specVersion: null, nodeVersion: '' });
  const [identity, setIdentity] = useState<NodeIdentity>({ peerId: null, role: null });
  const [issuance, setIssuance] = useState<TotalIssuance>({ totalIssuance: null });
  const [stake, setStake] = useState<TotalStake>({ totalStake: null });
  const [error, setError] = useState<string | null>(null);
  const refreshInFlightRef = useRef(false);

  const loadHome = useCallback(async (silent: boolean) => {
    const [s, c, i, t, k] = await Promise.allSettled([
      api.getNodeStatus(),
      api.getChainStatus(),
      api.getNodeIdentity(),
      api.getTotalIssuance(),
      api.getTotalStake(),
    ]);
    let successCount = 0;
    const failures: string[] = [];
    const applyResult = <T,>(
      result: PromiseSettledResult<T>,
      onFulfilled: (value: T) => void,
    ) => {
      if (result.status === 'fulfilled') {
        onFulfilled(result.value);
        successCount += 1;
        return;
      }
      failures.push(sanitizeError(result.reason));
    };

    applyResult(s, setStatus);
    applyResult(c, setChain);
    applyResult(i, setIdentity);
    applyResult(t, setIssuance);
    applyResult(k, setStake);

    if (successCount === 0) {
      throw new Error(failures[0] ?? '首页数据加载失败');
    }
    if (failures.length > 0) {
      if (!silent) {
        setError(`${PARTIAL_REFRESH_ERROR_PREFIX}${failures[0]}`);
      }
      return;
    }

    if (!silent) {
      setError(null);
      return;
    }

    setError((prev) => {
      if (!prev) return null;
      return prev.startsWith(PARTIAL_REFRESH_ERROR_PREFIX) ? null : prev;
    });
  }, []);

  const runLoadHome = useCallback(async (silent: boolean) => {
    if (refreshInFlightRef.current) return;
    refreshInFlightRef.current = true;
    try {
      await loadHome(silent);
    } finally {
      refreshInFlightRef.current = false;
    }
  }, [loadHome]);

  useEffect(() => {
    void runLoadHome(false).catch((e) => setError(sanitizeError(e)));
  }, [runLoadHome]);

  useEffect(() => {
    const timer = globalThis.setInterval(() => {
      if (refreshInFlightRef.current) {
        return;
      }
      void runLoadHome(true).catch(() => undefined);
    }, 3000);
    return () => globalThis.clearInterval(timer);
  }, [runLoadHome]);

  return (
    <>
      <p className="status-line">
        <span className={`status-dot ${status.running ? 'running' : 'stopped'}`} />
        状态: {status.running ? '运行中' : '已停止'}
      </p>
      <ChainSection chain={chain} nodeRunning={status.running} />
      <IdentitySection identity={identity} />
      <IssuanceSection issuance={issuance} stake={stake} />
      {error ? <pre className="error">{error}</pre> : null}
    </>
  );
}
