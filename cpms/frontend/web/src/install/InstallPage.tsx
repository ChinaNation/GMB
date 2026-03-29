import { useState, useEffect } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { InstallStatus } from '../types';

export default function InstallPage() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [qrInput, setQrInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [msg, setMsg] = useState('');

  const load = async () => {
    try {
      const res = await api.installStatus();
      if (res.data) setStatus(res.data);
    } catch { /* ignore */ }
  };

  useEffect(() => { load(); }, []);

  const handleInit = async () => {
    if (!qrInput.trim()) { setError('请输入 SFID 初始化二维码内容'); return; }
    setError(''); setMsg('');
    setLoading(true);
    try {
      const res = await api.installInitialize(qrInput.trim());
      if (res.data) setMsg(`初始化成功，站点 SFID: ${res.data.site_sfid}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '初始化失败');
    }
    setLoading(false);
  };

  return (
    <div className="login-page">
      <div style={{ background: '#fff', borderRadius: 8, boxShadow: '0 8px 40px rgba(0,0,0,0.25)', padding: 40, width: 560 }}>
        <div className="login-card__title">CPMS 系统初始化</div>
        <div className="login-card__subtitle">公民护照管理系统</div>

        {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}
        {msg && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{msg}</div>}

        {status?.initialized ? (
          <div style={{ textAlign: 'center', color: 'var(--color-success)', marginBottom: 24 }}>
            系统已初始化 | 站点 SFID: <strong>{status.site_sfid}</strong>
          </div>
        ) : (
          <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
            <div className="card__title">步骤 1: SFID 授权初始化</div>
            <div className="form-group">
              <label>SFID 初始化二维码内容</label>
              <textarea className="form-input" rows={3} value={qrInput} onChange={e => setQrInput(e.target.value)} />
            </div>
            <button className="btn btn--primary" onClick={handleInit} disabled={loading}>
              {loading ? '初始化中...' : '执行初始化'}
            </button>
          </div>
        )}

        {status?.initialized && status.super_admin_bind_qrs.length > 0 && (
          <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)', marginTop: 16 }}>
            <div className="card__title">步骤 2: 绑定超级管理员 ({status.super_admin_bound_count}/3)</div>
            <div style={{ display: 'flex', gap: 16, justifyContent: 'center', flexWrap: 'wrap' }}>
              {status.super_admin_bind_qrs.map(qr => (
                <div key={qr.key_id} style={{ textAlign: 'center' }}>
                  {qr.bound ? (
                    <div style={{ width: 140, height: 140, background: '#C6F6D5', borderRadius: 'var(--radius)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-success)', fontWeight: 600 }}>
                      已绑定
                    </div>
                  ) : (
                    <QRCodeSVG value={qr.qr_content} size={140} />
                  )}
                  <div style={{ marginTop: 6, fontSize: 12, color: 'var(--color-text-secondary)' }}>{qr.key_id}</div>
                </div>
              ))}
            </div>
          </div>
        )}

        {status?.initialized && status.super_admin_bound_count >= 3 && (
          <div className="mt-16 text-center">
            <a href="/login" className="btn btn--primary">前往登录</a>
          </div>
        )}
      </div>
    </div>
  );
}
