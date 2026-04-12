// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 5)
// 独立的登录页组件:渲染未登录态(QR 扫码登录 + 签名挑战应答),
// 持有登录相关 state / handler / useEffect / videoRef,
// 登录成功后通过 useAuth().setAuth 写入全局,App.tsx 只负责在 !auth 时渲染 <LoginView />。

import { useEffect, useRef, useState } from 'react';
import { Button, QRCode, Typography, message } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import { useAuth } from '../../hooks/useAuth';
import { writeStoredAuth } from '../../utils/storedAuth';
import { startCameraScanner } from '../../utils/cameraScanner';
import { parseSignedLoginPayload } from '../../utils/parseSignedPayload';
import type { AdminAuth, AdminQrChallengeResult } from '../../api/client';
import {
  completeAdminQrLogin,
  createAdminQrChallenge,
  queryAdminQrLoginResult,
} from '../../api/client';

function createSessionId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `sid-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function LoginView() {
  const { auth, setAuth } = useAuth();
  const [pendingQrLogin, setPendingQrLogin] = useState<AdminQrChallengeResult | null>(null);
  const [challengeLoading, setChallengeLoading] = useState(false);
  const [scannerActive, setScannerActive] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const loginScanCleanupRef = useRef<(() => void) | null>(null);

  const stopScanner = () => {
    if (loginScanCleanupRef.current) {
      loginScanCleanupRef.current();
      loginScanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  const onCreateQrLogin = async () => {
    setChallengeLoading(true);
    try {
      const sessionId = createSessionId();
      const origin = window.location.origin;
      const challenge = await createAdminQrChallenge({
        origin,
        session_id: sessionId,
      });
      setPendingQrLogin(challenge);
      setScannerActive(false);
      stopScanner();
      message.success('登录二维码已生成');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成登录二维码失败';
      message.error(msg);
      setPendingQrLogin(null);
    } finally {
      setChallengeLoading(false);
    }
  };

  const onCompleteSignedLogin = async (raw: string) => {
    if (!pendingQrLogin) {
      message.error('请先生成登录二维码');
      return;
    }
    setScanSubmitting(true);
    try {
      const payload = parseSignedLoginPayload(raw, pendingQrLogin.challenge_id);
      await completeAdminQrLogin({
        challenge_id: payload.challenge_id,
        session_id: payload.session_id || pendingQrLogin.session_id,
        admin_pubkey: payload.admin_pubkey,
        signer_pubkey: payload.signer_pubkey,
        signature: payload.signature,
      });
      stopScanner();
      setScannerActive(false);
      const status = await queryAdminQrLoginResult(
        pendingQrLogin.challenge_id,
        pendingQrLogin.session_id,
      );
      if (status.status === 'SUCCESS' && status.access_token && status.admin) {
        const nextAuth: AdminAuth = {
          access_token: status.access_token,
          admin_pubkey: status.admin.admin_pubkey,
          role: status.admin.role,
          admin_name: status.admin.admin_name,
          admin_province: status.admin.admin_province ?? null,
          admin_city: status.admin.admin_city ?? null,
        };
        setAuth(nextAuth);
        writeStoredAuth(nextAuth);
        setPendingQrLogin(null);
        message.success('登录成功');
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '签名二维码处理失败';
      if (msg.includes('admin not found')) {
        message.error('非管理员禁止登录本系统');
      } else {
        message.error(msg);
      }
    } finally {
      setScanSubmitting(false);
    }
  };

  // 摄像头扫码 effect
  useEffect(() => {
    if (!scannerActive || !pendingQrLogin || !videoRef.current) {
      stopScanner();
      return;
    }
    loginScanCleanupRef.current = startCameraScanner(
      videoRef.current,
      (raw) => {
        setScannerActive(false);
        stopScanner();
        void onCompleteSignedLogin(raw);
      },
      () => setScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scannerActive, pendingQrLogin]);

  // QR 后台轮询 effect
  useEffect(() => {
    if (auth || !pendingQrLogin) return;
    let cancelled = false;
    const timer = window.setInterval(async () => {
      try {
        const status = await queryAdminQrLoginResult(
          pendingQrLogin.challenge_id,
          pendingQrLogin.session_id,
        );
        if (cancelled) return;
        if (status.status === 'PENDING') return;
        if (status.status === 'EXPIRED') {
          message.warning('二维码已过期，请重新生成');
          setPendingQrLogin(null);
          return;
        }
        if (status.status === 'SUCCESS' && status.access_token && status.admin) {
          const nextAuth: AdminAuth = {
            access_token: status.access_token,
            admin_pubkey: status.admin.admin_pubkey,
            role: status.admin.role,
            admin_name: status.admin.admin_name,
            admin_province: status.admin.admin_province ?? null,
            admin_city: status.admin.admin_city ?? null,
          };
          setAuth(nextAuth);
          writeStoredAuth(nextAuth);
          setPendingQrLogin(null);
          message.success('登录成功');
        }
      } catch {
        // keep polling
      }
    }, 1200);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [auth, pendingQrLogin, setAuth]);

  // 组件卸载时 cleanup 摄像头
  useEffect(() => {
    return () => stopScanner();
  }, []);

  const onToggleScanner = () => {
    if (!pendingQrLogin) {
      message.warning('请先生成登录二维码');
      return;
    }
    setScannerActive((v) => !v);
  };

  return (
    <div
      style={{
        width: 780,
        maxWidth: '95vw',
        background: 'rgba(255,255,255,0.92)',
        backdropFilter: 'blur(20px)',
        borderRadius: 20,
        boxShadow: '0 8px 40px rgba(0,0,0,0.12), 0 1px 3px rgba(0,0,0,0.06)',
        border: '1px solid rgba(255,255,255,0.6)',
        overflow: 'hidden',
      }}
    >
      {/* 登录卡片顶部 */}
      <div
        style={{
          background: 'linear-gradient(135deg, #0d9488 0%, #0f766e 50%, #115e59 100%)',
          padding: '28px 32px',
          textAlign: 'center',
        }}
      >
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: 52,
            height: 52,
            borderRadius: 14,
            background: 'rgba(255,255,255,0.18)',
            marginBottom: 12,
            border: '1px solid rgba(255,255,255,0.25)',
          }}
        >
          <QrcodeOutlined style={{ fontSize: 26, color: '#fff' }} />
        </div>
        <Typography.Title
          level={4}
          style={{ color: '#fff', margin: 0, fontWeight: 600, letterSpacing: 1 }}
        >
          管理员扫码登录
        </Typography.Title>
        <Typography.Text style={{ color: 'rgba(255,255,255,0.7)', fontSize: 13 }}>
          使用 公民 移动端扫描二维码完成身份验证
        </Typography.Text>
      </div>

      {/* 登录内容区域 */}
      <div style={{ padding: '32px 36px 36px' }}>
        <div style={{ display: 'flex', gap: 32, alignItems: 'stretch', flexWrap: 'wrap' }}>
          {/* 左侧：QR 码生成 */}
          <div
            style={{
              flex: '1 1 300px',
              minWidth: 280,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
            }}
          >
            <Typography.Text
              strong
              style={{ fontSize: 14, marginBottom: 16, color: '#374151' }}
            >
              登录二维码
            </Typography.Text>
            <div
              style={{
                position: 'relative',
                width: 260,
                height: 260,
                background: '#f8fffe',
                borderRadius: 16,
                border: '2px solid #e6f7f5',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                overflow: 'hidden',
              }}
            >
              <div style={{ position: 'absolute', top: 0, left: 0, width: 20, height: 20, borderTop: '3px solid #0d9488', borderLeft: '3px solid #0d9488', borderTopLeftRadius: 8 }} />
              <div style={{ position: 'absolute', top: 0, right: 0, width: 20, height: 20, borderTop: '3px solid #0d9488', borderRight: '3px solid #0d9488', borderTopRightRadius: 8 }} />
              <div style={{ position: 'absolute', bottom: 0, left: 0, width: 20, height: 20, borderBottom: '3px solid #0d9488', borderLeft: '3px solid #0d9488', borderBottomLeftRadius: 8 }} />
              <div style={{ position: 'absolute', bottom: 0, right: 0, width: 20, height: 20, borderBottom: '3px solid #0d9488', borderRight: '3px solid #0d9488', borderBottomRightRadius: 8 }} />
              <div
                style={{
                  filter: pendingQrLogin ? 'none' : 'blur(3px) opacity(0.4)',
                  transition: 'filter 0.3s ease',
                }}
              >
                <QRCode
                  value={pendingQrLogin?.login_qr_payload || 'SFID_LOGIN_PENDING'}
                  size={228}
                  color="#134e4a"
                />
              </div>
            </div>
            <div style={{ marginTop: 14, textAlign: 'center' }}>
              <Typography.Text
                type="secondary"
                style={{ fontSize: 12, display: 'block', marginBottom: 12 }}
              >
                {pendingQrLogin
                  ? `有效期至 ${new Date(pendingQrLogin.expire_at * 1000).toLocaleTimeString()}`
                  : '请点击按钮生成二维码'}
              </Typography.Text>
              <Button
                type="primary"
                size="large"
                onClick={onCreateQrLogin}
                loading={challengeLoading}
                style={{
                  borderRadius: 10,
                  fontWeight: 500,
                  width: 200,
                  boxShadow: '0 2px 8px rgba(13,148,136,0.3)',
                }}
              >
                {pendingQrLogin ? '重新生成' : '生成二维码'}
              </Button>
            </div>
          </div>

          {/* 分割线 */}
          <div
            style={{
              width: 1,
              background: 'linear-gradient(to bottom, transparent, #e5e7eb, transparent)',
              alignSelf: 'stretch',
              margin: '0 4px',
            }}
          />

          {/* 右侧：摄像头扫码 */}
          <div
            style={{
              flex: '1 1 300px',
              minWidth: 280,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
            }}
          >
            <Typography.Text
              strong
              style={{ fontSize: 14, marginBottom: 16, color: '#374151' }}
            >
              扫码窗口
            </Typography.Text>
            <div
              style={{
                width: 260,
                height: 260,
                background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                borderRadius: 16,
                overflow: 'hidden',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                position: 'relative',
                border: '2px solid #334155',
                boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
              }}
            >
              <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
              <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
              <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
              <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
              <video
                ref={videoRef}
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                muted
                playsInline
              />
              {!scannerReady && (
                <div
                  style={{
                    position: 'absolute',
                    inset: 0,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: 8,
                  }}
                >
                  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.25)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/><rect x="7" y="7" width="10" height="10" rx="1"/></svg>
                  <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                    {scannerActive ? '摄像头初始化中...' : '等待开启摄像头'}
                  </Typography.Text>
                </div>
              )}
            </div>
            <div style={{ marginTop: 14, textAlign: 'center' }}>
              <Typography.Text
                type="secondary"
                style={{ fontSize: 12, display: 'block', marginBottom: 12 }}
              >
                开启摄像头扫描已签名的二维码
              </Typography.Text>
              <Button
                size="large"
                onClick={onToggleScanner}
                disabled={scanSubmitting}
                style={{
                  borderRadius: 10,
                  fontWeight: 500,
                  width: 200,
                }}
              >
                {scannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
