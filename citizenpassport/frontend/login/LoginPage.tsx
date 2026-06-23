// 登录页:QR_V1 双向扫码登录
// 左侧展示签名请求二维码 → 手机扫码签名
// 右侧摄像头扫码 → 扫描手机签名响应 → 完成登录

import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import { useAuth } from '../authz/AuthProvider';
import * as api from './api';
import { parseQrEnvelope, QrParseError } from '../qr/citizenQr';
import CameraQrScanner from '../qr/CameraQrScanner';
import type { SessionUser } from '../common/types';

export default function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();

  const [qrSignRequest, setQrSignRequest] = useState<{
    challenge_id: string;
    login_qr_payload: string;
    session_id: string;
    expire_at: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [scannerActive, setScannerActive] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const pollingRef = useRef<number | null>(null);
  const loggedInRef = useRef(false);

  const stopPolling = () => {
    if (pollingRef.current !== null) {
      window.clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  };

  const doLogin = (user: SessionUser) => {
    if (loggedInRef.current) return;
    loggedInRef.current = true;
    stopPolling();
    setScannerActive(false);
    login(user);
    navigate('/admin');
  };

  useEffect(() => {
    return () => { stopPolling(); };
  }, []);

  const handleGenerateQr = async () => {
    setError('');
    setLoading(true);
    stopPolling();
    loggedInRef.current = false;
    try {
      const res = await api.authQrSignRequest();
      if (res.data) {
        setQrSignRequest(res.data);
        const { challenge_id, session_id } = res.data;
        pollingRef.current = window.setInterval(async () => {
          try {
            const r = await api.authQrResult(challenge_id, session_id);
            if (r.data?.status === 'SUCCESS' && r.data.user) {
              doLogin(r.data.user);
            } else if (r.data?.status === 'EXPIRED') {
              stopPolling();
              setError('二维码已过期，请重新生成');
              setQrSignRequest(null);
            }
          } catch { /* keep polling */ }
        }, 1500);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '生成登录二维码失败');
    } finally {
      setLoading(false);
    }
  };

  const handleReceiptScanned = async (raw: string) => {
    if (!qrSignRequest) return;
    setScanSubmitting(true);
    try {
      let env;
      try {
        env = parseQrEnvelope(raw);
      } catch (e) {
        const msg = e instanceof QrParseError ? e.message : '签名响应二维码格式无效';
        setError(msg);
        setScanSubmitting(false);
        return;
      }
      if (env.kind !== 'sign_response') {
        setError(`期望 sign_response,实际: ${env.kind}`);
        setScanSubmitting(false);
        return;
      }
      const body = env.body as { pubkey: string; signature: string };
      if (!body.pubkey || !body.signature) {
        setError('签名二维码缺少 pubkey/signature');
        setScanSubmitting(false);
        return;
      }

      await api.authQrComplete({
        challenge_id: env.id || qrSignRequest.challenge_id,
        session_id: qrSignRequest.session_id,
        admin_account: body.pubkey,
        signature: body.signature,
      });

      const result = await api.authQrResult(qrSignRequest.challenge_id, qrSignRequest.session_id);
      if (result.data?.status === 'SUCCESS' && result.data.user) {
        doLogin(result.data.user);
        return;
      }
      setError('登录验证失败，请重试');
    } catch (e) {
      const msg = e instanceof Error ? e.message : '签名处理失败';
      if (msg.includes('admin not found')) {
        setError('非管理员禁止登录本系统');
      } else if (msg.includes('annual status export required')) {
        const tip = '上一年度数据超过1月10日仍未导出，操作员登录已锁定。请联系管理员在系统设置中导出年度报告后再登录。';
        window.alert(tip);
        setError(tip);
      } else {
        setError(msg);
      }
    } finally {
      setScanSubmitting(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card" style={{ width: 680 }}>
        <div className="login-card__header">
          <div className="login-card__title">公民护照管理系统</div>
          <div className="login-card__subtitle">管理员登录</div>
        </div>
        <div className="login-card__body">
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}

          <div style={{ display: 'flex', gap: 24, alignItems: 'stretch', flexWrap: 'wrap' }}>
            {/* 左侧：登录二维码 */}
            <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>登录二维码</div>
              <div style={{
                width: 260, height: 260,
                background: '#f8fffe',
                borderRadius: 16,
                border: '2px solid #e6f7f5',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                overflow: 'hidden',
              }}>
                <div style={{
                  filter: qrSignRequest ? 'none' : 'blur(3px) opacity(0.4)',
                  transition: 'filter 0.3s ease',
                }}>
                  <QRCodeSVG
                    value={qrSignRequest?.login_qr_payload || 'CPMS_LOGIN_PENDING'}
                    size={228}
                    fgColor="#134e4a"
                  />
                </div>
              </div>
              <div style={{ marginTop: 10, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
                {qrSignRequest
                  ? `有效期至 ${new Date(qrSignRequest.expire_at * 1000).toLocaleTimeString()}`
                  : '请点击按钮生成二维码'}
              </div>
              <button
                className="btn btn--primary"
                style={{ width: 200, marginTop: 10 }}
                onClick={handleGenerateQr}
                disabled={loading}
              >
                {loading ? '生成中...' : qrSignRequest ? '重新生成' : '生成二维码'}
              </button>
            </div>

            {/* 分割线 */}
            <div style={{
              width: 1,
              background: 'linear-gradient(to bottom, transparent, var(--color-border), transparent)',
              alignSelf: 'stretch',
            }} />

            {/* 右侧：扫码窗口 */}
            <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>扫码窗口</div>
              <CameraQrScanner
                active={scannerActive}
                onActiveChange={(active) => {
                  if (active) {
                    if (!qrSignRequest) {
                      setError('请先生成登录二维码');
                      return;
                    }
                    setError('');
                  }
                  setScannerActive(active);
                }}
                onDetected={handleReceiptScanned}
                onError={setError}
                hint="开启摄像头扫描签名响应二维码"
                busy={scanSubmitting}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
