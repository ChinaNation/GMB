// 登录页：Sr25519 签名挑战验证

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../auth';
import * as api from '../api';

export default function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();

  const [pubkey, setPubkey] = useState('');
  const [challenge, setChallenge] = useState<{ challenge_id: string; challenge_payload: string } | null>(null);
  const [signature, setSignature] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleGetChallenge = async () => {
    if (!pubkey.trim()) { setError('请输入管理员公钥'); return; }
    setError('');
    setLoading(true);
    try {
      const res = await api.authChallenge(pubkey.trim());
      if (res.data) setChallenge(res.data);
    } catch (e) {
      setError(e instanceof Error ? e.message : '获取挑战失败');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    if (!challenge || !signature.trim()) { setError('请输入签名'); return; }
    setError('');
    setLoading(true);
    try {
      const res = await api.authVerify(challenge.challenge_id, pubkey.trim(), signature.trim());
      if (res.data) {
        login(res.data.access_token, res.data.user);
        navigate(res.data.user.role === 'SUPER_ADMIN' ? '/admin' : '/operator');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '验证失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="login-card__header">
          <div className="login-card__title">CPMS</div>
          <div className="login-card__subtitle">公民护照管理系统</div>
        </div>
        <div className="login-card__body">
          {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{error}</div>}

          <div className="form-group">
            <label>管理员公钥</label>
            <input className="form-input" placeholder="Sr25519 公钥（hex 或 base64）" value={pubkey} onChange={e => setPubkey(e.target.value)} />
          </div>

          {!challenge ? (
            <button className="btn btn--primary" style={{ width: '100%' }} onClick={handleGetChallenge} disabled={loading}>
              {loading ? '请求中...' : '获取签名挑战'}
            </button>
          ) : (
            <>
              <div className="form-group">
                <label>签名挑战</label>
                <textarea
                  className="form-input"
                  rows={3}
                  readOnly
                  value={challenge.challenge_payload}
                  style={{ fontSize: 12, fontFamily: 'monospace', resize: 'none', height: 'auto' }}
                />
              </div>
              <div className="form-group">
                <label>签名结果</label>
                <input className="form-input" placeholder="对上述挑战的 Sr25519 签名（hex 或 base64）" value={signature} onChange={e => setSignature(e.target.value)} />
              </div>
              <button className="btn btn--primary" style={{ width: '100%' }} onClick={handleVerify} disabled={loading}>
                {loading ? '验证中...' : '验证并登录'}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
