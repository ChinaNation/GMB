import { Alert, Button, Card, Input, Space, Tag, Typography } from 'antd';
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  getChallengeMessage,
  issueLoginChallenge,
  parseSignedLoginPayload,
  type LoginChallenge
} from '../../services/auth/challenge';
import { asHexAddress, resolveOrganizationByAddress } from '../../services/auth/organization';
import { generateOfflineQrDataUrl } from '../../services/auth/qr';
import { verifyLoginSignature } from '../../services/auth/verify';
import { useAuthStore } from '../../stores/auth';
import { isValidAddress } from '../../utils/address';
import { formatNodeDisplayName, getOrganizationName } from '../../utils/organization';
import { CameraQrScanner } from './CameraQrScanner';

export function LoginCard() {
  const navigate = useNavigate();
  const { session, login, logout } = useAuthStore();

  const [publicKey, setPublicKey] = useState('');
  const [challenge, setChallenge] = useState<LoginChallenge | null>(null);
  const [scannedPayload, setScannedPayload] = useState('');
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);

  const inputAddress = publicKey.trim();
  const invalidAddress = inputAddress.length > 0 && !isValidAddress(inputAddress);
  const organization = inputAddress && !invalidAddress ? resolveOrganizationByAddress(inputAddress) : null;

  const canGenerateChallenge = !!inputAddress && !invalidAddress;
  const parsedSignature = parseSignedLoginPayload(scannedPayload);
  const canLogin = useMemo(
    () => canGenerateChallenge && challenge !== null && parsedSignature !== null,
    [canGenerateChallenge, challenge, parsedSignature]
  );

  const generateChallenge = async () => {
    if (!inputAddress || invalidAddress) return;
    let loginAddress = organization?.publicKey;
    if (!loginAddress) {
      try {
        loginAddress = asHexAddress(inputAddress);
      } catch {
        setLoginError('输入地址无法解码，请确认 SS58 或 0x 地址正确。');
        return;
      }
    }
    const loginRole = organization?.role ?? 'full';
    const next = issueLoginChallenge({
      role: loginRole,
      address: loginAddress,
      province: organization?.province
    });

    setChallenge(next);
    setScannedPayload('');
    setLoginError(null);
    setQrDataUrl(null);
    const message = getChallengeMessage(next);
    const offlineQr = await generateOfflineQrDataUrl(message);
    setQrDataUrl(offlineQr);
  };

  const doLogin = async () => {
    if (!inputAddress || invalidAddress) {
      setLoginError('请输入合法的公钥/SS58 地址。');
      return;
    }
    if (!challenge) {
      setLoginError('请先生成挑战二维码。');
      return;
    }
    if (Date.now() - challenge.issuedAt > 120_000) {
      setLoginError('挑战已过期，请重新生成二维码。');
      return;
    }
    if (!parsedSignature) {
      setLoginError('签名二维码格式错误。请使用标准 JSON：citizennode.login.signature v1。');
      return;
    }

    let fallbackPublicKey = '';
    if (!organization) {
      try {
        fallbackPublicKey = asHexAddress(inputAddress);
      } catch {
        setLoginError('输入地址无法解码，请确认 SS58 或 0x 地址正确。');
        return;
      }
    }
    const loginSession = organization ?? {
      role: 'full' as const,
      publicKey: fallbackPublicKey,
      organizationName: 'SFID 本地管理员'
    };

    let signerHex = '';
    try {
      signerHex = asHexAddress(parsedSignature.publicKey);
    } catch {
      setLoginError('签名中的 publicKey 地址格式不合法。');
      return;
    }
    if (signerHex !== loginSession.publicKey.toLowerCase()) {
      setLoginError('签名中的 publicKey 与当前管理员公钥不一致。');
      return;
    }
    if (parsedSignature.nonce !== challenge.nonce) {
      setLoginError('签名中的 nonce 与当前挑战不一致，请重新扫码。');
      return;
    }

    const payload = getChallengeMessage(challenge);
    let verified = false;
    try {
      verified = await verifyLoginSignature({
        payload,
        signature: parsedSignature.signature,
        publicKey: loginSession.publicKey,
        crypto: parsedSignature.crypto
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

        <div>
          <Typography.Text>管理员公钥地址</Typography.Text>
          <Input
            value={publicKey}
            onChange={(event) => {
              setPublicKey(event.target.value);
              setChallenge(null);
              setScannedPayload('');
              setQrDataUrl(null);
              setLoginError(null);
            }}
            placeholder="输入公钥或 SS58 地址"
            status={invalidAddress ? 'error' : ''}
          />
          <div style={{ marginTop: 6 }}>
            {invalidAddress ? (
              <Typography.Text type="danger">地址格式不合法（SS58）</Typography.Text>
            ) : (
              <Typography.Text type="secondary">输入后自动识别机构名称</Typography.Text>
            )}
          </div>
        </div>

        {organization ? (
          <Typography.Text style={{ color: '#1d7a46' }}>
            已识别机构：{formatNodeDisplayName(organization)}
          </Typography.Text>
        ) : inputAddress && !invalidAddress ? (
          <Typography.Text>SFID 本地管理员</Typography.Text>
        ) : null}

        <Space wrap>
          <Button onClick={generateChallenge} disabled={!canGenerateChallenge}>
            生成签名二维码
          </Button>
          <Button type="primary" onClick={() => void doLogin()} disabled={!canLogin}>
            验签并登录
          </Button>
        </Space>

        {challenge ? (
          <Space direction="vertical" size={12} style={{ width: '100%' }}>
            <Typography.Text type="secondary">
              请用手机钱包/硬件钱包扫码挑战二维码并返回签名二维码。
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
                  placeholder="扫码得到的签名 JSON（备用可手动粘贴）"
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
          <Typography.Text type="secondary">请输入管理员公钥后开始扫码登录。</Typography.Text>
        )}
      </Space>
    </Card>
  );
}
