import { Alert, Button, Card, Input, Space, Tag, Typography } from 'antd';
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  getChallengeMessage,
  getSignMessage,
  issueLoginChallenge,
  parseSignedLoginPayload,
  type LoginChallenge
} from '../../services/auth/challenge';
import { asHexAddress, resolveCitizenchainSession } from '../../services/auth/organization';
import { generateOfflineQrDataUrl } from '../../services/auth/qr';
import { verifyLoginSignature } from '../../services/auth/verify';
import { useAuthStore } from '../../stores/auth';
import { getOrganizationName } from '../../utils/organization';
import { CameraQrScanner } from './CameraQrScanner';

export function LoginCard() {
  const navigate = useNavigate();
  const { session, login, logout } = useAuthStore();

  const [challenge, setChallenge] = useState<LoginChallenge | null>(null);
  const [scannedPayload, setScannedPayload] = useState('');
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [consumedRequestIds, setConsumedRequestIds] = useState<Set<string>>(new Set());

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
    if (!challenge) {
      setLoginError('请先生成挑战二维码。');
      return;
    }
    if (Math.floor(Date.now() / 1000) > challenge.expiresAt) {
      setLoginError('挑战已过期，请重新生成二维码。');
      return;
    }
    if (!parsedSignature) {
      setLoginError('签名二维码格式错误。请使用标准 JSON：WUMINAPP_LOGIN_V1 回执。');
      return;
    }
    if (consumedRequestIds.has(parsedSignature.request_id)) {
      setLoginError('该回执已使用，请重新生成挑战二维码。');
      return;
    }
    if (parsedSignature.request_id !== challenge.requestId) {
      setLoginError('回执中的 request_id 与当前挑战不一致，请重新扫码。');
      return;
    }

    let signerHex = '';
    try {
      signerHex = asHexAddress(parsedSignature.pubkey);
    } catch {
      setLoginError('签名中的 pubkey 地址格式不合法。');
      return;
    }
    const loginSession = resolveCitizenchainSession(signerHex);
    if (!loginSession) {
      setLoginError('无法识别登录账户。');
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
      setLoginError('调用本地验签失败，请确认桌面壳正在运行。');
      return;
    }

    if (!verified) {
      setLoginError('签名验证失败，请确认签名内容对应当前挑战。');
      return;
    }

    setLoginError(null);
    setChallenge(null);
    setConsumedRequestIds((prev) => new Set(prev).add(parsedSignature.request_id));
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
                  二维码生成失败。请检查网络（浏览器模式）或确认在 Tauri 桌面壳内运行。
                </Typography.Text>
              )}

              <Space direction="vertical" size={10} style={{ minWidth: 320, flex: 1 }}>
                <CameraQrScanner enabled={!!challenge} onDetected={setScannedPayload} />
                <Input.TextArea
                  value={scannedPayload}
                  onChange={(event) => setScannedPayload(event.target.value)}
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
