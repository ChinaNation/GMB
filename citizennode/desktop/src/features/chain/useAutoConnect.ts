import { useEffect } from 'react';
import { connectNode } from '../../services/rpc/polkadot';
import { useSessionStore } from '../../stores/session';

export function useAutoConnect(): void {
  const endpoint = useSessionStore((state) => state.endpoint);
  const setState = useSessionStore((state) => state.setState);

  useEffect(() => {
    let cancelled = false;

    const run = async () => {
      try {
        setState('connecting');
        await connectNode(endpoint);
        if (!cancelled) {
          setState('connected');
        }
      } catch (e) {
        if (!cancelled) {
          const message = e instanceof Error ? e.message : '节点连接失败';
          setState('error', message);
        }
      }
    };

    void run();

    return () => {
      cancelled = true;
    };
  }, [endpoint, setState]);
}
