import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

type NodeStatus = {
  running: boolean;
  state: string;
  pid: number | null;
};

export default function App() {
  const [status, setStatus] = useState<NodeStatus>({ running: false, state: 'stopped', pid: null });
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadStatus = useCallback(async () => {
    const next = await invoke<NodeStatus>('get_node_status');
    setStatus(next);
  }, []);

  useEffect(() => {
    void loadStatus().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [loadStatus]);

  const startNode = useCallback(async () => {
    if (starting || stopping) return;
    setStarting(true);
    setError(null);
    try {
      const next = await invoke<NodeStatus>('start_node');
      setStatus(next);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      void loadStatus();
    } finally {
      setStarting(false);
    }
  }, [loadStatus, starting, stopping]);

  const stopNode = useCallback(async () => {
    if (starting || stopping) return;
    setStopping(true);
    setError(null);
    try {
      const next = await invoke<NodeStatus>('stop_node');
      setStatus(next);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      void loadStatus();
    } finally {
      setStopping(false);
    }
  }, [loadStatus, starting, stopping]);

  return (
    <main className="app">
      <h1>CitizenChain NodeUI</h1>
      <p>状态: {status.running ? '运行中' : '已停止'}</p>
      <p>PID: {status.pid ?? '-'}</p>

      <div className="actions">
        <button onClick={startNode} disabled={starting || stopping || status.running}>
          {starting ? '启动中...' : '启动节点'}
        </button>
        <button onClick={stopNode} disabled={starting || stopping || !status.running}>
          {stopping ? '停止中...' : '停止节点'}
        </button>
      </div>

      {error ? <pre className="error">{error}</pre> : null}
    </main>
  );
}
