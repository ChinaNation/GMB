// 管理员列表。
// 新增管理员：标题右侧内联展开姓名+账户输入框，账户输入框右侧扫码图标。

import { useState, useEffect, useCallback, useRef } from 'react';
import * as api from './api';
import type { AdminRole, AdminUser } from './types';
import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import type { UserContactBody } from '../qr/wuminQr';
import { startCameraScanner } from '../qr/cameraScanner';
import { ScanIcon } from '../components/ScanIcon';

export default function AdminList() {
  const [admins, setAdmins] = useState<AdminUser[]>([]);
  const [addOpen, setAddOpen] = useState(false);
  const [newRole, setNewRole] = useState<AdminRole | ''>('');
  const [newName, setNewName] = useState('');
  const [newPubkey, setNewPubkey] = useState('');
  const [editingUserId, setEditingUserId] = useState('');
  const [editingName, setEditingName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  // 扫码弹窗
  const [scanOpen, setScanOpen] = useState(false);
  const [scanReady, setScanReady] = useState(false);
  const [scanError, setScanError] = useState('');
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanCleanupRef = useRef<(() => void) | null>(null);

  const load = useCallback(async () => {
    try {
      const res = await api.listAdmins();
      if (res.data) setAdmins(res.data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { load(); }, [load]);

  const superAdminCount = admins.filter(admin => admin.role === 'SUPER_ADMIN').length;

  const handleCreate = async () => {
    if (!newRole) { setError('请选择管理员类型'); return; }
    if (!newName.trim()) { setError('请输入管理员姓名'); return; }
    if (!newPubkey.trim()) { setError('请输入管理员账户'); return; }
    setError('');
    setLoading(true);
    try {
      await api.createAdmin({
        role: newRole,
        admin_name: newName.trim(),
        admin_pubkey: newPubkey.trim(),
      });
      setNewPubkey('');
      setNewName('');
      setNewRole('');
      setAddOpen(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建失败');
    }
    setLoading(false);
  };

  const handleEdit = (admin: AdminUser) => {
    setEditingUserId(admin.user_id);
    setEditingName(admin.admin_name);
    setError('');
  };

  const handleSaveName = async (admin: AdminUser) => {
    if (!editingName.trim()) { setError('请输入管理员姓名'); return; }
    setLoading(true);
    setError('');
    try {
      await api.updateAdminName(admin.user_id, editingName.trim());
      setEditingUserId('');
      setEditingName('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '保存失败');
    }
    setLoading(false);
  };

  const handleDelete = async (admin: AdminUser) => {
    if (!admin.can_delete) return;
    if (!confirm(`确认删除管理员 ${admin.admin_name || admin.user_id}？`)) return;
    try {
      await api.deleteAdmin(admin.user_id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '删除失败');
    }
  };

  const resetAddForm = () => {
    setAddOpen(false);
    setError('');
    setNewRole('');
    setNewName('');
    setNewPubkey('');
  };

  // ── 扫码 ──
  const stopScanner = () => {
    if (scanCleanupRef.current) {
      scanCleanupRef.current();
      scanCleanupRef.current = null;
    }
    setScanReady(false);
  };

  useEffect(() => {
    if (!scanOpen || !videoRef.current) {
      stopScanner();
      return;
    }
    const video = videoRef.current;
    const cleanup = startCameraScanner(
      video,
      (raw) => {
        try {
          const env = parseQrEnvelope(raw);
          if (env.kind !== 'user_contact') {
            setScanError(`需要扫描公民名片（user_contact），当前为 ${env.kind}`);
            return;
          }
          const { address } = env.body as UserContactBody;
          setNewPubkey(address.trim());
          setScanOpen(false);
          stopScanner();
        } catch (e) {
          setScanError(e instanceof QrParseError ? e.message : '二维码格式无效');
        }
      },
      () => { setScanReady(true); },
      (msg) => { setScanError(msg); },
    );
    scanCleanupRef.current = cleanup;
    return () => stopScanner();
  }, [scanOpen]);

  return (
    <div className="card">
      <div className="card__title flex-between">
        <span>管理员列表</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {addOpen && (
          <>
            <select
              className="form-input"
              style={{ width: 150, flexShrink: 0 }}
              value={newRole}
              onChange={e => setNewRole(e.target.value as AdminRole | '')}
            >
              <option value="">请选择类型</option>
              <option value="SUPER_ADMIN" disabled={superAdminCount >= 5}>超级管理员</option>
              <option value="OPERATOR_ADMIN">操作管理员</option>
            </select>
            <input
              className="form-input"
              style={{ width: 140, flexShrink: 0 }}
              placeholder="管理员姓名"
              value={newName}
              onChange={e => setNewName(e.target.value)}
            />
            <div style={{ position: 'relative' }}>
              <input
                className="form-input"
                style={{ width: 460, paddingRight: 36 }}
                placeholder="管理员账户（SS58 地址）"
                value={newPubkey}
                onChange={e => setNewPubkey(e.target.value)}
              />
              <button
                onClick={() => { setScanError(''); setScanOpen(true); }}
                style={{
                  position: 'absolute', right: 4, top: '50%', transform: 'translateY(-50%)',
                  background: 'none', border: 'none', cursor: 'pointer', padding: 4,
                  color: 'var(--color-primary)', lineHeight: 1,
                }}
                title="扫码识别账户"
              >
                <ScanIcon size={18} />
              </button>
            </div>
            <button className="btn btn--primary" onClick={handleCreate} disabled={loading}>
              {loading ? '...' : '确认'}
            </button>
          </>
          )}
          <button
            className={addOpen ? 'btn btn--ghost' : 'btn btn--primary'}
            onClick={() => {
              if (addOpen) {
                resetAddForm();
              } else {
                setAddOpen(true);
              }
            }}
          >
            {addOpen ? '取消新增' : '+ 新增管理员'}
          </button>
        </div>
      </div>

      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{error}</div>}

      <table className="table admin-table">
        <thead>
          <tr><th>姓名</th><th>用户ID</th><th>账户</th><th>角色</th><th>操作</th></tr>
        </thead>
        <tbody>
          {admins.length === 0 ? (
            <tr><td colSpan={5} className="text-center" style={{ color: 'var(--color-text-secondary)' }}>暂无管理员</td></tr>
          ) : admins.map(admin => (
            <tr key={admin.user_id}>
              <td>
                {editingUserId === admin.user_id ? (
                  <input
                    className="form-input"
                    style={{ width: 160 }}
                    value={editingName}
                    onChange={e => setEditingName(e.target.value)}
                  />
                ) : admin.admin_name || '—'}
              </td>
              <td style={{ fontFamily: 'monospace', whiteSpace: 'nowrap' }}>{admin.user_id}</td>
              <td style={{ fontFamily: 'monospace', whiteSpace: 'nowrap' }}>{admin.admin_address || admin.admin_pubkey}</td>
              <td>{admin.role === 'SUPER_ADMIN' ? '超级管理员' : '操作管理员'}</td>
              <td>
                <div className="admin-table__actions">
                  {editingUserId === admin.user_id ? (
                    <>
                      <button className="btn btn--primary btn--sm" onClick={() => void handleSaveName(admin)} disabled={loading}>保存</button>
                      <button className="btn btn--ghost btn--sm" onClick={() => { setEditingUserId(''); setEditingName(''); }}>取消</button>
                    </>
                  ) : (
                    <>
                      <button className="btn btn--ghost btn--sm" onClick={() => handleEdit(admin)} disabled={!admin.can_edit_name}>编辑</button>
                      <button className="btn btn--danger btn--sm" onClick={() => handleDelete(admin)} disabled={!admin.can_delete}>删除</button>
                    </>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* 扫码弹窗 */}
      {scanOpen && (
        <div
          style={{
            position: 'fixed', inset: 0, zIndex: 1000,
            background: 'rgba(0,0,0,0.5)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}
          onClick={() => setScanOpen(false)}
        >
          <div
            style={{
              background: '#fff', borderRadius: 16, padding: 24, width: 360,
              display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12,
            }}
            onClick={e => e.stopPropagation()}
          >
            <div style={{ fontSize: 16, fontWeight: 600 }}>扫描公民名片二维码</div>
            <div style={{
              width: 280, height: 280,
              background: 'linear-gradient(145deg, #0f172a, #1e293b)',
              borderRadius: 12, overflow: 'hidden',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              position: 'relative',
            }}>
              <video ref={videoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
              {!scanReady && (
                <div style={{
                  position: 'absolute', inset: 0,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  color: 'rgba(255,255,255,0.5)', fontSize: 13,
                }}>
                  摄像头初始化中...
                </div>
              )}
            </div>
            {scanError && <div style={{ color: 'var(--color-danger)', fontSize: 12 }}>{scanError}</div>}
            <button className="btn btn--ghost btn--sm" onClick={() => setScanOpen(false)}>关闭</button>
          </div>
        </div>
      )}
    </div>
  );
}
