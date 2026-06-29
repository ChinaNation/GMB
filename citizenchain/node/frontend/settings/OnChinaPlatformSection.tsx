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
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const running = platform?.running ?? false;
  const url = platform?.url ?? fallbackUrl;

  const confirmStart = async () => {
    setSaving(true);
    setError(null);
    try {
      const next = await api.startOnChinaPlatform();
      onUpdated(next);
      setConfirmOpen(false);
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
        <span className="onchina-platform-url">{url}</span>
        <button
          type="button"
          className="onchina-platform-button"
          disabled={!platform || running || saving}
          onClick={() => setConfirmOpen(true)}
        >
          {saving ? '启动中' : running ? '已启动' : '启动'}
        </button>
      </div>
      {error ? <p className="section-inline-error">{error}</p> : null}
      {confirmOpen ? (
        <div className="communication-node-confirm-mask" onClick={() => !saving && setConfirmOpen(false)}>
          <div className="communication-node-confirm" onClick={(event) => event.stopPropagation()}>
            <h3>确定</h3>
            <p>确认启动链上中国平台？</p>
            <div className="communication-node-confirm-actions">
              <button type="button" disabled={saving} onClick={() => setConfirmOpen(false)}>
                取消
              </button>
              <button type="button" disabled={saving} onClick={confirmStart}>
                确认
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
