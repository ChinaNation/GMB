import { useState, useEffect } from 'react';
import * as api from './api';
import type { InstallStatus } from './types';
import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import type { UserContactBody } from '../qr/wuminQr';
import CameraQrScanner from '../qr/CameraQrScanner';

// CPMS 初始化页面。
// 三个事实状态步骤：1.扫描 INSTALL 安装码  2.绑定管理员  3.完成（可直接签发 ARCHIVE）
// CPMS 安装码由 SFID 签发；安装后不再生成中间注册码。

export default function InstallPage() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [msg, setMsg] = useState('');
  const [scannerActive, setScannerActive] = useState(false);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindLoading, setBindLoading] = useState(false);

  const load = async () => {
    try {
      const res = await api.installStatus();
      if (res.data) {
        setStatus(res.data);
      }
    } catch { /* ignore */ }
  };

  useEffect(() => { load(); }, []);

  const handleBindScanned = async (raw: string) => {
    setError('');
    setMsg('');
    setBindLoading(true);
    setBindScannerActive(false);
    try {
      // WUMIN_QR_V1 统一协议：解析 user_contact envelope，取 address（SS58）
      const env = parseQrEnvelope(raw);
      if (env.kind !== 'user_contact') {
        throw new Error(`需要扫描公民名片二维码（user_contact），当前为 ${env.kind}`);
      }
      const { address } = env.body as UserContactBody;
      // SS58 address 传后端，后端做 SS58→hex 解码
      await api.bindSuperAdmin(address.trim());
      setMsg('超级管理员绑定成功');
      await load();
    } catch (e) {
      if (e instanceof QrParseError) {
        setError(`二维码格式错误：${e.message}`);
      } else {
        setError(e instanceof Error ? e.message : '绑定失败');
      }
    }
    setBindLoading(false);
  };

  const handleQr1Scanned = async (qrContent: string) => {
    setError('');
    setMsg('');
    setLoading(true);
    setScannerActive(false);
    try {
      const res = await api.installInitialize(qrContent);
      if (res.data) {
        setMsg(`机构 SFID: ${res.data.sfid_number}`);
      }
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : '初始化失败');
    }
    setLoading(false);
  };

  // 三个事实状态：1.未初始化  2.已初始化未绑定管理员  3.已初始化已绑定管理员
  const initialized = status?.initialized ?? false;
  const adminBound = (status?.super_admin_bound_count ?? 0) >= 1;

  let currentStep = 1;
  if (initialized && !adminBound) currentStep = 2;
  if (initialized && adminBound) currentStep = 3;

  return (
    <div className="login-page">
      <div className="login-card" style={{ width: 580 }}>
        <div className="login-card__header">
          <div className="login-card__title">公民护照管理系统</div>
          <div className="login-card__subtitle">系统初始化</div>
        </div>
        <div className="login-card__body">
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}
          {msg && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{msg}</div>}

          <div style={{ display: 'flex', gap: 8, marginBottom: 20, justifyContent: 'center' }}>
            {['扫描安装码', '绑定管理员', '完成'].map((label, i) => (
              <div key={i} style={{
                padding: '4px 12px',
                borderRadius: 6,
                fontSize: 12,
                fontWeight: 500,
                background: currentStep > i + 1 ? '#dcfce7' : currentStep === i + 1 ? 'var(--color-primary)' : '#f3f4f6',
                color: currentStep > i + 1 ? 'var(--color-success)' : currentStep === i + 1 ? '#fff' : '#9ca3af',
              }}>
                {label}
              </div>
            ))}
          </div>

          {currentStep === 1 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title" style={{ textAlign: 'center', borderLeft: 'none', paddingLeft: 0 }}>扫描 SFID 安装授权二维码</div>
              <div style={{ margin: '12px auto' }}>
                <CameraQrScanner
                  active={scannerActive}
                  onActiveChange={setScannerActive}
                  onDetected={handleQr1Scanned}
                  onError={setError}
                  buttonLabel={loading ? '处理中...' : '开启扫码'}
                  idleText="点击开启摄像头扫码"
                  busy={loading}
                  size={280}
                />
              </div>
            </div>
          )}

          {currentStep === 2 && (
            <div className="card" style={{ boxShadow: 'none', border: '1px solid var(--color-border)' }}>
              <div className="card__title" style={{ textAlign: 'center', borderLeft: 'none', paddingLeft: 0 }}>绑定超级管理员</div>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 12 }}>
                打开公民钱包，展示钱包二维码，用摄像头扫码读取账户地址
              </div>
              <div style={{ margin: '12px auto' }}>
                <CameraQrScanner
                  active={bindScannerActive}
                  onActiveChange={setBindScannerActive}
                  onDetected={handleBindScanned}
                  onError={setError}
                  buttonLabel={bindLoading ? '绑定中...' : '开启扫码'}
                  idleText="点击开启摄像头扫码"
                  busy={bindLoading}
                  size={280}
                />
              </div>
            </div>
          )}

          {currentStep === 3 && (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <div style={{ fontSize: 36, marginBottom: 12 }}>✅</div>
              <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-success)', marginBottom: 8 }}>
                初始化完成
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 4 }}>
                机构 SFID: <strong>{status?.sfid_number}</strong>
              </div>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 20 }}>
                请登录系统创建档案并签发 ARCHIVE 档案二维码
              </div>
              <a href="/login" className="btn btn--primary">前往登录</a>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
