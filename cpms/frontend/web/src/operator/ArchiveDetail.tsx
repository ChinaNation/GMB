// 公民档案详情页。左侧公民信息（可编辑），右侧 ARCHIVE 二维码（自动生成，仅下载）。
// 下方公民资料区域预留给出生纸、证件照等上传。

import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { Archive } from '../types';
import { parseQrEnvelope } from '../qr/wuminQr';
import { scanImageQr, startCameraScanner } from '../utils/cameraScanner';

export default function ArchiveDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [archive, setArchive] = useState<Archive | null>(null);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editForm, setEditForm] = useState<Record<string, unknown>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [walletAddressRaw, setWalletAddressRaw] = useState('');
  const [walletModalOpen, setWalletModalOpen] = useState(false);
  const [walletScanError, setWalletScanError] = useState('');
  const [walletBusy, setWalletBusy] = useState(false);
  const walletVideoRef = useRef<HTMLVideoElement | null>(null);
  const walletScanCleanupRef = useRef<(() => void) | null>(null);
  // 名称解析
  const [provinceName, setProvinceName] = useState('');
  const [cityName, setCityName] = useState('');
  const [townName, setTownName] = useState('');
  const [villageName, setVillageName] = useState('');
  // 编辑用镇村列表
  const [towns, setTowns] = useState<{ town_code: string; town_name: string }[]>([]);
  const [villages, setVillages] = useState<{ village_id: string; village_name: string }[]>([]);

  const loadArchive = () => {
    if (!id) return;
    api.getArchive(id).then(res => { if (res.data) setArchive(res.data); }).finally(() => setLoading(false));
  };

  useEffect(() => {
    loadArchive();
    api.installStatus().then(res => {
      if (res.data?.province_name) setProvinceName(res.data.province_name);
      if (res.data?.city_name) setCityName(res.data.city_name);
    }).catch(() => {});
    api.listTowns().then(res => { if (res.data) setTowns(res.data); }).catch(() => {});
  }, [id]);

  // 解析镇村名称
  useEffect(() => {
    if (!archive?.town_code) return;
    const t = towns.find(t => t.town_code === archive.town_code);
    if (t) setTownName(t.town_name);
    if (archive.village_id) {
      api.listVillages(archive.town_code).then(res => {
        if (res.data) {
          setVillages(res.data);
          const v = res.data.find(v => v.village_id === archive.village_id);
          if (v) setVillageName(v.village_name);
        }
      }).catch(() => {});
    }
  }, [archive?.town_code, archive?.village_id, towns]);

  useEffect(() => {
    if (!walletModalOpen || !walletVideoRef.current) return;
    setWalletScanError('');
    walletScanCleanupRef.current?.();
    walletScanCleanupRef.current = startCameraScanner(
      walletVideoRef.current,
      (raw) => {
        setWalletAddressRaw(raw);
        void handleBindWallet(raw);
      },
      () => setWalletScanError(''),
      (msg) => setWalletScanError(msg),
    );
    return () => {
      walletScanCleanupRef.current?.();
      walletScanCleanupRef.current = null;
    };
  }, [walletModalOpen]);

  const startEdit = () => {
    if (!archive) return;
    setEditForm({
      full_name: archive.full_name,
      birth_date: archive.birth_date,
      gender_code: archive.gender_code,
      height_cm: archive.height_cm ?? '',
      town_code: archive.town_code,
      village_id: archive.village_id,
      address: archive.address,
      citizen_status: archive.citizen_status,
      voting_eligible: archive.voting_eligible,
    });
    setEditing(true);
    setError('');
    // 加载编辑用村列表
    if (archive.town_code) {
      api.listVillages(archive.town_code).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
    }
  };

  const handleEditTownChange = (tc: string) => {
    setEditForm(f => ({ ...f, town_code: tc, village_id: '' }));
    if (tc) {
      api.listVillages(tc).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
    } else {
      setVillages([]);
    }
  };

  const handleSave = async () => {
    if (!id) return;
    setError('');
    setSaving(true);
    try {
      const body: Record<string, unknown> = { ...editForm };
      if (body.height_cm === '' || body.height_cm === null) delete body.height_cm;
      else body.height_cm = parseFloat(String(body.height_cm));
      const res = await api.updateArchive(id, body);
      if (res.data) setArchive(res.data);
      setEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : '保存失败');
    }
    setSaving(false);
  };

  // ARCHIVE 二维码下载
  const handleDownloadArchiveQr = () => {
    const svg = document.querySelector('[data-archive-qr] svg');
    if (!svg) return;
    const svgData = new XMLSerializer().serializeToString(svg);
    const canvas = document.createElement('canvas');
    canvas.width = 420; canvas.height = 420;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, 420, 420);
    const img = new Image();
    img.onload = () => {
      ctx.drawImage(img, 10, 10, 400, 400);
      const a = document.createElement('a');
      a.href = canvas.toDataURL('image/png');
      a.download = `archive-${archive?.archive_no || 'unknown'}.png`;
      a.click();
    };
    img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgData)));
  };

  const openWalletScanner = () => {
    setError('');
    setWalletScanError('');
    setWalletAddressRaw('');
    setWalletModalOpen(true);
  };

  const handleUploadWalletResponse = async (file: File) => {
    try {
      const raw = await scanImageQr(file);
      setWalletAddressRaw(raw);
      await handleBindWallet(raw);
    } catch (e) {
      setWalletScanError(e instanceof Error ? e.message : '钱包二维码识别失败');
    }
  };

  const extractWalletAddress = (raw: string) => {
    const text = raw.trim();
    try {
      const env = parseQrEnvelope(text);
      if (env.kind !== 'user_contact') {
        throw new Error('请扫描 wuminapp 的钱包地址二维码');
      }
      const body = env.body as { address: string };
      return body.address.trim();
    } catch (e) {
      if (text.startsWith('{')) throw e;
      return text;
    }
  };

  const handleBindWallet = async (rawOverride?: string) => {
    if (!id) return;
    setError('');
    setWalletScanError('');
    setWalletBusy(true);
    try {
      const walletAddress = extractWalletAddress(rawOverride ?? walletAddressRaw);
      const res = await api.bindArchiveWallet(id, walletAddress);
      if (res.data) setArchive(res.data);
      walletScanCleanupRef.current?.();
      walletScanCleanupRef.current = null;
      setWalletModalOpen(false);
      setWalletAddressRaw('');
    } catch (e) {
      setWalletScanError(e instanceof Error ? e.message : '保存钱包账户失败');
    } finally {
      setWalletBusy(false);
    }
  };

  const closeWalletModal = () => {
    walletScanCleanupRef.current?.();
    walletScanCleanupRef.current = null;
    setWalletModalOpen(false);
    setWalletAddressRaw('');
    setWalletScanError('');
  };

  const handleGenerateArchiveQr = async () => {
    if (!id) return;
    setError('');
    setWalletBusy(true);
    try {
      const res = await api.generateArchiveQr(id);
      if (res.data?.qr_content && archive) {
        setArchive({ ...archive, archive_qr_payload: res.data.qr_content });
      } else {
        loadArchive();
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '生成档案码失败');
    } finally {
      setWalletBusy(false);
    }
  };

  if (loading) return <div className="card">加载中...</div>;
  if (!archive) return <div className="card">档案不存在</div>;

  return (
    <>
      <div className="card">
        <div className="card__title flex-between">
          公民档案详情
          <div style={{ display: 'flex', gap: 8 }}>
            {!editing && <button className="btn btn--primary btn--sm" onClick={startEdit}>编辑</button>}
            <button className="btn btn--ghost btn--sm" onClick={() => navigate('/admin')}>返回列表</button>
          </div>
        </div>
        {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{error}</div>}
        <div style={{ fontSize: 13, color: 'var(--color-text-secondary)', marginBottom: 16 }}>档案号：{archive.archive_no}</div>

        <div style={{ display: 'flex', gap: 32, alignItems: 'flex-start' }}>
          {/* 左侧：公民信息 */}
          <div style={{ flex: 1 }}>
            {editing ? (
              <>
                <div className="form-row">
                  <div className="form-group"><label>姓名</label><input className="form-input" value={String(editForm.full_name || '')} onChange={e => setEditForm(f => ({ ...f, full_name: e.target.value }))} /></div>
                  <div className="form-group"><label>出生日期</label><input className="form-input" type="date" value={String(editForm.birth_date || '')} onChange={e => setEditForm(f => ({ ...f, birth_date: e.target.value }))} /></div>
                </div>
                <div className="form-row mt-16">
                  <div className="form-group">
                    <label>性别</label>
                    <select className="form-input" value={String(editForm.gender_code || 'M')} onChange={e => setEditForm(f => ({ ...f, gender_code: e.target.value }))}>
                      <option value="M">男</option><option value="W">女</option>
                    </select>
                  </div>
                  <div className="form-group"><label>身高 (cm)</label><input className="form-input" type="number" value={String(editForm.height_cm ?? '')} onChange={e => setEditForm(f => ({ ...f, height_cm: e.target.value }))} /></div>
                </div>
                <div className="form-row mt-16">
                  <div className="form-group">
                    <label>镇/街道</label>
                    <select className="form-input" value={String(editForm.town_code || '')} onChange={e => handleEditTownChange(e.target.value)}>
                      <option value="">请选择</option>
                      {towns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
                    </select>
                  </div>
                  <div className="form-group">
                    <label>村/路</label>
                    <select className="form-input" value={String(editForm.village_id || '')} onChange={e => setEditForm(f => ({ ...f, village_id: e.target.value }))}>
                      <option value="">请选择</option>
                      {villages.map(v => <option key={v.village_id} value={v.village_id}>{v.village_name}</option>)}
                    </select>
                  </div>
                </div>
                <div className="form-group mt-16"><label>具体地址</label><input className="form-input" maxLength={100} value={String(editForm.address || '')} onChange={e => setEditForm(f => ({ ...f, address: e.target.value }))} /></div>
                <div className="form-row mt-16">
                  <div className="form-group">
                    <label>公民状态</label>
                    <select className="form-input" value={String(editForm.citizen_status || 'NORMAL')} onChange={e => setEditForm(f => ({ ...f, citizen_status: e.target.value }))}>
                      <option value="NORMAL">正常</option><option value="ABNORMAL">异常</option>
                    </select>
                  </div>
                  <div className="form-group">
                    <label>选举资格</label>
                    <select className="form-input" value={String(editForm.voting_eligible ?? true)} onChange={e => setEditForm(f => ({ ...f, voting_eligible: e.target.value === 'true' }))}>
                      <option value="true">有选举资格</option><option value="false">无选举资格</option>
                    </select>
                  </div>
                </div>
                <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
                  <button className="btn btn--primary" onClick={handleSave} disabled={saving}>{saving ? '保存中...' : '保存'}</button>
                  <button className="btn btn--ghost" onClick={() => setEditing(false)}>取消</button>
                </div>
              </>
            ) : (
              <>
                <div className="form-row">
                  <div><strong>姓名：</strong>{archive.full_name}</div>
                  <div><strong>省份：</strong>{provinceName || archive.province_code}</div>
                </div>
                <div className="form-row mt-16">
                  <div><strong>性别：</strong>{archive.gender_code === 'M' ? '男' : '女'}</div>
                  <div><strong>城市：</strong>{cityName || archive.city_code}</div>
                </div>
                <div className="form-row mt-16">
                  <div><strong>出生日期：</strong>{archive.birth_date}</div>
                  <div><strong>身高：</strong>{archive.height_cm ? `${archive.height_cm} cm` : '未填写'}</div>
                </div>
                {(archive.town_code || archive.village_id || archive.address) && (
                  <div className="form-row mt-16">
                    <div><strong>地址：</strong>{[townName, villageName, archive.address].filter(Boolean).join(' ')}</div>
                  </div>
                )}
                <div className="form-row mt-16">
                  <div><strong>公民状态：</strong>
                    <span className={`tag ${archive.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--danger'}`}>
                      {archive.citizen_status === 'NORMAL' ? '正常' : '异常'}
                    </span>
                  </div>
                  <div><strong>选举资格：</strong>
                    <span className={`tag ${archive.voting_eligible ? 'tag--success' : 'tag--warning'}`}>
                      {archive.voting_eligible ? '有选举资格' : '无选举资格'}
                    </span>
                  </div>
                </div>
                <div className="form-row mt-16">
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                    <strong>钱包账户：</strong>
                    <span>{archive.wallet_address || '未绑定'}</span>
                    <button className="btn btn--primary btn--sm" onClick={openWalletScanner} disabled={walletBusy}>
                      {archive.wallet_address ? '更新' : '绑定'}
                    </button>
                  </div>
                </div>
                {archive.wallet_pubkey && (
                  <div className="form-row mt-16">
                    <div style={{ wordBreak: 'break-all' }}><strong>钱包公钥：</strong>{archive.wallet_pubkey}</div>
                  </div>
                )}
              </>
            )}
          </div>

          {/* 右侧：ARCHIVE 二维码 */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8, minWidth: 200 }}>
            {archive.archive_qr_payload ? (
              <>
                <div data-archive-qr="" style={{ lineHeight: 0 }}>
                  <QRCodeSVG value={archive.archive_qr_payload} size={200} />
                </div>
                <button className="btn btn--ghost btn--sm" onClick={handleDownloadArchiveQr}>下载二维码</button>
              </>
            ) : (
              <div style={{ width: 200, height: 200, background: '#f3f4f6', borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)', fontSize: 13 }}>
                签发未就绪
              </div>
            )}
            <button className="btn btn--primary btn--sm" onClick={handleGenerateArchiveQr} disabled={walletBusy || !archive.wallet_pubkey}>
              生成档案码
            </button>
          </div>
        </div>
      </div>

      {walletModalOpen && (
        <div className="modal-overlay">
          <div className="modal" style={{ maxWidth: 560 }}>
            <div className="modal__title">识别钱包账户</div>
            {walletScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{walletScanError}</div>}
            <video
              ref={walletVideoRef}
              muted
              playsInline
              style={{ width: '100%', aspectRatio: '1 / 1', objectFit: 'cover', borderRadius: 8, background: '#111827' }}
            />
            <div style={{ color: 'var(--color-text-secondary)', fontSize: 12, marginTop: 8 }}>
              请让用户打开 wuminapp 电子护照页面的钱包二维码，对准本摄像头；识别成功会直接保存钱包账户。
            </div>
            <div className="form-group mt-16">
              <label>钱包二维码原文</label>
              <textarea
                className="form-input"
                style={{ minHeight: 72 }}
                value={walletAddressRaw}
                onChange={e => setWalletAddressRaw(e.target.value)}
                placeholder="摄像头不可用时，可粘贴或上传 wuminapp 钱包二维码"
              />
            </div>
            <div style={{ display: 'flex', gap: 8, marginTop: 8, flexWrap: 'wrap' }}>
              <label className="btn btn--ghost">
                上传钱包二维码
                <input type="file" accept="image/*" style={{ display: 'none' }} onChange={e => {
                  const file = e.target.files?.[0];
                  if (file) void handleUploadWalletResponse(file);
                  e.currentTarget.value = '';
                }} />
              </label>
              <button className="btn btn--primary" disabled={walletBusy || !walletAddressRaw.trim()} onClick={() => void handleBindWallet()}>
                保存钱包账户
              </button>
            </div>
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeWalletModal} disabled={walletBusy}>取消</button>
            </div>
          </div>
        </div>
      )}

      <div className="card">
        <div className="card__title">公民资料</div>
        <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
          出生纸、证件照、档案等资料（待开发）
        </div>
      </div>
    </>
  );
}
