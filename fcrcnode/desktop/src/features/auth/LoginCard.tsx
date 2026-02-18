import { useMemo, useState } from 'react';
import { Button, Card, CardContent, Chip, Stack, TextField, Typography } from '@mui/material';
import { useNavigate } from 'react-router-dom';
import {
  getChallengeMessage,
  issueLoginChallenge,
  parseSignedLoginPayload,
  type LoginChallenge
} from '../../services/auth/challenge';
import { CameraQrScanner } from './CameraQrScanner';
import { asHexAddress, resolveOrganizationByAddress } from '../../services/auth/organization';
import { generateOfflineQrDataUrl } from '../../services/auth/qr';
import { verifyLoginSignature } from '../../services/auth/verify';
import { useAuthStore } from '../../stores/auth';
import { useSessionStore } from '../../stores/session';
import { isValidAddress } from '../../utils/address';
import { getOrganizationName } from '../../utils/organization';

const textFieldNotchFixSx = {
  '& .MuiInputLabel-root.MuiInputLabel-shrink': {
    backgroundColor: '#f8fafc',
    paddingLeft: '6px',
    paddingRight: '6px',
    borderRadius: '4px'
  }
};

export function LoginCard() {
  const navigate = useNavigate();
  const { state } = useSessionStore();
  const { session, login, logout } = useAuthStore();

  const [publicKey, setPublicKey] = useState('');
  const [challenge, setChallenge] = useState<LoginChallenge | null>(null);
  const [scannedPayload, setScannedPayload] = useState('');
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);

  const inputAddress = publicKey.trim();
  const invalidAddress = inputAddress.length > 0 && !isValidAddress(inputAddress);
  const organization = inputAddress && !invalidAddress ? resolveOrganizationByAddress(inputAddress) : null;

  const canGenerateChallenge = state === 'connected' && !!organization;
  const parsedSignature = parseSignedLoginPayload(scannedPayload);
  const canLogin = useMemo(
    () => canGenerateChallenge && challenge !== null && parsedSignature !== null,
    [canGenerateChallenge, challenge, parsedSignature]
  );

  const generateChallenge = async () => {
    if (!organization) return;

    const next = issueLoginChallenge({
      role: organization.role,
      address: organization.publicKey,
      province: organization.province
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
    if (!organization) {
      setLoginError('未识别到机构，请确认管理员公钥是否在机构注册表中。');
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
      setLoginError('签名二维码格式错误。请使用标准 JSON：fcrc.login.signature v1。');
      return;
    }

    let signerHex = '';
    try {
      signerHex = asHexAddress(parsedSignature.publicKey);
    } catch {
      setLoginError('签名中的 publicKey 地址格式不合法。');
      return;
    }
    if (signerHex !== organization.publicKey.toLowerCase()) {
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
        publicKey: organization.publicKey,
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
    login(organization);
    navigate(`/${organization.role}`);
  };

  return (
    <Card
      sx={{
        backgroundColor: '#f8fafc',
        color: '#0f172a',
        borderRadius: 3,
        border: '1px solid rgba(15, 23, 42, 0.12)',
        boxShadow: '0 22px 54px rgba(0, 0, 0, 0.28)'
      }}
    >
      <CardContent>
        <Stack spacing={2}>
          <Typography variant="h6" sx={{ color: '#0f172a' }}>
            管理员扫码登录
          </Typography>

          <TextField
            label="管理员公钥地址"
            value={publicKey}
            onChange={(event) => {
              setPublicKey(event.target.value);
              setChallenge(null);
              setScannedPayload('');
              setQrDataUrl(null);
              setLoginError(null);
            }}
            fullWidth
            size="small"
            error={invalidAddress}
            helperText={invalidAddress ? '地址格式不合法（SS58）' : '输入后自动识别机构名称'}
            sx={textFieldNotchFixSx}
          />

          {organization ? (
            <Typography variant="body2" sx={{ color: '#8fd19e' }}>
              已识别机构：{organization.organizationName}
            </Typography>
          ) : inputAddress && !invalidAddress ? (
            <Typography variant="body2" sx={{ color: '#e8c26e' }}>
              未识别机构。请将该公钥录入机构注册表后再登录。
            </Typography>
          ) : null}

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
            <Button variant="outlined" onClick={generateChallenge} disabled={!canGenerateChallenge}>
              生成签名二维码
            </Button>
            <Button variant="contained" color="warning" onClick={() => void doLogin()} disabled={!canLogin}>
              验签并登录
            </Button>
          </Stack>

          {challenge ? (
            <Stack spacing={1.5}>
              <Typography variant="body2" sx={{ color: '#475569' }}>
                请用手机钱包/硬件钱包扫码挑战二维码并返回签名二维码。
              </Typography>
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="flex-start">
                {qrDataUrl ? (
                  <img src={qrDataUrl} alt="login challenge qr" width={220} height={220} />
                ) : (
                  <Typography variant="body2" sx={{ color: '#e8c26e' }}>
                    离线二维码未生成。请确认在 Tauri 桌面壳内运行。
                  </Typography>
                )}
                <Stack spacing={1.2} sx={{ width: '100%' }}>
                  <CameraQrScanner enabled={!!challenge} onDetected={setScannedPayload} />
                  <TextField
                    label="扫码得到的签名 JSON（备用可手动粘贴）"
                    value={scannedPayload}
                    onChange={(event) => setScannedPayload(event.target.value)}
                    size="small"
                    fullWidth
                    multiline
                    minRows={4}
                    sx={textFieldNotchFixSx}
                  />
                </Stack>
              </Stack>
            </Stack>
          ) : null}

          {loginError ? (
            <Typography variant="body2" sx={{ color: '#ff8a80' }}>
              {loginError}
            </Typography>
          ) : null}

          {session ? (
            <Stack direction="row" spacing={1} alignItems="center">
              <Chip size="small" color="success" label={getOrganizationName(session)} />
              <Typography variant="body2">地址：{session.publicKey}</Typography>
              <Button
                color="inherit"
                size="small"
                onClick={() => {
                  logout();
                  navigate('/');
                }}
              >
                退出
              </Button>
            </Stack>
          ) : (
            <Typography variant="body2" sx={{ color: '#475569' }}>
              请输入管理员公钥后开始扫码登录。
            </Typography>
          )}
        </Stack>
      </CardContent>
    </Card>
  );
}
