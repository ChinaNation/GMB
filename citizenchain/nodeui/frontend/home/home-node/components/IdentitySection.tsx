import { useState } from 'react';
import { api } from '../../../api';
import type { NodeIdentity } from '../../../types';

type Props = {
  identity: NodeIdentity;
  onUpdated: (next: NodeIdentity) => void;
  disabled: boolean;
};

export function IdentitySection({ identity, onUpdated, disabled }: Props) {
  const [editing, setEditing] = useState(false);
  const [input, setInput] = useState(identity.nodeName ?? '');
  const [saving, setSaving] = useState(false);

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
              onClick={async () => {
                setSaving(true);
                try {
                  const next = await api.setNodeName(input);
                  onUpdated(next);
                  setEditing(false);
                } finally {
                  setSaving(false);
                }
              }}
            >
              {saving ? '保存中...' : '保存'}
            </button>
            <button
              disabled={disabled || saving}
              onClick={() => {
                setInput(identity.nodeName ?? '');
                setEditing(false);
              }}
            >
              取消
            </button>
          </div>
        </>
      ) : null}
      <p>P2P地址: {identity.peerId ? `/p2p/${identity.peerId}` : '-'}</p>
      <p>节点角色: {identity.role ?? '-'}</p>
    </section>
  );
}
