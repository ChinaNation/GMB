// 公民档案详情页。左侧公民信息（可编辑），右侧 ARCHIVE 二维码（自动生成，仅下载）。
// 下方公民资料区域预留给出生纸、证件照等上传。

import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { Archive } from '../types';
import { parseQrEnvelope, type SignResponseBody } from '../qr/wuminQr';
import { startCameraScanner } from '../utils/cameraScanner';
import { ScanIcon } from '../components/ScanIcon';

function calcAge(birthDate: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(birthDate)) return '-';
  const birth = new Date(`${birthDate}T00:00:00`);
  if (Number.isNaN(birth.getTime())) return '-';
  const todayDate = new Date();
  let age = todayDate.getFullYear() - birth.getFullYear();
  const passedBirthday = todayDate.getMonth() > birth.getMonth()
    || (todayDate.getMonth() === birth.getMonth() && todayDate.getDate() >= birth.getDate());
  if (!passedBirthday) age -= 1;
  return age >= 0 ? `${age}岁` : '-';
}

export default function ArchiveDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const today = new Date().toISOString().slice(0, 10);
  const [archive, setArchive] = useState<Archive | null>(null);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editForm, setEditForm] = useState<Record<string, unknown>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [walletModalOpen, setWalletModalOpen] = useState(false);
  const [walletScanError, setWalletScanError] = useState('');
  const [walletBusy, setWalletBusy] = useState(false);
  const walletVideoRef = useRef<HTMLVideoElement | null>(null);
  const walletScanCleanupRef = useRef<(() => void) | null>(null);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [deleteChallenge, setDeleteChallenge] = useState<{ challenge_id: string; sign_request: string; expire_at: number } | null>(null);
  const [deleteScanError, setDeleteScanError] = useState('');
  const [deleteBusy, setDeleteBusy] = useState(false);
  const deleteVideoRef = useRef<HTMLVideoElement | null>(null);
  const deleteScanCleanupRef = useRef<(() => void) | null>(null);
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

  useEffect(() => {
    if (!deleteModalOpen || !deleteVideoRef.current || !deleteChallenge) return;
    setDeleteScanError('');
    deleteScanCleanupRef.current?.();
    deleteScanCleanupRef.current = startCameraScanner(
      deleteVideoRef.current,
      (raw) => {
        void handleDeleteReceipt(raw);
      },
      () => setDeleteScanError(''),
      (msg) => setDeleteScanError(msg),
    );
    return () => {
      deleteScanCleanupRef.current?.();
      deleteScanCleanupRef.current = null;
    };
  }, [deleteModalOpen, deleteChallenge?.challenge_id]);

  const startEdit = () => {
    if (!archive) return;
    setEditForm({
      last_name: archive.last_name,
      first_name: archive.first_name,
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
    const birthDate = String(editForm.birth_date || '');
    const heightText = String(editForm.height_cm ?? '');
    if (!String(editForm.last_name || '').trim()) { setError('请输入姓氏'); return; }
    if (!String(editForm.first_name || '').trim()) { setError('请输入名字'); return; }
    if (!/^\d{4}-\d{2}-\d{2}$/.test(birthDate) || birthDate > today) { setError('请选择正确的出生日期'); return; }
    if (!String(editForm.gender_code || '')) { setError('请选择性别'); return; }
    const height = Number(heightText);
    if (!Number.isFinite(height) || height < 30 || height > 260) { setError('请输入正确的身高'); return; }
    if (!String(editForm.town_code || '')) { setError('请选择镇/街道'); return; }
    if (!String(editForm.village_id || '')) { setError('请选择村/路'); return; }
    setSaving(true);
    try {
      const body: Record<string, unknown> = { ...editForm };
      body.last_name = String(body.last_name || '').trim();
      body.first_name = String(body.first_name || '').trim();
      body.height_cm = height;
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
    setWalletModalOpen(true);
  };

  const extractWalletAddress = (raw: string) => {
    const text = raw.trim();
    try {
      const env = parseQrEnvelope(text);
      if (env.kind !== 'user_contact') {
        throw new Error('请扫描 wumin 的钱包地址二维码');
      }
      const body = env.body as { address: string };
      return body.address.trim();
    } catch (e) {
      if (text.startsWith('{')) throw e;
      return text;
    }
  };

  const handleBindWallet = async (raw: string) => {
    if (!id) return;
    setError('');
    setWalletScanError('');
    setWalletBusy(true);
    try {
      const walletAddress = extractWalletAddress(raw);
      const res = await api.bindArchiveWallet(id, walletAddress);
      if (res.data) setArchive(res.data);
      walletScanCleanupRef.current?.();
      walletScanCleanupRef.current = null;
      setWalletModalOpen(false);
    } catch (e) {
      setWalletScanError(e instanceof Error ? e.message : '保存投票账户失败');
    } finally {
      setWalletBusy(false);
    }
  };

  const closeWalletModal = () => {
    walletScanCleanupRef.current?.();
    walletScanCleanupRef.current = null;
    setWalletModalOpen(false);
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

  const handlePrintArchiveQr = async () => {
    if (!id) return;
    setError('');
    setWalletBusy(true);
    try {
      await api.printArchiveQr(id);
      window.print();
    } catch (e) {
      setError(e instanceof Error ? e.message : '打印档案码失败');
    } finally {
      setWalletBusy(false);
    }
  };

  const openDeleteModal = async () => {
    if (!id) return;
    setError('');
    setDeleteScanError('');
    setDeleteBusy(true);
    try {
      const res = await api.createArchiveDeleteChallenge(id);
      if (!res.data) throw new Error('创建删除签名请求失败');
      setDeleteChallenge(res.data);
      setDeleteModalOpen(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建删除签名请求失败');
    } finally {
      setDeleteBusy(false);
    }
  };

  const handleDeleteReceipt = async (raw: string) => {
    if (!id || !deleteChallenge || deleteBusy) return;
    setDeleteBusy(true);
    setDeleteScanError('');
    try {
      const env = parseQrEnvelope(raw.trim());
      if (env.kind !== 'sign_response') {
        throw new Error('请扫描 wumin 返回的删除签名回执');
      }
      if (env.id !== deleteChallenge.challenge_id) {
        throw new Error('删除签名回执和当前请求不一致');
      }
      const body = env.body as SignResponseBody;
      await api.completeArchiveDelete(id, {
        challenge_id: deleteChallenge.challenge_id,
        pubkey: body.pubkey,
        sig_alg: body.sig_alg,
        signature: body.signature,
        payload_hash: body.payload_hash,
        signed_at: body.signed_at,
      });
      deleteScanCleanupRef.current?.();
      deleteScanCleanupRef.current = null;
      setDeleteModalOpen(false);
      navigate('/admin');
    } catch (e) {
      setDeleteScanError(e instanceof Error ? e.message : '删除签名验证失败');
    } finally {
      setDeleteBusy(false);
    }
  };

  const closeDeleteModal = () => {
    deleteScanCleanupRef.current?.();
    deleteScanCleanupRef.current = null;
    setDeleteModalOpen(false);
    setDeleteChallenge(null);
    setDeleteScanError('');
  };

  if (loading) return <div className="card">加载中...</div>;
  if (!archive) return <div className="card">档案不存在</div>;
  const archiveDeleted = archive.status === 'DELETED' || archive.deleted_at !== null;

  return (
    <>
      <div className="card">
        <div className="card__title flex-between">
          公民档案详情
          <div style={{ display: 'flex', gap: 8 }}>
            {!archiveDeleted && !editing && <button className="btn btn--danger btn--sm" onClick={openDeleteModal} disabled={deleteBusy}>删除</button>}
            {!archiveDeleted && !editing && <button className="btn btn--primary btn--sm" onClick={startEdit}>编辑</button>}
            <button className="btn btn--ghost btn--sm" onClick={() => navigate('/admin')}>返回列表</button>
          </div>
        </div>
        {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{error}</div>}
        {archiveDeleted && (
          <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>
            该档案已删除{archive.deleted_at ? `，删除时间：${new Date(archive.deleted_at * 1000).toLocaleString()}` : ''}{archive.deleted_by ? `，删除人：${archive.deleted_by}` : ''}
          </div>
        )}
        <div style={{ fontSize: 13, color: 'var(--color-text-secondary)', marginBottom: 16 }}>档案号：{archive.archive_no}</div>

        <div style={{ display: 'flex', gap: 32, alignItems: 'flex-start' }}>
          {/* 左侧：公民信息 */}
          <div style={{ flex: 1 }}>
            {editing && !archiveDeleted ? (
              <>
                <div className="form-row">
                  <div className="form-group"><label>姓氏 *</label><input className="form-input" value={String(editForm.last_name || '')} onChange={e => setEditForm(f => ({ ...f, last_name: e.target.value }))} /></div>
                  <div className="form-group"><label>名字 *</label><input className="form-input" value={String(editForm.first_name || '')} onChange={e => setEditForm(f => ({ ...f, first_name: e.target.value }))} /></div>
                </div>
                <div className="form-row mt-16">
                  <div className="form-group"><label>出生日期 *</label><input className="form-input" type="date" max={today} value={String(editForm.birth_date || '')} onChange={e => setEditForm(f => ({ ...f, birth_date: e.target.value }))} /></div>
                  <div className="form-group">
                    <label>性别 *</label>
                    <select className="form-input" value={String(editForm.gender_code || 'M')} onChange={e => setEditForm(f => ({ ...f, gender_code: e.target.value }))}>
                      <option value="M">男</option><option value="W">女</option>
                    </select>
                  </div>
                  <div className="form-group"><label>身高 (cm) *</label><input className="form-input" type="number" min={30} max={260} step="0.1" value={String(editForm.height_cm ?? '')} onChange={e => setEditForm(f => ({ ...f, height_cm: e.target.value }))} /></div>
                </div>
                <div className="form-row mt-16">
                  <div className="form-group">
                    <label>镇/街道 *</label>
                    <select className="form-input" value={String(editForm.town_code || '')} onChange={e => handleEditTownChange(e.target.value)}>
                      <option value="">请选择</option>
                      {towns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
                    </select>
                  </div>
                  <div className="form-group">
                    <label>村/路 *</label>
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
                <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 1fr)', columnGap: 32, rowGap: 16 }}>
                  <div><strong>姓氏：</strong>{archive.last_name || '-'}</div>
                  <div><strong>名字：</strong>{archive.first_name || '-'}</div>
                  <div><strong>性别：</strong>{archive.gender_code === 'M' ? '男' : '女'}</div>
                  <div><strong>身高：</strong>{archive.height_cm ? `${archive.height_cm} cm` : '-'}</div>
                  <div><strong>出生日期：</strong>{archive.birth_date}</div>
                  <div><strong>年龄：</strong>{calcAge(archive.birth_date)}</div>
                  <div><strong>省份：</strong>{provinceName || archive.province_code}</div>
                  <div><strong>城市：</strong>{cityName || archive.city_code}</div>
                  <div style={{ gridColumn: '1 / -1' }}><strong>地址：</strong>{[townName, villageName, archive.address].filter(Boolean).join(' ') || '-'}</div>
                </div>
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
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap', width: '100%' }}>
                    <strong>投票账户：</strong>
                    {archive.wallet_address ? (
                      <>
                        <span style={{ fontFamily: 'monospace', overflowWrap: 'anywhere' }}>{archive.wallet_address}</span>
                        {!archiveDeleted && <button className="btn btn--primary btn--sm" onClick={openWalletScanner} disabled={walletBusy}>更换</button>}
                      </>
                    ) : (
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6, minWidth: 520, flex: '1 1 640px', maxWidth: 960 }}>
                        <input
                          className="form-input"
                          value=""
                          placeholder="未绑定"
                          readOnly
                          style={{ flex: 1 }}
                        />
                        {!archiveDeleted && (
                          <button
                            className="btn btn--primary btn--sm"
                            onClick={openWalletScanner}
                            disabled={walletBusy}
                            title="扫描钱包二维码"
                            aria-label="扫描钱包二维码"
                            style={{ width: 36, height: 36, padding: 0, display: 'inline-flex', alignItems: 'center', justifyContent: 'center' }}
                          >
                            <ScanIcon size={18} />
                          </button>
                        )}
                      </div>
                    )}
                  </div>
                </div>
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
              </>
            ) : (
              <div style={{ width: 200, height: 200, background: '#f3f4f6', borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--color-text-secondary)', fontSize: 13 }}>
                签发未就绪
              </div>
            )}
            {!archiveDeleted && (
              <div style={{ display: 'flex', gap: 8 }}>
                <button className="btn btn--primary btn--sm" onClick={handleGenerateArchiveQr} disabled={walletBusy || !archive.wallet_pubkey}>
                  更新
                </button>
                <button className="btn btn--ghost btn--sm" onClick={handleDownloadArchiveQr} disabled={!archive.archive_qr_payload}>
                  下载
                </button>
                <button className="btn btn--ghost btn--sm" onClick={handlePrintArchiveQr} disabled={walletBusy || !archive.archive_qr_payload}>
                  打印
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {walletModalOpen && (
        <div className="modal-overlay">
          <div className="modal" style={{ width: 340, minWidth: 340, maxWidth: 340 }}>
            <div className="modal__title">扫描钱包二维码</div>
            {walletScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{walletScanError}</div>}
            <video
              ref={walletVideoRef}
              muted
              playsInline
              style={{ width: 292, height: 292, display: 'block', objectFit: 'cover', borderRadius: 8, background: '#111827' }}
            />
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeWalletModal} disabled={walletBusy}>取消</button>
            </div>
          </div>
        </div>
      )}

      {deleteModalOpen && deleteChallenge && (
        <div className="modal-overlay">
          <div className="modal" style={{ width: 620, minWidth: 620, maxWidth: 620 }}>
            <div className="modal__title">删除档案签名</div>
            {deleteScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{deleteScanError}</div>}
            <div style={{ display: 'grid', gridTemplateColumns: '260px 1fr', gap: 18, alignItems: 'start' }}>
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8 }}>
                <QRCodeSVG value={deleteChallenge.sign_request} size={240} />
                <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', textAlign: 'center' }}>
                  请使用 wumin 扫码确认删除
                </div>
              </div>
              <div>
                <video
                  ref={deleteVideoRef}
                  muted
                  playsInline
                  style={{ width: 292, height: 292, display: 'block', objectFit: 'cover', borderRadius: 8, background: '#111827' }}
                />
              </div>
            </div>
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeDeleteModal} disabled={deleteBusy}>取消</button>
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
