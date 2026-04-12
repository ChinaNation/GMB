// 系统管理员列表。
// 新增管理员：标题右侧内联展开姓名+账户输入框，账户输入框右侧扫码图标。

import { useState, useEffect, useCallback, useRef } from 'react';
import * as api from '../api';
import type { AdminUser } from '../types';
import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import type { UserContactBody } from '../qr/wuminQr';
import { startCameraScanner } from '../utils/cameraScanner';
import { ScanIcon } from '../components/ScanIcon';

export default function OperatorList() {
  const [operators, setOperators] = useState<AdminUser[]>([]);
  const [addOpen, setAddOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newPubkey, setNewPubkey] = useState('');
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
      const res = await api.listOperators();
      if (res.data) setOperators(res.data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleCreate = async () => {
    if (!newPubkey.trim()) { setError('请输入管理员账户'); return; }
    setError('');
    setLoading(true);
    try {
      await api.createOperator(newPubkey.trim(), newName.trim());
      setNewPubkey('');
      setNewName('');
      setAddOpen(false);
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
    if (!confirm(`确认删除管理员 ${op.admin_name || op.user_id}？`)) return;
    try {
      await api.deleteOperator(op.user_id);
      await load();
    } catch { /* ignore */ }
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
        <span>系统管理员列表</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {addOpen && (
          <>
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
                setAddOpen(false);
                setError('');
                setNewName('');
                setNewPubkey('');
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

      <table className="table">
        <thead>
          <tr><th>姓名</th><th>用户ID</th><th>公钥</th><th>角色</th><th>状态</th><th>操作</th></tr>
        </thead>
        <tbody>
          {operators.length === 0 ? (
            <tr><td colSpan={6} className="text-center" style={{ color: 'var(--color-text-secondary)' }}>暂无管理员</td></tr>
          ) : operators.map(op => (
            <tr key={op.user_id}>
              <td>{op.admin_name || '—'}</td>
              <td><span className="text-ellipsis">{op.user_id}</span></td>
              <td><span className="text-ellipsis" style={{ maxWidth: 160 }}>{op.admin_pubkey}</span></td>
              <td>{op.role === 'OPERATOR_ADMIN' ? '系统管理员' : op.role}</td>
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
