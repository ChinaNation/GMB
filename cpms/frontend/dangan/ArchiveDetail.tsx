// 公民档案详情页：公民信息、护照有效期、投票账户和 ARCHIVE 二维码操作。

import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import { listTowns, listVillages } from '../address/api';
import { installStatus } from '../initialize/api';
import * as api from './api';
import type { Archive, ArchiveMaterial, ArchiveMaterialType } from './types';
import type { Town, Village } from '../address/types';
import { parseQrEnvelope, type SignResponseBody } from '../qr/wuminQr';
import CameraQrScanner from '../qr/CameraQrScanner';
import { ScanIcon } from '../components/ScanIcon';
import DateInput, { isAtLeastAgeYmd, isPastYmd } from '../components/DateInput';

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

function formatYmdZh(value: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return '-';
  return `${match[1]}年${match[2]}月${match[3]}日`;
}

const materialTypeLabels: Record<ArchiveMaterialType, string> = {
  PHOTO: '照片',
  BIRTH_CERTIFICATE: '出生纸',
  COPY: '复印件',
  VIDEO: '视频',
  OTHER: '其他资料',
};

const archiveQrErrorLabels: Record<string, string> = {
  'archive qr requires last_name': '档案码生成条件未满足：姓氏',
  'archive qr requires first_name': '档案码生成条件未满足：名字',
  'archive qr requires gender': '档案码生成条件未满足：性别',
  'archive qr requires height': '档案码生成条件未满足：身高',
  'archive qr requires birth_date': '档案码生成条件未满足：出生日期',
  'archive qr requires passport_no': '档案码生成条件未满足：护照号',
  'archive qr requires valid_from': '档案码生成条件未满足：有效期',
  'archive qr requires valid_until': '档案码生成条件未满足：有效期',
  'archive qr requires province': '档案码生成条件未满足：省份',
  'archive qr requires city': '档案码生成条件未满足：城市',
  'archive qr requires normal citizen_status': '档案码生成条件未满足：公民状态必须为正常',
  'archive qr requires voting_eligible': '档案码生成条件未满足：选举资格必须为有',
  'archive qr requires age 16': '档案码生成条件未满足：公民必须年满16周岁',
  'archive qr requires wallet_address': '档案码生成条件未满足：投票账户',
  'archive qr requires wallet_pubkey': '档案码生成条件未满足：投票账户',
  'archive qr requires photo': '档案码生成条件未满足：照片至少1张',
  'archive qr requires birth_certificate': '档案码生成条件未满足：出生纸至少1张',
};

function archiveQrActionError(e: unknown, fallback: string): string {
  const message = e instanceof Error ? e.message : '';
  return archiveQrErrorLabels[message] ?? (message || fallback);
}

function formatFileSize(value: number): string {
  if (value >= 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB`;
  if (value >= 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${value} B`;
}

function isImageMaterial(item: ArchiveMaterial): boolean {
  return item.mime_type.startsWith('image/');
}

function isVideoMaterial(item: ArchiveMaterial): boolean {
  return item.mime_type.startsWith('video/');
}

export default function ArchiveDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [archive, setArchive] = useState<Archive | null>(null);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editForm, setEditForm] = useState<Record<string, unknown>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [walletModalOpen, setWalletModalOpen] = useState(false);
  const [walletScannerActive, setWalletScannerActive] = useState(false);
  const [walletScanError, setWalletScanError] = useState('');
  const [walletBusy, setWalletBusy] = useState(false);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [deleteChallenge, setDeleteChallenge] = useState<{ challenge_id: string; sign_request: string; expire_at: number } | null>(null);
  const [deleteScannerActive, setDeleteScannerActive] = useState(false);
  const [deleteScanError, setDeleteScanError] = useState('');
  const [deleteBusy, setDeleteBusy] = useState(false);
  // 名称解析
  const [provinceName, setProvinceName] = useState('');
  const [cityName, setCityName] = useState('');
  const [townName, setTownName] = useState('');
  const [villageName, setVillageName] = useState('');
  // 编辑用镇村列表
  const [towns, setTowns] = useState<Town[]>([]);
  const [villages, setVillages] = useState<Village[]>([]);
  const [materials, setMaterials] = useState<ArchiveMaterial[]>([]);
  const [materialType, setMaterialType] = useState<ArchiveMaterialType>('PHOTO');
  const [materialNote, setMaterialNote] = useState('');
  const [materialFile, setMaterialFile] = useState<File | null>(null);
  const [materialBusy, setMaterialBusy] = useState(false);
  const [materialError, setMaterialError] = useState('');
  const materialInputRef = useRef<HTMLInputElement | null>(null);

  const loadArchive = () => {
    if (!id) return;
    api.getArchive(id).then(res => { if (res.data) setArchive(res.data); }).finally(() => setLoading(false));
  };

  const loadMaterials = () => {
    if (!id) return;
    api.listArchiveMaterials(id)
      .then(res => { if (res.data) setMaterials(res.data.items); })
      .catch(e => setMaterialError(e instanceof Error ? e.message : '加载公民资料库失败'));
  };

  useEffect(() => {
    loadArchive();
    loadMaterials();
    installStatus().then(res => {
      if (res.data?.province_name) setProvinceName(res.data.province_name);
      if (res.data?.city_name) setCityName(res.data.city_name);
    }).catch(() => {});
    listTowns().then(res => { if (res.data) setTowns(res.data); }).catch(() => {});
  }, [id]);

  // 解析镇村名称
  useEffect(() => {
    if (!archive?.town_code) return;
    const t = towns.find(t => t.town_code === archive.town_code);
    if (t) setTownName(t.town_name);
    if (archive.village_id) {
      listVillages(archive.town_code).then(res => {
        if (res.data) {
          setVillages(res.data);
          const v = res.data.find(v => v.village_id === archive.village_id);
          if (v) setVillageName(v.village_name);
        }
      }).catch(() => {});
    }
  }, [archive?.town_code, archive?.village_id, towns]);

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
      listVillages(archive.town_code).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
    }
  };

  const handleEditTownChange = (tc: string) => {
    setEditForm(f => ({ ...f, town_code: tc, village_id: '' }));
    if (tc) {
      listVillages(tc).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
    } else {
      setVillages([]);
    }
  };

  const handleEditCitizenStatusChange = (value: string) => {
    setEditForm(f => ({
      ...f,
      citizen_status: value,
      voting_eligible: value === 'REVOKED' || !isAtLeastAgeYmd(String(f.birth_date || ''), 16) ? false : f.voting_eligible,
    }));
  };
  const handleEditBirthDateChange = (value: string) => {
    setEditForm(f => ({
      ...f,
      birth_date: value,
      voting_eligible: isAtLeastAgeYmd(value, 16) ? f.voting_eligible : false,
    }));
  };
  const canSetEditVotingEligible =
    editForm.citizen_status === 'NORMAL' && isAtLeastAgeYmd(String(editForm.birth_date || ''), 16);

  const handleSave = async () => {
    if (!id) return;
    setError('');
    const birthDate = String(editForm.birth_date || '');
    const heightText = String(editForm.height_cm ?? '');
    if (!String(editForm.last_name || '').trim()) { setError('请输入姓氏'); return; }
    if (!String(editForm.first_name || '').trim()) { setError('请输入名字'); return; }
    if (!isPastYmd(birthDate)) { setError('请选择正确的出生日期'); return; }
    if (editForm.voting_eligible === true && !isAtLeastAgeYmd(birthDate, 16)) { setError('未满16周岁的公民不能设置为有选举资格'); return; }
    if (!String(editForm.gender_code || '')) { setError('请选择性别'); return; }
    const height = Number(heightText);
    if (!Number.isFinite(height) || height < 30 || height > 260) { setError('请输入正确的身高'); return; }
    if (!String(editForm.town_code || '')) { setError('请选择镇'); return; }
    if (!String(editForm.village_id || '')) { setError('请选择村/路'); return; }
    if (!String(editForm.address || '').trim()) { setError('请输入详细地址'); return; }
    setSaving(true);
    try {
      const body: Record<string, unknown> = { ...editForm };
      body.last_name = String(body.last_name || '').trim();
      body.first_name = String(body.first_name || '').trim();
      body.address = String(body.address || '').trim();
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
    setWalletScannerActive(true);
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

  const handleWalletScanned = (raw: string) => {
    let walletAddress = '';
    try {
      walletAddress = extractWalletAddress(raw);
    } catch (e) {
      setWalletScanError(e instanceof Error ? e.message : '钱包二维码格式无效');
      return false;
    }
    void saveWalletAddress(walletAddress);
    return true;
  };

  const saveWalletAddress = async (walletAddress: string) => {
    if (!id) return;
    setError('');
    setWalletScanError('');
    setWalletBusy(true);
    try {
      const res = await api.bindArchiveWallet(id, walletAddress);
      if (res.data) setArchive(res.data);
      setWalletModalOpen(false);
      setWalletScannerActive(false);
    } catch (e) {
      const message = e instanceof Error ? e.message : '保存投票账户失败';
      setWalletScanError(message.includes('wallet already bound')
        ? '该钱包账户已绑定其他公民档案，不能重复绑定。'
        : message);
    } finally {
      setWalletBusy(false);
    }
  };

  const closeWalletModal = () => {
    setWalletModalOpen(false);
    setWalletScannerActive(false);
    setWalletScanError('');
  };

  const handleGenerateArchiveQr = async () => {
    if (!id) return;
    setError('');
    if (!ensureArchiveQrReady()) return;
    setWalletBusy(true);
    try {
      const res = await api.generateArchiveQr(id);
      if (res.data?.qr_content && archive) {
        setArchive({ ...archive, archive_qr_payload: res.data.qr_content });
      } else {
        loadArchive();
      }
    } catch (e) {
      setError(archiveQrActionError(e, '生成档案码失败'));
    } finally {
      setWalletBusy(false);
    }
  };

  const handlePrintArchiveQr = async () => {
    if (!id) return;
    setError('');
    if (!ensureArchiveQrReady()) return;
    setWalletBusy(true);
    try {
      await api.printArchiveQr(id);
      window.print();
    } catch (e) {
      setError(archiveQrActionError(e, '打印档案码失败'));
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
      setDeleteScannerActive(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建删除签名请求失败');
    } finally {
      setDeleteBusy(false);
    }
  };

  const handleDeleteReceiptScanned = (raw: string) => {
    if (!id || !deleteChallenge || deleteBusy) return false;
    try {
      const env = parseQrEnvelope(raw.trim());
      if (env.kind !== 'sign_response') {
        throw new Error('请扫描 wumin 返回的删除签名回执');
      }
      if (env.id !== deleteChallenge.challenge_id) {
        throw new Error('删除签名回执和当前请求不一致');
      }
      const body = env.body as SignResponseBody;
      void completeDeleteReceipt(body);
      return true;
    } catch (e) {
      setDeleteScanError(e instanceof Error ? e.message : '删除签名验证失败');
      return false;
    }
  };

  const completeDeleteReceipt = async (body: SignResponseBody) => {
    if (!id || !deleteChallenge) return;
    setDeleteBusy(true);
    setDeleteScanError('');
    try {
      await api.completeArchiveDelete(id, {
        challenge_id: deleteChallenge.challenge_id,
        pubkey: body.pubkey,
        sig_alg: body.sig_alg,
        signature: body.signature,
        payload_hash: body.payload_hash,
        signed_at: body.signed_at,
      });
      setDeleteModalOpen(false);
      setDeleteScannerActive(false);
      navigate('/admin');
    } catch (e) {
      setDeleteScanError(e instanceof Error ? e.message : '删除签名验证失败');
    } finally {
      setDeleteBusy(false);
    }
  };

  const closeDeleteModal = () => {
    setDeleteModalOpen(false);
    setDeleteScannerActive(false);
    setDeleteChallenge(null);
    setDeleteScanError('');
  };

  const handleMaterialUpload = async () => {
    if (!id || !materialFile) {
      setMaterialError('请选择要上传的公民资料');
      return;
    }
    setMaterialBusy(true);
    setMaterialError('');
    try {
      const body = new FormData();
      body.append('material_type', materialType);
      body.append('note', materialNote.trim());
      body.append('file', materialFile);
      const res = await api.uploadArchiveMaterial(id, body);
      const uploaded = res.data?.item;
      if (uploaded) setMaterials(items => [uploaded, ...items]);
      setMaterialFile(null);
      setMaterialNote('');
      if (materialInputRef.current) materialInputRef.current.value = '';
    } catch (e) {
      setMaterialError(e instanceof Error ? e.message : '上传公民资料失败');
    } finally {
      setMaterialBusy(false);
    }
  };

  const handleMaterialDelete = async (materialId: string) => {
    if (!id || materialBusy) return;
    setMaterialBusy(true);
    setMaterialError('');
    try {
      await api.deleteArchiveMaterial(id, materialId);
      setMaterials(items => items.filter(item => item.material_id !== materialId));
      setArchive(current => current ? { ...current, archive_qr_payload: '' } : current);
    } catch (e) {
      setMaterialError(e instanceof Error ? e.message : '删除公民资料失败');
    } finally {
      setMaterialBusy(false);
    }
  };

  const archiveQrMissingReasons = () => {
    if (!archive) return ['档案不存在'];
    const reasons: string[] = [];
    if (!archive.last_name.trim()) reasons.push('姓氏');
    if (!archive.first_name.trim()) reasons.push('名字');
    if (archive.gender_code !== 'M' && archive.gender_code !== 'W') reasons.push('性别');
    if (archive.height_cm === null || !Number.isFinite(Number(archive.height_cm))) reasons.push('身高');
    if (!isPastYmd(archive.birth_date)) reasons.push('出生日期');
    if (!archive.passport_no.trim()) reasons.push('护照号');
    if (!archive.valid_from.trim() || !archive.valid_until.trim()) reasons.push('有效期');
    if (!archive.province_code.trim()) reasons.push('省份');
    if (!archive.city_code.trim()) reasons.push('城市');
    if (archive.citizen_status !== 'NORMAL') reasons.push('公民状态必须为正常');
    if (!archive.voting_eligible) reasons.push('选举资格必须为有');
    if (archive.voting_eligible && !isAtLeastAgeYmd(archive.birth_date, 16)) reasons.push('年满16周岁');
    if (!archive.wallet_address?.trim() || !archive.wallet_pubkey?.trim()) reasons.push('投票账户');
    if (!materials.some(item => item.material_type === 'PHOTO')) reasons.push('照片至少1张');
    if (!materials.some(item => item.material_type === 'BIRTH_CERTIFICATE')) reasons.push('出生纸至少1张');
    return reasons;
  };

  const ensureArchiveQrReady = () => {
    const reasons = archiveQrMissingReasons();
    if (reasons.length === 0) return true;
    setError(`档案码生成条件未满足：${reasons.join('、')}`);
    return false;
  };

  if (loading) return <div className="card">加载中...</div>;
  if (!archive) return <div className="card">档案不存在</div>;
  const archiveDeleted = archive.status === 'DELETED' || archive.deleted_at !== null;
  const archiveId = id || archive.archive_id;

  return (
    <>
      <div className="card archive-detail-card print-area">
        <div className="card__title flex-between">
          公民档案详情
          <div className="no-print" style={{ display: 'flex', gap: 8 }}>
            {!archiveDeleted && !editing && <button className="btn btn--danger btn--sm" onClick={openDeleteModal} disabled={deleteBusy}>删除</button>}
            {!archiveDeleted && !editing && <button className="btn btn--primary btn--sm" onClick={startEdit}>编辑</button>}
            <button className="btn btn--ghost btn--sm" onClick={() => navigate('/admin')}>返回列表</button>
          </div>
        </div>
        {error && <div className="no-print" style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{error}</div>}
        {archiveDeleted && (
          <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>
            该档案已删除{archive.deleted_at ? `，删除时间：${new Date(archive.deleted_at * 1000).toLocaleString()}` : ''}{archive.deleted_by ? `，删除人：${archive.deleted_by}` : ''}
          </div>
        )}
        <div style={{ fontSize: 13, color: 'var(--color-text-secondary)', marginBottom: 16 }}>档案号：{archive.archive_no}</div>

        <div className="archive-detail-body">
          {/* 左侧：公民信息 */}
          <div className="archive-detail-main">
            {editing && !archiveDeleted ? (
              <>
                <div className="form-row">
                  <div className="form-group"><label>姓氏 *</label><input className="form-input" value={String(editForm.last_name || '')} onChange={e => setEditForm(f => ({ ...f, last_name: e.target.value }))} /></div>
                  <div className="form-group"><label>名字 *</label><input className="form-input" value={String(editForm.first_name || '')} onChange={e => setEditForm(f => ({ ...f, first_name: e.target.value }))} /></div>
                </div>
                <div className="form-row mt-16">
                  <div className="form-group"><label>出生日期 *</label><DateInput value={String(editForm.birth_date || '')} onChange={handleEditBirthDateChange} required /></div>
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
                    <label>镇 *</label>
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
                <div className="form-group mt-16"><label>详细地址 *</label><input className="form-input" maxLength={100} value={String(editForm.address || '')} onChange={e => setEditForm(f => ({ ...f, address: e.target.value }))} /></div>
                <div className="form-row mt-16">
                  <div className="form-group">
                    <label>公民状态 *</label>
                    <select className="form-input" value={String(editForm.citizen_status || 'NORMAL')} onChange={e => handleEditCitizenStatusChange(e.target.value)}>
                      <option value="NORMAL">正常</option><option value="REVOKED">注销</option>
                    </select>
                  </div>
                  <div className="form-group">
                    <label>选举资格 *</label>
                    <select
                      className="form-input"
                      value={String(canSetEditVotingEligible ? (editForm.voting_eligible ?? true) : false)}
                      onChange={e => setEditForm(f => ({ ...f, voting_eligible: e.target.value === 'true' }))}
                      disabled={!canSetEditVotingEligible}
                    >
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
                <div className="archive-detail-grid" style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 1fr)', columnGap: 32, rowGap: 16 }}>
                  <div><strong>姓氏：</strong>{archive.last_name || '-'}</div>
                  <div><strong>名字：</strong>{archive.first_name || '-'}</div>
                  <div><strong>性别：</strong>{archive.gender_code === 'M' ? '男' : '女'}</div>
                  <div><strong>身高：</strong>{archive.height_cm ? `${archive.height_cm} cm` : '-'}</div>
                  <div><strong>出生日期：</strong>{archive.birth_date}</div>
                  <div><strong>年龄：</strong>{calcAge(archive.birth_date)}</div>
                  <div><strong>护照号：</strong>{archive.passport_no || '-'}</div>
                  <div className="archive-validity-field">
                    <strong>有效期</strong>
                    <span className="archive-validity-lines">
                      <span className="archive-validity-row">
                        <span className="archive-validity-mark">：</span>
                        <span>{formatYmdZh(archive.valid_from)}</span>
                      </span>
                      <span className="archive-validity-row">
                        <span className="archive-validity-mark">-</span>
                        <span>{formatYmdZh(archive.valid_until)}</span>
                      </span>
                    </span>
                  </div>
                  <div><strong>省份：</strong>{provinceName || archive.province_code}</div>
                  <div><strong>城市：</strong>{cityName || archive.city_code}</div>
                  <div style={{ gridColumn: '1 / -1' }}><strong>详细地址：</strong>{[townName, villageName, archive.address].filter(Boolean).join(' ') || '-'}</div>
                </div>
                <div className="form-row mt-16">
                  <div><strong>公民状态：</strong>
                    <span className={`tag ${archive.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--danger'}`}>
                      {archive.citizen_status === 'NORMAL' ? '正常' : '注销'}
                    </span>
                  </div>
                  <div><strong>选举资格：</strong>
                    <span className={`tag ${archive.voting_eligible ? 'tag--success' : 'tag--warning'}`}>
                      {archive.voting_eligible ? '有选举资格' : '无选举资格'}
                    </span>
                  </div>
                </div>
                <div className="form-row mt-16">
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'nowrap', width: '100%', gridColumn: '1 / -1' }}>
                    <strong>投票账户：</strong>
                    {archive.wallet_address ? (
                      <>
                        <span className="archive-wallet-address">{archive.wallet_address}</span>
                        {!archiveDeleted && <button className="btn btn--primary btn--sm no-print" onClick={openWalletScanner} disabled={walletBusy}>更换</button>}
                      </>
                    ) : (
                      <>
                        <span className="print-only">未绑定</span>
                        <div className="no-print archive-wallet-bind-row">
                          <input
                            className="form-input"
                            value=""
                            placeholder="未绑定"
                            readOnly
                            style={{ flex: 1 }}
                          />
                          {!archiveDeleted && (
                            <button
                              className="btn btn--primary btn--sm no-print"
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
                      </>
                    )}
                  </div>
                </div>
              </>
            )}
          </div>

          {/* 右侧：ARCHIVE 二维码 */}
          <div className="archive-detail-qr">
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
              <div className="no-print" style={{ display: 'flex', gap: 8 }}>
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

      <div className="card archive-material-section">
        <div className="card__title flex-between">
          公民资料库
          <span className="archive-material-count">{materials.length}</span>
        </div>
        {materialError && (
          <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{materialError}</div>
        )}
        {!archiveDeleted && (
          <div className="archive-material-upload no-print">
            <div className="form-group">
              <label>资料类型</label>
              <select
                className="form-input"
                value={materialType}
                onChange={e => setMaterialType(e.target.value as ArchiveMaterialType)}
                disabled={materialBusy}
              >
                {Object.entries(materialTypeLabels).map(([value, label]) => (
                  <option key={value} value={value}>{label}</option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>文件</label>
              <input
                ref={materialInputRef}
                className="form-input"
                type="file"
                accept="image/jpeg,image/png,image/webp,application/pdf,video/mp4,video/quicktime,video/webm"
                onChange={e => setMaterialFile(e.target.files?.[0] ?? null)}
                disabled={materialBusy}
              />
            </div>
            <div className="form-group">
              <label>备注</label>
              <input
                className="form-input"
                maxLength={200}
                value={materialNote}
                onChange={e => setMaterialNote(e.target.value)}
                disabled={materialBusy}
              />
            </div>
            <button className="btn btn--primary" onClick={handleMaterialUpload} disabled={materialBusy || !materialFile}>
              {materialBusy ? '上传中...' : '上传'}
            </button>
          </div>
        )}
        {materials.length === 0 ? (
          <div className="archive-material-empty">暂无资料</div>
        ) : (
          <div className="archive-material-grid">
            {materials.map(item => {
              const url = api.archiveMaterialDownloadUrl(archiveId, item.material_id);
              return (
                <div className="archive-material-card" key={item.material_id}>
                  <div className="archive-material-preview">
                    {isImageMaterial(item) ? (
                      <img src={url} alt={item.original_file_name} />
                    ) : isVideoMaterial(item) ? (
                      <video src={url} controls />
                    ) : (
                      <div className="archive-material-file">{item.mime_type === 'application/pdf' ? 'PDF' : '文件'}</div>
                    )}
                  </div>
                  <div className="archive-material-meta">
                    <div className="archive-material-name" title={item.original_file_name}>
                      {item.original_file_name}
                    </div>
                    <div className="archive-material-sub">
                      {materialTypeLabels[item.material_type]} · {formatFileSize(item.file_size)}
                    </div>
                    <div className="archive-material-sub">
                      {new Date(item.uploaded_at * 1000).toLocaleString()}
                    </div>
                    {item.note && <div className="archive-material-note">{item.note}</div>}
                  </div>
                  <div className="archive-material-actions no-print">
                    <a className="btn btn--ghost btn--sm" href={url} download={item.original_file_name}>下载</a>
                    {!archiveDeleted && (
                      <button
                        className="btn btn--danger btn--sm"
                        onClick={() => void handleMaterialDelete(item.material_id)}
                        disabled={materialBusy}
                      >
                        删除
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {walletModalOpen && (
        <div className="modal-overlay">
          <div className="modal" style={{ width: 340, minWidth: 340, maxWidth: 340 }}>
            <div className="modal__title">扫描钱包二维码</div>
            {walletScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{walletScanError}</div>}
            <CameraQrScanner
              active={walletScannerActive}
              onActiveChange={setWalletScannerActive}
              onDetected={handleWalletScanned}
              onError={setWalletScanError}
              size={292}
              busy={walletBusy}
              loadingText="摄像头初始化中..."
            />
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeWalletModal} disabled={walletBusy}>取消</button>
            </div>
          </div>
        </div>
      )}

      {deleteModalOpen && deleteChallenge && (
        <div className="modal-overlay">
          <div className="modal" style={{ width: 'min(680px, calc(100vw - 32px))', minWidth: 0, maxWidth: 680 }}>
            <div className="modal__title">删除档案签名确认</div>
            {deleteScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12, textAlign: 'center' }}>{deleteScanError}</div>}
            <div style={{ display: 'flex', gap: 24, alignItems: 'stretch', flexWrap: 'wrap' }}>
              <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
                <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>删除签名二维码</div>
                <div style={{
                  width: 260, height: 260,
                  background: '#f8fffe',
                  borderRadius: 16,
                  border: '2px solid #e6f7f5',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  overflow: 'hidden',
                }}>
                  <QRCodeSVG value={deleteChallenge.sign_request} size={228} fgColor="#134e4a" />
                </div>
                <div style={{ marginTop: 10, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
                  有效期至 {new Date(deleteChallenge.expire_at * 1000).toLocaleTimeString()}
                </div>
                <div style={{ marginTop: 6, textAlign: 'center', fontSize: 12, color: 'var(--color-text-secondary)' }}>
                  当前登录管理员使用 wumin 扫码签名
                </div>
              </div>

              <div style={{
                width: 1,
                background: 'linear-gradient(to bottom, transparent, var(--color-border), transparent)',
                alignSelf: 'stretch',
              }} />

              <div style={{ flex: '1 1 260px', minWidth: 240, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
                <div style={{ fontSize: 14, fontWeight: 500, color: 'var(--color-text)', marginBottom: 12 }}>扫码窗口</div>
                <CameraQrScanner
                  active={deleteScannerActive}
                  onActiveChange={setDeleteScannerActive}
                  onDetected={handleDeleteReceiptScanned}
                  onError={setDeleteScanError}
                  hint="扫描 wumin 返回的删除签名回执"
                  busy={deleteBusy}
                  loadingText="摄像头初始化中..."
                />
              </div>
            </div>
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeDeleteModal} disabled={deleteBusy}>取消</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
