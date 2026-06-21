// 中文注释:独立的登录页组件:渲染未登录态(QR 扫码登录 + 签名挑战应答),
// 持有登录相关 state / handler / useEffect / videoRef,
// 登录成功后通过 useAuth().setAuth 写入全局,App.tsx 只负责在 !auth 时渲染 <LoginView />。

import { useCallback, useEffect, useState } from 'react';
import { Typography } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import { useAuth } from '../hooks/useAuth';
import { writeStoredAuth } from '../utils/storedAuth';
import { parseSignedLoginPayload } from '../utils/parseSignedPayload';
import { CitizenSignaturePanel } from '../core/CitizenSignaturePanel';
import type { AdminAuth } from './types';
import type { AdminQrChallengeResult } from './api';
import {
  completeAdminQrLogin,
  createAdminQrChallenge,
  queryAdminQrLoginResult,
} from './api';
import { notice } from '../utils/notice';

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
  const [scanSubmitting, setScanSubmitting] = useState(false);

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
      notice.success('登录二维码已生成');
    } catch (err) {
      notice.error(err, '生成登录二维码失败');
      setPendingQrLogin(null);
    } finally {
      setChallengeLoading(false);
    }
  };

  const onCompleteSignedLogin = useCallback(async (raw: string) => {
    if (!pendingQrLogin) {
      notice.error('请先生成登录二维码');
      return;
    }
    setScanSubmitting(true);
    try {
      const payload = parseSignedLoginPayload(raw, pendingQrLogin.challenge_id);
      await completeAdminQrLogin({
        challenge_id: payload.challenge_id,
        session_id: payload.session_id || pendingQrLogin.session_id,
        admin_account: payload.admin_account,
        signer_pubkey: payload.signer_pubkey,
        signature: payload.signature,
      });
      const status = await queryAdminQrLoginResult(
        pendingQrLogin.challenge_id,
        pendingQrLogin.session_id,
      );
      if (status.status === 'SUCCESS' && status.access_token && status.admin) {
        const nextAuth: AdminAuth = {
          access_token: status.access_token,
          admin_account: status.admin.admin_account,
          registry_org_code: status.admin.registry_org_code,
          admin_display_name: status.admin.admin_display_name,
          scope_province_name: status.admin.scope_province_name ?? null,
          scope_city_name: status.admin.scope_city_name ?? null,
          passkey_bound: status.admin.passkey_bound,
        };
        setAuth(nextAuth);
        writeStoredAuth(nextAuth);
        setPendingQrLogin(null);
        notice.success('登录成功');
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '';
      if (msg.toLowerCase().includes('admin not found')) {
        notice.error('非管理员禁止登录本系统');
      } else {
        notice.error(err, '登录回执处理失败');
      }
    } finally {
      setScanSubmitting(false);
    }
  }, [pendingQrLogin, setAuth]);

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
          notice.warning('二维码已过期，请重新生成');
          setPendingQrLogin(null);
          return;
        }
        if (status.status === 'SUCCESS' && status.access_token && status.admin) {
          const nextAuth: AdminAuth = {
            access_token: status.access_token,
            admin_account: status.admin.admin_account,
            registry_org_code: status.admin.registry_org_code,
            admin_display_name: status.admin.admin_display_name,
            scope_province_name: status.admin.scope_province_name ?? null,
            scope_city_name: status.admin.scope_city_name ?? null,
            passkey_bound: status.admin.passkey_bound,
          };
          setAuth(nextAuth);
          writeStoredAuth(nextAuth);
          setPendingQrLogin(null);
          notice.success('登录成功');
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
          使用公民钱包扫描登录二维码并回扫登录回执
        </Typography.Text>
      </div>

      {/* 登录内容区域 */}
      <div style={{ padding: '32px 36px 36px' }}>
        <CitizenSignaturePanel
          qrTitle="登录二维码"
          qrValue={pendingQrLogin?.login_qr_payload}
          qrPlaceholderValue="SFID_LOGIN_PENDING"
          qrHint={
            pendingQrLogin
              ? `有效期至 ${new Date(pendingQrLogin.expire_at * 1000).toLocaleTimeString()}`
              : '请点击按钮生成二维码'
          }
          scannerHint="开启摄像头扫描公民钱包生成的登录回执二维码"
          primaryActionText={pendingQrLogin ? '重新生成' : '生成二维码'}
          primaryActionLoading={challengeLoading}
          onPrimaryAction={onCreateQrLogin}
          scannerDisabled={!pendingQrLogin || scanSubmitting}
          scannerLoading={scanSubmitting}
          onDetected={onCompleteSignedLogin}
          onScannerError={(msg) => notice.error(msg)}
        />
      </div>
    </div>
  );
}
