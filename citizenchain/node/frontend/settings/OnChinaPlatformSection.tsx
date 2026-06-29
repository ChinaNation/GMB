import { useState } from 'react';
import { sanitizeError } from '../core/tauri';
import { settingsApi as api } from './api';
import type { OnChinaPlatformState } from './types';

type Props = {
  platform: OnChinaPlatformState | null;
  onUpdated: (next: OnChinaPlatformState) => void;
};

const fallbackUrl = 'https://onchina.local:8964';

export function OnChinaPlatformSection({ platform, onUpdated }: Props) {
  const [pendingAction, setPendingAction] = useState<'start' | 'stop' | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const running = platform?.running ?? false;
  const url = platform?.url ?? fallbackUrl;
  const actionText = running ? '关闭' : '启动';

  const confirmAction = async () => {
    if (!pendingAction) return;
    setSaving(true);
    setError(null);
    try {
      const next = pendingAction === 'start'
        ? await api.startOnChinaPlatform()
        : await api.stopOnChinaPlatform();
      onUpdated(next);
      setPendingAction(null);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="section settings-onchina-platform-section">
      <div className="onchina-platform-row">
        <span className="onchina-platform-label">链上中国平台</span>
        <span className={`onchina-platform-status ${running ? 'enabled' : 'disabled'}`}>
          {running ? '已开启' : '未开启'}
        </span>
        <span className="onchina-platform-url">{url}</span>
        <button
          type="button"
          className="onchina-platform-button"
          disabled={!platform || saving}
          onClick={() => setPendingAction(running ? 'stop' : 'start')}
        >
          {saving ? '处理中' : actionText}
        </button>
      </div>
      {error ? <p className="section-inline-error">{error}</p> : null}
      {pendingAction ? (
        <div className="communication-node-confirm-mask" onClick={() => !saving && setPendingAction(null)}>
          <div className="communication-node-confirm" onClick={(event) => event.stopPropagation()}>
            <h3>确定</h3>
            <p>{pendingAction === 'start' ? '确认启动链上中国平台？' : '确认关闭链上中国平台？'}</p>
            <div className="communication-node-confirm-actions">
              <button type="button" disabled={saving} onClick={() => setPendingAction(null)}>
                取消
              </button>
              <button type="button" disabled={saving} onClick={confirmAction}>
                确认
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
