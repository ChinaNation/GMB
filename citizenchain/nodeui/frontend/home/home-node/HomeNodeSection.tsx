import { useCallback, useEffect, useRef, useState } from 'react';
import { api, sanitizeError } from '../../api';
import { ChainSection } from './components/ChainSection';
import { IdentitySection } from './components/IdentitySection';
import type { ChainStatus, NodeIdentity, NodeStatus } from '../../types';

const PARTIAL_REFRESH_ERROR_PREFIX = '部分数据刷新失败：';

type Props = {
  onNodeActionBusyChange?: (busy: boolean) => void;
};

export function HomeNodeSection({ onNodeActionBusyChange }: Props) {
  const [status, setStatus] = useState<NodeStatus>({ running: false, state: 'stopped', pid: null });
  const [chain, setChain] = useState<ChainStatus>({ blockHeight: null, finalizedHeight: null, syncing: null });
  const [identity, setIdentity] = useState<NodeIdentity>({ nodeName: null, peerId: null, role: null });
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [showStartUnlockDialog, setShowStartUnlockDialog] = useState(false);
  const [startUnlockPassword, setStartUnlockPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const refreshInFlightRef = useRef(false);

  const loadHome = useCallback(async (silent: boolean) => {
    const [s, c, i] = await Promise.allSettled([
      api.getNodeStatus(),
      api.getChainStatus(),
      api.getNodeIdentity(),
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
      if (starting || stopping || refreshInFlightRef.current) {
        return;
      }
      void runLoadHome(true).catch(() => undefined);
    }, 3000);
    return () => globalThis.clearInterval(timer);
  }, [runLoadHome, starting, stopping]);

  useEffect(() => {
    onNodeActionBusyChange?.(starting || stopping);
  }, [onNodeActionBusyChange, starting, stopping]);

  useEffect(() => {
    return () => {
      onNodeActionBusyChange?.(false);
    };
  }, [onNodeActionBusyChange]);

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
      await runLoadHome(false);
      setShowStartUnlockDialog(false);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setStartUnlockPassword('');
      setStarting(false);
    }
  }, [runLoadHome, starting, stopping]);

  const onStop = useCallback(async () => {
    if (starting || stopping) return;
    setStopping(true);
    setError(null);
    try {
      const next = await api.stopNode();
      setStatus(next);
      await runLoadHome(false);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setStopping(false);
    }
  }, [runLoadHome, starting, stopping]);

  const closeStartUnlockDialog = useCallback(() => {
    if (starting || stopping) return;
    setShowStartUnlockDialog(false);
  }, [starting, stopping]);

  return (
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

      {showStartUnlockDialog ? (
        <div className="unlock-modal-mask" onClick={closeStartUnlockDialog}>
          <div className="unlock-modal" onClick={(e) => e.stopPropagation()}>
            <h3>启动节点解锁</h3>
            <input
              className="unlock-password-input"
              type="password"
              value={startUnlockPassword}
              onChange={(e) => setStartUnlockPassword(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  void onStart(startUnlockPassword);
                }
              }}
              placeholder="请输入设备开机密码"
              disabled={starting || stopping}
            />
            <div className="unlock-modal-actions">
              <button
                onClick={closeStartUnlockDialog}
                disabled={starting || stopping}
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
    </>
  );
}
