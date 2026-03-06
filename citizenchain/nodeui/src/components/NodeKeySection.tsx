import { useEffect, useState } from 'react';
import { api } from '../api';
import type { BootnodeKey } from '../types';

type Props = {
  nodeKey: BootnodeKey;
  onUpdated: (next: BootnodeKey) => void;
  onApplied: () => void;
  disabled: boolean;
};

export function NodeKeySection({ nodeKey, onUpdated, onApplied, disabled }: Props) {
  type PendingAction = 'bootnode' | 'grandpa' | null;

  const [input, setInput] = useState(nodeKey.nodeKey ?? '');
  const [bootnodeInstitutionName, setBootnodeInstitutionName] = useState<string | null>(
    nodeKey.institutionName ?? null,
  );
  const [grandpaInput, setGrandpaInput] = useState('');
  const [grandpaInstitutionName, setGrandpaInstitutionName] = useState<string | null>(null);
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState('');
  const [pendingAction, setPendingAction] = useState<PendingAction>(null);
  const [saving, setSaving] = useState(false);
  const [savingGrandpa, setSavingGrandpa] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setInput(nodeKey.nodeKey ?? '');
    setBootnodeInstitutionName(nodeKey.institutionName ?? null);
  }, [nodeKey.nodeKey, nodeKey.institutionName]);

  useEffect(() => {
    let cancelled = false;
    void api
      .getGrandpaKey()
      .then((k) => {
        if (cancelled) return;
        setGrandpaInput(k.key ?? '');
        setGrandpaInstitutionName(k.institutionName ?? null);
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="section settings-nodekey-section">
      <div className="bootnode-inline">
        <h2>
          区块链引导节点
          <span className="grandpa-bind-state">
            {bootnodeInstitutionName ?? '未绑定'}
          </span>
        </h2>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="请输入区块链引导节点私钥"
          type="password"
          disabled={disabled || saving}
        />
        <button
          disabled={disabled || saving}
          onClick={() => {
            if (!input.trim()) {
              setError('请输入区块链引导节点私钥');
              return;
            }
            setError(null);
            setPendingAction('bootnode');
            setUnlockPassword('');
            setShowPasswordModal(true);
          }}
        >
          {saving ? '上传中...' : '上传私钥'}
        </button>
      </div>
      <div className="bootnode-inline grandpa-inline">
        <h2>
          确定性投票节点
          <span className="grandpa-bind-state">
            {grandpaInstitutionName ?? '未绑定'}
          </span>
        </h2>
        <input
          value={grandpaInput}
          onChange={(e) => setGrandpaInput(e.target.value)}
          placeholder="请输入确定性投票节点私钥"
          type="password"
          disabled={disabled || savingGrandpa}
        />
        <button
          disabled={disabled || savingGrandpa}
          onClick={() => {
            if (!grandpaInput.trim()) {
              setError('请输入确定性投票节点私钥');
              return;
            }
            setError(null);
            setPendingAction('grandpa');
            setUnlockPassword('');
            setShowPasswordModal(true);
          }}
        >
          {savingGrandpa ? '上传中...' : '上传私钥'}
        </button>
      </div>
      {error ? <p className="section-inline-error">{error}</p> : null}

      {showPasswordModal ? (
        <div
          className="unlock-modal-mask"
          onClick={() => !(saving || savingGrandpa) && setShowPasswordModal(false)}
        >
          <div className="unlock-modal" onClick={(e) => e.stopPropagation()}>
            <h3>设备密码验证</h3>
            <input
              className="unlock-password-input"
              type="password"
              value={unlockPassword}
              onChange={(e) => setUnlockPassword(e.target.value)}
              placeholder="请输入设备开机密码"
              disabled={saving || savingGrandpa}
            />
            <div className="unlock-modal-actions">
              <button
                onClick={() => setShowPasswordModal(false)}
                disabled={saving || savingGrandpa}
              >
                取消
              </button>
              <button
                onClick={async () => {
                  const password = unlockPassword.trim();
                  if (!password) {
                    setError('请输入设备开机密码');
                    return;
                  }

                  if (pendingAction === 'bootnode') {
                    setSaving(true);
                    try {
                      const next = await api.setBootnodeKey(input.trim(), password);
                      if (next.institutionName) {
                        await api.setNodeName(next.institutionName);
                      }
                      onUpdated(next);
                      onApplied();
                      setBootnodeInstitutionName(next.institutionName ?? null);
                      setInput('');
                      setShowPasswordModal(false);
                      setPendingAction(null);
                      setUnlockPassword('');
                      setError(null);
                    } catch (e) {
                      setError(e instanceof Error ? e.message : String(e));
                    } finally {
                      setSaving(false);
                    }
                    return;
                  }

                  if (pendingAction === 'grandpa') {
                    setSavingGrandpa(true);
                    try {
                      const next = await api.setGrandpaKey(grandpaInput.trim(), password);
                      setGrandpaInput('');
                      setGrandpaInstitutionName(next.institutionName ?? null);
                      setShowPasswordModal(false);
                      setPendingAction(null);
                      setUnlockPassword('');
                      setError(null);
                    } catch (e) {
                      setError(e instanceof Error ? e.message : String(e));
                    } finally {
                      setSavingGrandpa(false);
                    }
                    return;
                  }

                  setError('未选择上传类型，请重新点击上传按钮');
                }}
                disabled={saving || savingGrandpa || disabled}
              >
                {saving || savingGrandpa ? '验证中...' : '确认'}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
