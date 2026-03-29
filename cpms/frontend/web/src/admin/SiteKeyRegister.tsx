import { useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';

export default function SiteKeyRegister() {
  const [qrContent, setQrContent] = useState('');
  const [loading, setLoading] = useState(false);

  const handleGenerate = async () => {
    setLoading(true);
    try {
      const res = await api.siteKeyRegistrationQr();
      if (res.data) setQrContent(res.data.qr_content);
    } catch { /* ignore */ }
    setLoading(false);
  };

  return (
    <div className="card">
      <div className="card__title">站点密钥注册</div>
      <p style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginBottom: 16 }}>
        生成站点密钥注册二维码，供 SFID 系统扫码登记本站点的 QR 签名公钥。
      </p>
      <button className="btn btn--blue" onClick={handleGenerate} disabled={loading}>
        {loading ? '生成中...' : '生成注册二维码'}
      </button>
      {qrContent && (
        <div className="mt-16 text-center">
          <QRCodeSVG value={qrContent} size={240} />
          <div style={{ marginTop: 8, fontSize: 12, color: 'var(--color-text-secondary)' }}>请使用 SFID 系统扫描此二维码</div>
        </div>
      )}
    </div>
  );
}
