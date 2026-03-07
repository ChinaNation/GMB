import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api';
import { ChainSection } from './components/ChainSection';
import { IdentitySection } from './components/IdentitySection';
import type { ChainStatus, NodeIdentity, NodeStatus } from '../../types';

type Props = {
  onNodeActionBusyChange?: (busy: boolean) => void;
};

export function HomeNodeSection({ onNodeActionBusyChange }: Props) {
  const [status, setStatus] = useState<NodeStatus>({ running: false, state: 'stopped', pid: null });
  const [chain, setChain] = useState<ChainStatus>({ blockHeight: null });
  const [identity, setIdentity] = useState<NodeIdentity>({ nodeName: null, peerId: null, role: null });
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [showStartUnlockDialog, setShowStartUnlockDialog] = useState(false);
  const [startUnlockPassword, setStartUnlockPassword] = useState('');
  const [error, setError] = useState<string | null>(null);

  const loadHome = useCallback(async () => {
    const [s, c, i] = await Promise.allSettled([
      api.getNodeStatus(),
      api.getChainStatus(),
      api.getNodeIdentity(),
    ]);
    if (s.status === 'fulfilled') setStatus(s.value);
    if (c.status === 'fulfilled') setChain(c.value);
    if (i.status === 'fulfilled') setIdentity(i.value);
  }, []);

  useEffect(() => {
    void loadHome().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [loadHome]);

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
    const busy = starting || stopping;
    onNodeActionBusyChange?.(busy);
    return () => {
      onNodeActionBusyChange?.(false);
    };
  }, [onNodeActionBusyChange, starting, stopping]);

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
      await loadHome();
      setStartUnlockPassword('');
      setShowStartUnlockDialog(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  }, [loadHome, starting, stopping]);

  const onStop = useCallback(async () => {
    if (starting || stopping) return;
    setStopping(true);
    setError(null);
    try {
      const next = await api.stopNode();
      setStatus(next);
      await loadHome();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setStopping(false);
    }
  }, [loadHome, starting, stopping]);

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
    </>
  );
}
