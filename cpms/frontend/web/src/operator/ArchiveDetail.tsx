import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { Archive } from '../types';

export default function ArchiveDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [archive, setArchive] = useState<Archive | null>(null);
  const [qrContent, setQrContent] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    api.getArchive(id).then(res => { if (res.data) setArchive(res.data); }).finally(() => setLoading(false));
  }, [id]);

  const handleGenerateQr = async () => {
    if (!id) return;
    try {
      const res = await api.generateQr(id);
      if (res.data) setQrContent(res.data.qr_content);
    } catch { /* ignore */ }
  };

  const handlePrint = async () => {
    if (!id) return;
    try { await api.printQr(id); alert('打印记录已保存'); } catch { /* ignore */ }
  };

  if (loading) return <div className="card">加载中...</div>;
  if (!archive) return <div className="card">档案不存在</div>;

  return (
    <>
      <div className="card">
        <div className="card__title flex-between">
          档案详情
          <button className="btn btn--ghost btn--sm" onClick={() => navigate('/admin')}>返回列表</button>
        </div>
        <div style={{ fontSize: 13, color: 'var(--color-text-secondary)', marginBottom: 16 }}>档案号：{archive.archive_no}</div>
        <div className="form-row">
          <div><strong>姓名：</strong>{archive.full_name}</div>
          <div><strong>省份：</strong>{archive.province_code}</div>
        </div>
        <div className="form-row mt-16">
          <div><strong>性别：</strong>{archive.gender_code === 'M' ? '男' : '女'}</div>
          <div><strong>城市：</strong>{archive.city_code}</div>
        </div>
        <div className="form-row mt-16">
          <div><strong>出生日期：</strong>{archive.birth_date}</div>
          <div><strong>身高：</strong>{archive.height_cm ? `${archive.height_cm} cm` : '未填写'}</div>
        </div>
        <div className="form-row mt-16">
          <div><strong>护照号：</strong>{archive.passport_no}</div>
          <div><strong>公民状态：</strong>
            <span className={`tag ${archive.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--danger'}`}>
              {archive.citizen_status === 'NORMAL' ? '正常' : '异常'}
            </span>
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card__title">QR 码操作</div>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn--blue" onClick={handleGenerateQr}>生成 QR 码</button>
          <button className="btn btn--ghost" onClick={handlePrint}>记录打印</button>
        </div>
        {qrContent && (
          <div className="mt-16 text-center">
            <QRCodeSVG value={qrContent} size={260} />
            <div style={{ marginTop: 8, fontSize: 12, color: 'var(--color-text-secondary)' }}>扫描上方二维码查看公民护照信息</div>
          </div>
        )}
      </div>
    </>
  );
}
