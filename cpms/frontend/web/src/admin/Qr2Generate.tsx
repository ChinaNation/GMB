import { useState, useEffect } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';

export default function Qr2Generate() {
  const [qr2Payload, setQr2Payload] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState<{ qr2_ready: boolean; anon_cert_done: boolean } | null>(null);

  useEffect(() => {
    api.installStatus().then(res => {
      if (res.data) {
        setStatus({ qr2_ready: res.data.qr2_ready, anon_cert_done: res.data.anon_cert_done });
        if (res.data.qr2_payload) setQr2Payload(res.data.qr2_payload);
      }
    }).catch(() => {});
  }, []);

  const handleGenerate = async () => {
    setError('');
    setLoading(true);
    try {
      const res = await api.adminGenerateQr2();
      if (res.data) {
        setQr2Payload(res.data.qr2_payload);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '生成 QR2 失败');
    }
    setLoading(false);
  };

  if (status?.anon_cert_done) {
    return (
      <div className="card">
        <div className="card__title">QR2 注册</div>
        <div style={{ textAlign: 'center', padding: '20px 0' }}>
          <div style={{ fontSize: 36, marginBottom: 12 }}>✅</div>
          <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-success)' }}>
            匿名证书已注册完成，QR2 流程已完成
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <div className="card__title">生成注册二维码（QR2）</div>
      <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 16 }}>
        生成 QR2 后将二维码展示给 SFID 管理员扫码注册，注册完成后 SFID 会返回 QR3。
      </div>

      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}

      {!qr2Payload ? (
        <div style={{ textAlign: 'center' }}>
          <button className="btn btn--primary" onClick={handleGenerate} disabled={loading}>
            {loading ? '生成中...' : '生成 QR2'}
          </button>
        </div>
      ) : (
        <div style={{ textAlign: 'center' }}>
          <div style={{ display: 'inline-block', background: '#fff', padding: 16, borderRadius: 8, marginBottom: 12 }}>
            <QRCodeSVG value={qr2Payload} size={260} fgColor="#134e4a" />
          </div>
          <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
            请将此二维码展示给 SFID 管理员扫码注册
          </div>
          <div style={{ marginTop: 12 }}>
            <button className="btn btn--ghost" onClick={handleGenerate} disabled={loading}>
              {loading ? '重新生成中...' : '重新生成'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
