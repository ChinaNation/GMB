import { Alert, Button, Card, Input, Space, Tag, Typography } from 'antd';
import { type ChangeEvent, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  getChallengeMessage,
  getSignMessage,
  issueLoginChallenge,
  parseSignedLoginPayload,
  type LoginChallenge
} from '../../services/auth/challenge';
import {
  AmbiguousAdminMappingError,
  asHexAddress,
  resolveCitizenchainSessionRuntime
} from '../../services/auth/organization';
import { generateOfflineQrDataUrl } from '../../services/auth/qr';
import { verifyLoginSignature } from '../../services/auth/verify';
import { useAuthStore } from '../../stores/auth';
import { getOrganizationName } from '../../utils/organization';
import { CameraQrScanner } from './CameraQrScanner';

const CONSUMED_REQUEST_ID_STORAGE_KEY = 'citizenui.auth.consumedRequestIds';
const MAX_CONSUMED_REQUEST_IDS = 200;

function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

function loadConsumedRequestIds(): Set<string> {
  try {
    const raw = localStorage.getItem(CONSUMED_REQUEST_ID_STORAGE_KEY);
    if (!raw) {
      return new Set();
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return new Set();
    }
    const values = parsed.filter((item): item is string => typeof item === 'string' && item.trim().length > 0);
    return new Set(values.slice(-MAX_CONSUMED_REQUEST_IDS));
  } catch {
    return new Set();
  }
}

function persistConsumedRequestIds(ids: Set<string>): void {
  const trimmed = Array.from(ids).slice(-MAX_CONSUMED_REQUEST_IDS);
  localStorage.setItem(CONSUMED_REQUEST_ID_STORAGE_KEY, JSON.stringify(trimmed));
}

export function LoginCard() {
  const navigate = useNavigate();
  const { session, login, logout } = useAuthStore();

  const [challenge, setChallenge] = useState<LoginChallenge | null>(null);
  const [scannedPayload, setScannedPayload] = useState('');
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [consumedRequestIds, setConsumedRequestIds] = useState<Set<string>>(() => loadConsumedRequestIds());

  const canGenerateChallenge = true;
  const parsedSignature = parseSignedLoginPayload(scannedPayload);
  const canLogin = useMemo(
    () => challenge !== null && parsedSignature !== null,
    [challenge, parsedSignature]
  );

  const generateChallenge = async () => {
    const next = issueLoginChallenge();

    setChallenge(next);
    setScannedPayload('');
    setLoginError(null);
    setQrDataUrl(null);
    const message = getChallengeMessage(next);
    const offlineQr = await generateOfflineQrDataUrl(message);
    setQrDataUrl(offlineQr);
  };

  const doLogin = async () => {
    const reject = (message: string) => {
      setLoginError(message);
    };
    if (!challenge) {
      reject('请先生成挑战二维码。');
      return;
    }
    if (nowSeconds() > challenge.expiresAt) {
      reject('挑战已过期，请重新生成二维码。');
      return;
    }
    if (!parsedSignature) {
      reject('签名二维码格式错误。请使用标准 JSON：WUMINAPP_LOGIN_V1 回执。');
      return;
    }
    if (consumedRequestIds.has(parsedSignature.request_id)) {
      reject('该回执已使用，请重新生成挑战二维码。');
      return;
    }
    if (parsedSignature.request_id !== challenge.requestId) {
      reject('回执中的 request_id 与当前挑战不一致，请重新扫码。');
      return;
    }
    if (parsedSignature.signed_at < challenge.issuedAt || parsedSignature.signed_at > challenge.expiresAt) {
      reject('回执签名时间不在挑战有效期内，请重新生成二维码。');
      return;
    }

    let signerHex = '';
    try {
      signerHex = asHexAddress(parsedSignature.pubkey);
    } catch {
      reject('签名中的 pubkey 地址格式不合法。');
      return;
    }
    let loginSession;
    try {
      loginSession = (await resolveCitizenchainSessionRuntime(signerHex)) ?? {
        role: 'full' as const,
        publicKey: signerHex,
        organizationName: '全节点'
      };
    } catch (error) {
      if (error instanceof AmbiguousAdminMappingError) {
        reject('管理员授权数据异常：同一公钥命中多个机构，请稍后重试或联系运维处理。');
        return;
      }
      reject('读取管理员授权快照失败，请稍后重试。');
      return;
    }

    const payload = getSignMessage(challenge);
    let verified = false;
    try {
      verified = await verifyLoginSignature({
        payload,
        signature: parsedSignature.signature,
        publicKey: signerHex,
        crypto: parsedSignature.sig_alg
      });
    } catch {
      reject('调用本地验签失败，请确认桌面壳正在运行。');
      return;
    }

    if (!verified) {
      reject('签名验证失败，请确认签名内容对应当前挑战。');
      return;
    }

    setLoginError(null);
    setChallenge(null);
    setConsumedRequestIds((prev) => {
      const next = new Set(prev).add(parsedSignature.request_id);
      while (next.size > MAX_CONSUMED_REQUEST_IDS) {
        const oldest = next.values().next().value as string | undefined;
        if (!oldest) break;
        next.delete(oldest);
      }
      persistConsumedRequestIds(next);
      return next;
    });
    setScannedPayload('');
    login(loginSession);
    navigate(`/${loginSession.role}`);
  };

  return (
    <Card style={{ borderRadius: 12 }}>
      <Space direction="vertical" size={14} style={{ width: '100%' }}>
        <Typography.Title level={4} style={{ margin: 0 }}>
          管理员扫码登录
        </Typography.Title>

        <Typography.Text type="secondary">点击生成登录二维码，手机扫码签名后回扫即登录。</Typography.Text>

        <Space wrap>
          <Button onClick={generateChallenge} disabled={!canGenerateChallenge}>
            生成登录二维码
          </Button>
          <Button type="primary" onClick={() => void doLogin()} disabled={!canLogin}>
            验签并登录
          </Button>
        </Space>

        {challenge ? (
          <Space direction="vertical" size={12} style={{ width: '100%' }}>
            <Typography.Text type="secondary">
              请使用 wuminapp 扫描挑战二维码并回扫回执二维码。
            </Typography.Text>

            <Space size={16} align="start" wrap>
              {qrDataUrl ? (
                <img src={qrDataUrl} alt="login challenge qr" width={220} height={220} />
              ) : (
                <Typography.Text type="warning">
                  二维码生成失败。请在 Tauri 桌面壳内运行（开发环境浏览器模式仅用于调试）。
                </Typography.Text>
              )}

              <Space direction="vertical" size={10} style={{ minWidth: 320, flex: 1 }}>
                <CameraQrScanner enabled={!!challenge} onDetected={setScannedPayload} />
                <Input.TextArea
                  value={scannedPayload}
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setScannedPayload(event.target.value)
                  }
                  placeholder="扫码得到的 WUMINAPP_LOGIN_V1 回执 JSON（备用可手动粘贴）"
                  rows={5}
                />
              </Space>
            </Space>
          </Space>
        ) : null}

        {loginError ? <Alert type="error" showIcon message={loginError} /> : null}

        {session ? (
          <Space wrap>
            <Tag color="green">{getOrganizationName(session)}</Tag>
            <Typography.Text>地址：{session.publicKey}</Typography.Text>
            <Button
              onClick={() => {
                logout();
                navigate('/');
              }}
            >
              退出
            </Button>
          </Space>
        ) : (
          <Typography.Text type="secondary">系统不会手工录入公钥，公钥来自扫码回执内容。</Typography.Text>
        )}
      </Space>
    </Card>
  );
}
