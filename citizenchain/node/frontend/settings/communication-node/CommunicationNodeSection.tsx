import { useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../../core/tauri';
import { settingsApi as api } from '../api';
import type { CommunicationNodeState } from '../types';

type Props = {
  communicationNode: CommunicationNodeState | null;
  onUpdated: (next: CommunicationNodeState) => void;
};

function shortPeerId(peerId: string | null) {
  if (!peerId) return '节点启动后显示';
  if (peerId.length <= 18) return peerId;
  return `${peerId.slice(0, 10)}...${peerId.slice(-6)}`;
}

export function CommunicationNodeSection({ communicationNode, onUpdated }: Props) {
  const [saving, setSaving] = useState(false);
  const [qrOpen, setQrOpen] = useState(false);
  const [pendingEnabled, setPendingEnabled] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  const enabled = communicationNode?.enabled ?? false;
  const canShowQr = Boolean(communicationNode?.pairingPayload);

  const toggleEnabled = async () => {
    setPendingEnabled(!enabled);
  };

  const confirmToggleEnabled = async () => {
    if (pendingEnabled === null) return;
    setSaving(true);
    setError(null);
    try {
      const next = await api.setCommunicationNodeEnabled(pendingEnabled);
      onUpdated(next);
      setPendingEnabled(null);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="section settings-communication-node-section">
      <div className="communication-node-panel">
        <div className="communication-node-header">
          <div>
            <div className="communication-node-title-row">
              <h2>通信节点功能</h2>
              <span className={`communication-node-status ${enabled ? 'enabled' : 'disabled'}`}>
                {enabled ? '已开启' : '未开启'}
              </span>
            </div>
            <p>归档全节点和普通全节点都可以开启；本节点只服务自己的公民手机。</p>
          </div>
          <button type="button" disabled={!communicationNode || saving} onClick={toggleEnabled}>
            {saving ? '处理中...' : enabled ? '关闭' : '开启'}
          </button>
        </div>

        {communicationNode ? (
          <>
            <div className="communication-node-info">
              {canShowQr ? (
                <button
                  type="button"
                  className="communication-node-qr"
                  onClick={() => setQrOpen(true)}
                  aria-label="放大通信节点二维码"
                >
                  <QRCodeSVG value={communicationNode.pairingPayload!} size={44} level="M" />
                </button>
              ) : (
                <div>
                  <span>通信节点</span>
                  <strong>{shortPeerId(communicationNode.peerId)}</strong>
                </div>
              )}
              <div>
                <span>通信端点</span>
                <strong>{communicationNode.nodeMultiaddr ?? '节点启动后显示'}</strong>
              </div>
            </div>
            {!canShowQr ? (
              <p className="section-inline-hint">
                节点启动并取得 PeerId 后显示通信节点二维码。
              </p>
            ) : null}
          </>
        ) : (
          <p className="section-inline-hint">正在读取通信节点功能...</p>
        )}
        {error ? <p className="section-inline-error">{error}</p> : null}
      </div>
      {qrOpen && communicationNode?.pairingPayload ? (
        <div className="communication-node-qr-modal-mask" onClick={() => setQrOpen(false)}>
          <div className="communication-node-qr-modal" onClick={(event) => event.stopPropagation()}>
            <QRCodeSVG value={communicationNode.pairingPayload} size={300} level="M" />
            <button type="button" onClick={() => setQrOpen(false)}>关闭</button>
          </div>
        </div>
      ) : null}
      {pendingEnabled !== null ? (
        <div className="communication-node-confirm-mask" onClick={() => !saving && setPendingEnabled(null)}>
          <div className="communication-node-confirm" onClick={(event) => event.stopPropagation()}>
            <h3>确定</h3>
            <p>{pendingEnabled ? '确认开启通信节点功能？' : '确认关闭通信节点功能？'}</p>
            <div className="communication-node-confirm-actions">
              <button type="button" disabled={saving} onClick={() => setPendingEnabled(null)}>
                取消
              </button>
              <button type="button" disabled={saving} onClick={confirmToggleEnabled}>
                确认
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
