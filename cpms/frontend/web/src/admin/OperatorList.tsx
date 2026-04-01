import { useState, useEffect, useCallback } from 'react';
import * as api from '../api';
import type { AdminUser } from '../types';

export default function OperatorList() {
  const [operators, setOperators] = useState<AdminUser[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [newPubkey, setNewPubkey] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const load = useCallback(async () => {
    try {
      const res = await api.listOperators();
      if (res.data) setOperators(res.data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleCreate = async () => {
    if (!newPubkey.trim()) return;
    setError('');
    setLoading(true);
    try {
      await api.createOperator(newPubkey.trim());
      setNewPubkey('');
      setShowAdd(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建失败');
    }
    setLoading(false);
  };

  const handleToggleStatus = async (op: AdminUser) => {
    const next = op.status === 'ACTIVE' ? 'DISABLED' : 'ACTIVE';
    try {
      await api.updateOperatorStatus(op.user_id, next);
      await load();
    } catch { /* ignore */ }
  };

  const handleDelete = async (op: AdminUser) => {
    if (!confirm(`确认删除操作员 ${op.user_id}？`)) return;
    try {
      await api.deleteOperator(op.user_id);
      await load();
    } catch { /* ignore */ }
  };

  return (
    <div className="card">
      <div className="card__title flex-between">
        系统管理员列表
        <button className="btn btn--primary" onClick={() => setShowAdd(true)}>+ 新增管理员</button>
      </div>

      {showAdd && (
        <div style={{ background: '#f0fdfa', padding: 16, borderRadius: 'var(--radius)', marginBottom: 16 }}>
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{error}</div>}
          <div style={{ display: 'flex', gap: 8 }}>
            <input className="form-input" placeholder="管理员 Sr25519 公钥" value={newPubkey} onChange={e => setNewPubkey(e.target.value)} />
            <button className="btn btn--blue" onClick={handleCreate} disabled={loading}>确认</button>
            <button className="btn btn--ghost" onClick={() => { setShowAdd(false); setError(''); }}>取消</button>
          </div>
        </div>
      )}

      <table className="table">
        <thead>
          <tr><th>用户ID</th><th>公钥</th><th>角色</th><th>状态</th><th>操作</th></tr>
        </thead>
        <tbody>
          {operators.length === 0 ? (
            <tr><td colSpan={5} className="text-center" style={{ color: 'var(--color-text-secondary)' }}>暂无管理员</td></tr>
          ) : operators.map(op => (
            <tr key={op.user_id}>
              <td><span className="text-ellipsis">{op.user_id}</span></td>
              <td><span className="text-ellipsis" style={{ maxWidth: 160 }}>{op.admin_pubkey}</span></td>
              <td>{op.role}</td>
              <td>
                <span className={`tag ${op.status === 'ACTIVE' ? 'tag--success' : 'tag--warning'}`}>
                  {op.status === 'ACTIVE' ? '启用' : '禁用'}
                </span>
              </td>
              <td style={{ display: 'flex', gap: 4 }}>
                <button className="btn btn--ghost btn--sm" onClick={() => handleToggleStatus(op)}>
                  {op.status === 'ACTIVE' ? '禁用' : '启用'}
                </button>
                <button className="btn btn--danger btn--sm" onClick={() => handleDelete(op)}>删除</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
