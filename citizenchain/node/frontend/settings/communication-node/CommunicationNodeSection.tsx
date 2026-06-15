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

function formatExpiry(expiresAtMillis: number | null) {
  if (!expiresAtMillis) return null;
  return new Date(expiresAtMillis).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function CommunicationNodeSection({ communicationNode, onUpdated }: Props) {
  const [saving, setSaving] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const enabled = communicationNode?.enabled ?? false;
  const canShowQr = Boolean(communicationNode?.enabled && communicationNode.pairingPayload);
  const expiresAt = formatExpiry(communicationNode?.expiresAtMillis ?? null);

  const toggleEnabled = async () => {
    setSaving(true);
    setError(null);
    try {
      const next = await api.setCommunicationNodeEnabled(!enabled);
      onUpdated(next);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setSaving(false);
    }
  };

  const refreshPairingQr = async () => {
    setRefreshing(true);
    setError(null);
    try {
      const next = await api.getCommunicationNode();
      onUpdated(next);
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setRefreshing(false);
    }
  };

  return (
    <section className="section settings-communication-node-section">
      <div className="communication-node-panel">
        <div className="communication-node-header">
          <div>
            <h2>通信节点功能</h2>
            <p>归档全节点和普通全节点都可以开启；本节点只服务自己的公民手机。</p>
          </div>
          <button type="button" disabled={!communicationNode || saving} onClick={toggleEnabled}>
            {saving ? '处理中...' : enabled ? '关闭' : '开启'}
          </button>
        </div>

        {communicationNode ? (
          <>
            <div className="communication-node-info">
              <div>
                <span>状态</span>
                <strong>{enabled ? '已开启' : '未开启'}</strong>
              </div>
              <div>
                <span>PeerId</span>
                <strong title={communicationNode.peerId ?? undefined}>
                  {shortPeerId(communicationNode.peerId)}
                </strong>
              </div>
              <div>
                <span>RPC</span>
                <strong>{communicationNode.rpcUrl}</strong>
              </div>
              <div>
                <span>端点</span>
                <strong>{communicationNode.nodeMultiaddr ?? '开启后生成'}</strong>
              </div>
            </div>

            {canShowQr ? (
              <div className="communication-node-qr-row">
                <div className="communication-node-qr">
                  <QRCodeSVG value={communicationNode.pairingPayload!} size={220} level="M" />
                </div>
                <div className="communication-node-qr-copy">
                  <h3>公民扫码配对</h3>
                  <p>
                    打开公民 App：我的 → 设置 → 设置通信节点，扫描此二维码即可把手机连接到这台电脑通信节点。
                  </p>
                  <p>{expiresAt ? `二维码有效至 ${expiresAt}` : '二维码有效期 10 分钟'}</p>
                  <button type="button" disabled={refreshing} onClick={refreshPairingQr}>
                    {refreshing ? '刷新中...' : '刷新二维码'}
                  </button>
                </div>
              </div>
            ) : (
              <p className="section-inline-hint">
                {enabled ? '节点启动并取得 PeerId 后显示配对二维码。' : '开启后生成公民扫码配对二维码。'}
              </p>
            )}
          </>
        ) : (
          <p className="section-inline-hint">正在读取通信节点功能...</p>
        )}
        {error ? <p className="section-inline-error">{error}</p> : null}
      </div>
    </section>
  );
}
