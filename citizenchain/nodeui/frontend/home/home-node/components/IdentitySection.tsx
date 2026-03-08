import { useEffect, useState } from 'react';
import { api } from '../../../api';
import type { NodeIdentity } from '../../../types';

type Props = {
  identity: NodeIdentity;
  onUpdated: (next: NodeIdentity) => void;
  disabled: boolean;
};

export function IdentitySection({ identity, onUpdated, disabled }: Props) {
  const [editing, setEditing] = useState(false);
  const [input, setInput] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showUnlockModal, setShowUnlockModal] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState('');

  useEffect(() => {
    if (!editing) {
      setInput(identity.nodeName ?? '');
    }
  }, [editing, identity.nodeName]);

  const closeUnlockModal = () => {
    if (saving) return;
    setShowUnlockModal(false);
    setUnlockPassword('');
  };

  const saveNodeName = async () => {
    const password = unlockPassword.trim();
    if (!password) {
      setError('请输入设备开机密码');
      return;
    }
    setSaving(true);
    try {
      const next = await api.setNodeName(input, password);
      onUpdated(next);
      setEditing(false);
      setShowUnlockModal(false);
      setUnlockPassword('');
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="section">
      <h2>身份</h2>
      <p>
        节点名称: {identity.nodeName ?? '-'}{' '}
        {!editing ? (
          <button
            disabled={disabled || saving}
            onClick={() => {
              setInput(identity.nodeName ?? '');
              setEditing(true);
              setError(null);
            }}
          >
            编辑
          </button>
        ) : null}
      </p>
      {editing ? (
        <>
          <input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="输入节点名称"
            disabled={disabled || saving}
          />
          <div className="actions">
            <button
              disabled={disabled || saving}
              onClick={() => {
                if (!input.trim()) {
                  setError('请输入节点名称');
                  return;
                }
                setError(null);
                setUnlockPassword('');
                setShowUnlockModal(true);
              }}
            >
              {saving ? '保存中...' : '保存'}
            </button>
            <button
              disabled={disabled || saving}
              onClick={() => {
                setInput(identity.nodeName ?? '');
                setEditing(false);
                setError(null);
              }}
            >
              取消
            </button>
          </div>
        </>
      ) : null}
      {showUnlockModal ? (
        <div className="unlock-modal-mask" onClick={closeUnlockModal}>
          <div className="unlock-modal" onClick={(e) => e.stopPropagation()}>
            <h3>修改节点名称</h3>
            <input
              className="unlock-password-input"
              type="password"
              value={unlockPassword}
              onChange={(e) => setUnlockPassword(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  void saveNodeName();
                }
              }}
              placeholder="请输入设备开机密码"
              disabled={saving || disabled}
            />
            <div className="unlock-modal-actions">
              <button onClick={closeUnlockModal} disabled={saving || disabled}>
                取消
              </button>
              <button onClick={() => void saveNodeName()} disabled={saving || disabled}>
                {saving ? '保存中...' : '确认保存'}
              </button>
            </div>
          </div>
        </div>
      ) : null}
      {error ? <p className="section-inline-error">{error}</p> : null}
      <p>P2P地址: {identity.peerId ? `/p2p/${identity.peerId}` : '-'}</p>
      <p>节点角色: {identity.role ?? '-'}</p>
    </section>
  );
}
