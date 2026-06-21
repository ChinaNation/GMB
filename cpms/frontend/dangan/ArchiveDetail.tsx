// 公民档案详情页：左侧导航切换档案详情、资料库、操作记录，右侧承载当前业务区域。

import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import { listAddressUnits, listBirthCities, listBirthProvinces, listBirthTowns, listTowns } from '../address/api';
import { installStatus } from '../initialize/api';
import * as api from './api';
import type { Archive, ArchiveAuditLog, ArchiveMaterial, ArchiveMaterialType, ElectionScopeLevel } from './types';
import type { AddressUnit, Town } from '../address/types';
import { parseQrEnvelope, type SignResponseBody } from '../qr/citizenQr';
import CameraQrScanner from '../qr/CameraQrScanner';
import { ScanIcon } from '../components/ScanIcon';
import { isAtLeastAgeYmd, isPastYmd } from '../components/DateInput';

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

type ArchiveDetailTab = 'detail' | 'materials' | 'operations';

const archiveDetailTabs: Array<{ key: ArchiveDetailTab; label: string; icon: 'home' | 'folder' | 'history' }> = [
  { key: 'detail', label: '档案详情', icon: 'home' },
  { key: 'materials', label: '资料库', icon: 'folder' },
  { key: 'operations', label: '操作记录', icon: 'history' },
];

function ArchiveDetailNavIcon({ type }: { type: 'home' | 'folder' | 'history' }) {
  if (type === 'home') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6h-4v6H5a1 1 0 0 1-1-1v-9.5Z" />
      </svg>
    );
  }
  if (type === 'folder') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <path d="M3 8a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v2" />
        <path d="M3 11h18l-2 7a2 2 0 0 1-2 1H5a2 2 0 0 1-2-2v-6Z" />
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M3 12a9 9 0 1 0 3-6.7" />
      <path d="M3 5v5h5" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}

const auditActionLabels: Record<string, string> = {
  CREATE_ARCHIVE: '创建档案',
  UPDATE_ARCHIVE: '编辑档案',
  BIND_ARCHIVE_WALLET: '绑定投票账户',
  GENERATE_ARCHIVE_QR: '更新档案码',
  PRINT_ARCHIVE_QR: '打印档案码',
  ARCHIVE_MATERIAL_UPLOAD: '上传资料',
  ARCHIVE_MATERIAL_DOWNLOAD: '下载资料',
  ARCHIVE_MATERIAL_DELETE: '删除资料',
  ARCHIVE_DELETE_COMPLETE: '删除档案',
  ARCHIVE_DELETE_FAILED: '删除档案失败',
  UPDATE_ARCHIVE_CITIZEN_STATUS: '修改公民状态',
};

const auditDetailLabels: Record<string, string> = {
  archive_id: '档案ID',
  archive_no: '档案号',
  citizen_status: '公民状态',
  voting_eligible: '选举资格',
  wallet_address: '投票账户',
  wallet_pubkey: '投票账户公钥',
  election_scope_level: '选举注册范围',
  material_type: '资料类型',
  mime_type: '文件类型',
  file_size: '文件大小',
  sha256: 'SHA-256',
  valid_from: '有效期开始',
  valid_until: '有效期截止',
  reason: '原因',
};

function auditDetailText(detail: Record<string, unknown>): string {
  const parts: string[] = [];
  for (const [key, raw] of Object.entries(detail)) {
    if (raw === null || raw === undefined || raw === '') continue;
    let value = String(raw);
    if (typeof raw === 'boolean') value = raw ? '是' : '否';
    if (typeof raw === 'object') value = JSON.stringify(raw);
    if (key === 'material_type') value = materialTypeLabels[value as ArchiveMaterialType] ?? value;
    if (key === 'citizen_status') value = value === 'NORMAL' ? '正常' : value === 'REVOKED' ? '注销' : value;
    parts.push(`${auditDetailLabels[key] ?? key}：${value}`);
  }
  return parts.join('；');
}

function auditDetailWithResult(log: ArchiveAuditLog): string {
  const detail = auditDetailText(log.detail);
  const result = `结果：${log.result === 'SUCCESS' ? '成功' : '失败'}`;
  return detail ? `${result}；${detail}` : result;
}

const archiveQrErrorLabels: Record<string, string> = {
  'archive qr requires last_name': '档案码生成条件未满足：姓氏',
  'archive qr requires first_name': '档案码生成条件未满足：名字',
  'archive qr requires gender': '档案码生成条件未满足：性别',
  'archive qr requires height': '档案码生成条件未满足：身高',
  'archive qr requires birth_date': '档案码生成条件未满足：出生日期',
  'archive qr requires passport_no': '档案码生成条件未满足：护照号',
  'archive qr requires valid_from': '档案码生成条件未满足：有效期',
  'archive qr requires valid_until': '档案码生成条件未满足：有效期',
  'archive qr requires province': '档案码生成条件未满足：居住省份',
  'archive qr requires city': '档案码生成条件未满足：居住城市',
  'archive qr requires birth province': '档案码生成条件未满足：出生省份',
  'archive qr requires birth city': '档案码生成条件未满足：出生城市',
  'archive qr requires birth town': '档案码生成条件未满足：出生镇',
  'archive qr requires residence city': '档案码生成条件未满足：居住城市',
  'archive qr requires residence town': '档案码生成条件未满足：居住镇',
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
  const [walletDraftAddress, setWalletDraftAddress] = useState('');
  const [walletElectionCityChecked, setWalletElectionCityChecked] = useState(false);
  const [walletElectionTownChecked, setWalletElectionTownChecked] = useState(false);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [deleteChallenge, setDeleteChallenge] = useState<{ challenge_id: string; sign_request: string; expire_at: number } | null>(null);
  const [deleteScannerActive, setDeleteScannerActive] = useState(false);
  const [deleteScanError, setDeleteScanError] = useState('');
  const [deleteBusy, setDeleteBusy] = useState(false);
  // 名称解析
  const [provinceName, setProvinceName] = useState('');
  const [cityName, setCityName] = useState('');
  const [townName, setTownName] = useState('');
  const [addressUnitName, setAddressUnitName] = useState('');
  const [birthProvinceName, setBirthProvinceName] = useState('');
  const [birthCityName, setBirthCityName] = useState('');
  const [birthTownName, setBirthTownName] = useState('');
  // 编辑用镇和地址段列表
  const [towns, setTowns] = useState<Town[]>([]);
  const [addressUnits, setAddressUnits] = useState<AddressUnit[]>([]);
  const [materials, setMaterials] = useState<ArchiveMaterial[]>([]);
  const [materialType, setMaterialType] = useState<ArchiveMaterialType>('PHOTO');
  const [materialNote, setMaterialNote] = useState('');
  const [materialFile, setMaterialFile] = useState<File | null>(null);
  const [materialBusy, setMaterialBusy] = useState(false);
  const [materialError, setMaterialError] = useState('');
  const [materialModalOpen, setMaterialModalOpen] = useState(false);
  const materialInputRef = useRef<HTMLInputElement | null>(null);
  const [activeTab, setActiveTab] = useState<ArchiveDetailTab>('detail');
  const [auditLogs, setAuditLogs] = useState<ArchiveAuditLog[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditError, setAuditError] = useState('');

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

  const loadAuditLogs = () => {
    if (!id) return;
    setAuditLoading(true);
    setAuditError('');
    api.listArchiveAuditLogs(id)
      .then(res => setAuditLogs(res.data?.items || []))
      .catch(e => setAuditError(e instanceof Error ? e.message : '加载操作记录失败'))
      .finally(() => setAuditLoading(false));
  };

  useEffect(() => {
    loadArchive();
    loadMaterials();
    loadAuditLogs();
    installStatus().then(res => {
      if (res.data?.province_name) setProvinceName(res.data.province_name);
      if (res.data?.city_name) setCityName(res.data.city_name);
    }).catch(() => {});
    listTowns().then(res => { if (res.data) setTowns(res.data); }).catch(() => {});
  }, [id]);

  // 解析镇和地址段名称
  useEffect(() => {
    if (!archive?.town_code) return;
    const t = towns.find(t => t.town_code === archive.town_code);
    if (t) setTownName(t.town_name);
    if (archive.address_unit_id) {
      listAddressUnits(archive.town_code).then(res => {
        if (res.data) {
          setAddressUnits(res.data);
          const v = res.data.find(v => v.address_unit_id === archive.address_unit_id);
          if (v) setAddressUnitName(v.address_unit_name);
        }
      }).catch(() => {});
    }
  }, [archive?.town_code, archive?.address_unit_id, towns]);

  useEffect(() => {
    if (!archive) return;
    let active = true;
    setBirthProvinceName('');
    setBirthCityName('');
    setBirthTownName('');
    if (!archive.birth_province_code || !archive.birth_city_code || !archive.birth_town_code) return;

    listBirthProvinces().then(res => {
      if (!active || !res.data) return;
      const item = res.data.find(p => p.province_code === archive.birth_province_code);
      if (item) setBirthProvinceName(item.province_name);
    }).catch(() => {});
    listBirthCities(archive.birth_province_code).then(res => {
      if (!active || !res.data) return;
      const item = res.data.find(c => c.city_code === archive.birth_city_code);
      if (item) setBirthCityName(item.city_name);
    }).catch(() => {});
    listBirthTowns(archive.birth_province_code, archive.birth_city_code).then(res => {
      if (!active || !res.data) return;
      const item = res.data.find(t => t.town_code === archive.birth_town_code);
      if (item) setBirthTownName(item.town_name);
    }).catch(() => {});

    return () => { active = false; };
  }, [archive?.birth_province_code, archive?.birth_city_code, archive?.birth_town_code]);

  const startEdit = () => {
    if (!archive) return;
    setEditForm({
      last_name: archive.last_name,
      first_name: archive.first_name,
      gender_code: archive.gender_code,
      height_cm: archive.height_cm ?? '',
      town_code: archive.town_code,
      address_unit_id: archive.address_unit_id,
      address_detail: archive.address_detail,
      citizen_status: archive.citizen_status,
      voting_eligible: archive.voting_eligible,
    });
    setEditing(true);
    setError('');
    // 加载编辑用地址段列表
    if (archive.town_code) {
      listAddressUnits(archive.town_code).then(res => { if (res.data) setAddressUnits(res.data); }).catch(() => {});
    }
  };

  const handleEditTownChange = (tc: string) => {
    setEditForm(f => ({ ...f, town_code: tc, address_unit_id: '' }));
    if (tc) {
      listAddressUnits(tc).then(res => { if (res.data) setAddressUnits(res.data); }).catch(() => {});
    } else {
      setAddressUnits([]);
    }
  };

  const handleEditCitizenStatusChange = (value: string) => {
    setEditForm(f => ({
      ...f,
      citizen_status: value,
      voting_eligible: value === 'REVOKED' || !isAtLeastAgeYmd(archive?.birth_date || '', 16) ? false : f.voting_eligible,
    }));
  };
  const canSetEditVotingEligible =
    editForm.citizen_status === 'NORMAL' && isAtLeastAgeYmd(archive?.birth_date || '', 16);

  const handleSave = async () => {
    if (!id) return;
    setError('');
    const heightText = String(editForm.height_cm ?? '');
    if (!String(editForm.last_name || '').trim()) { setError('请输入姓氏'); return; }
    if (!String(editForm.first_name || '').trim()) { setError('请输入名字'); return; }
    if (editForm.voting_eligible === true && !isAtLeastAgeYmd(archive?.birth_date || '', 16)) { setError('未满16周岁的公民不能设置为有选举资格'); return; }
    if (!String(editForm.gender_code || '')) { setError('请选择性别'); return; }
    const height = Number(heightText);
    if (!Number.isFinite(height) || height < 30 || height > 260) { setError('请输入正确的身高'); return; }
    if (!String(editForm.town_code || '')) { setError('请选择居住镇'); return; }
    if (!String(editForm.address_unit_id || '')) { setError('请选择地址段'); return; }
    if (!String(editForm.address_detail || '').trim()) { setError('请输入详细地址'); return; }
    setSaving(true);
    try {
      const body: Record<string, unknown> = { ...editForm };
      body.last_name = String(body.last_name || '').trim();
      body.first_name = String(body.first_name || '').trim();
      body.address_detail = String(body.address_detail || '').trim();
      body.height_cm = height;
      delete body.birth_date;
      const res = await api.updateArchive(id, body);
      if (res.data) setArchive(res.data);
      setEditing(false);
      loadAuditLogs();
    } catch (e) {
      setError(e instanceof Error ? e.message : '保存失败');
    }
    setSaving(false);
  };

  const electionScopeFromFlags = (cityChecked: boolean, townChecked: boolean): ElectionScopeLevel => {
    if (townChecked) return 'TOWN';
    if (cityChecked) return 'CITY';
    return 'PROVINCE';
  };

  const handleWalletElectionCityChange = (checked: boolean) => {
    setWalletElectionCityChecked(checked);
    if (!checked) setWalletElectionTownChecked(false);
  };

  const handleWalletElectionTownChange = (checked: boolean) => {
    setWalletElectionTownChecked(checked);
    if (checked) setWalletElectionCityChecked(true);
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

  const openWalletSettings = () => {
    if (!archive?.voting_eligible) {
      setError('无选举资格的公民不能设置投票账户');
      return;
    }
    const cityRegistered = archive?.election_scope_level === 'CITY' || archive?.election_scope_level === 'TOWN';
    const townRegistered = archive?.election_scope_level === 'TOWN';
    setError('');
    setWalletScanError('');
    setWalletDraftAddress(archive?.wallet_address || '');
    setWalletElectionCityChecked(cityRegistered);
    setWalletElectionTownChecked(townRegistered);
    setWalletModalOpen(true);
    setWalletScannerActive(!archive?.wallet_address);
  };

  const extractWalletAddress = (raw: string) => {
    const text = raw.trim();
    try {
      const env = parseQrEnvelope(text);
      if (env.kind !== 'user_contact') {
        throw new Error('请扫描 citizenwallet 的钱包地址二维码');
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
    setWalletDraftAddress(walletAddress);
    setWalletScanError('');
    setWalletScannerActive(false);
    return true;
  };

  const saveWalletSettings = async () => {
    if (!id) return;
    setError('');
    setWalletScanError('');
    if (!archive?.voting_eligible) {
      setWalletScanError('无选举资格的公民不能设置投票账户');
      return;
    }
    const walletAddress = walletDraftAddress.trim();
    if (!walletAddress) {
      setWalletScanError('请扫描或填写投票账户');
      return;
    }
    setWalletBusy(true);
    try {
      const res = await api.bindArchiveWallet(id, {
        wallet_address: walletAddress,
        election_scope_level: electionScopeFromFlags(walletElectionCityChecked, walletElectionTownChecked),
      });
      if (res.data) setArchive(res.data);
      setWalletModalOpen(false);
      setWalletScannerActive(false);
      loadAuditLogs();
    } catch (e) {
      const message = e instanceof Error ? e.message : '保存投票账户失败';
      setWalletScanError(message.includes('wallet already bound')
        ? '该钱包账户已绑定其他公民档案，不能重复绑定。'
        : message.includes('archive voting ineligible')
          ? '无选举资格的公民不能设置投票账户。'
        : message);
    } finally {
      setWalletBusy(false);
    }
  };

  const closeWalletModal = () => {
    if (walletBusy) return;
    setWalletModalOpen(false);
    setWalletScannerActive(false);
    setWalletScanError('');
    setWalletDraftAddress('');
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
      loadAuditLogs();
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
      loadAuditLogs();
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
        throw new Error('请扫描 citizenwallet 返回的删除签名回执');
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

  const openMaterialModal = () => {
    setMaterialError('');
    setMaterialModalOpen(true);
  };

  const closeMaterialModal = () => {
    if (materialBusy) return;
    setMaterialModalOpen(false);
    setMaterialFile(null);
    setMaterialNote('');
    setMaterialError('');
    if (materialInputRef.current) materialInputRef.current.value = '';
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
      loadAuditLogs();
      setMaterialFile(null);
      setMaterialNote('');
      setMaterialModalOpen(false);
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
      loadAuditLogs();
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
    if (!archive.province_code.trim()) reasons.push('居住省份');
    if (!archive.city_code.trim()) reasons.push('居住城市');
    if (!archive.birth_province_code.trim()) reasons.push('出生省份');
    if (!archive.birth_city_code.trim()) reasons.push('出生城市');
    if (!archive.birth_town_code.trim()) reasons.push('出生镇');
    if (archive.election_scope_level === 'TOWN' && !archive.town_code.trim()) reasons.push('居住镇');
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
  const canSetWallet = !archiveDeleted && archive.voting_eligible;
  const walletActionTitle = archive.voting_eligible ? '设置投票账户' : '无选举资格，不能设置投票账户';

  const archiveTitle = `${archive.last_name || ''}${archive.first_name || ''}`.trim() || '公民档案';
  const birthplaceText = [
    birthProvinceName || archive.birth_province_code,
    birthCityName || archive.birth_city_code,
    birthTownName || archive.birth_town_code,
  ].filter(Boolean).join(' . ') || '-';
  const cityElectionRegistered = archive.election_scope_level === 'CITY' || archive.election_scope_level === 'TOWN';
  const townElectionRegistered = archive.election_scope_level === 'TOWN';

  const detailSection = (
    <div className="card archive-detail-card">
      <div className="card__title flex-between">
        公民档案详情
        <div className="no-print" style={{ display: 'flex', gap: 8 }}>
          {!archiveDeleted && editing && <button className="btn btn--primary btn--sm" onClick={handleSave} disabled={saving}>{saving ? '保存中...' : '保存'}</button>}
          {!archiveDeleted && editing && <button className="btn btn--ghost btn--sm" onClick={() => setEditing(false)} disabled={saving}>取消</button>}
          {!archiveDeleted && !editing && <button className="btn btn--danger btn--sm" onClick={openDeleteModal} disabled={deleteBusy}>删除</button>}
          {!archiveDeleted && !editing && <button className="btn btn--primary btn--sm" onClick={startEdit}>编辑</button>}
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
        <div className="archive-detail-main">
          {editing && !archiveDeleted ? (
            <>
              <div className="form-row">
                <div className="form-group"><label>姓氏 *</label><input className="form-input" value={String(editForm.last_name || '')} onChange={e => setEditForm(f => ({ ...f, last_name: e.target.value }))} /></div>
                <div className="form-group"><label>名字 *</label><input className="form-input" value={String(editForm.first_name || '')} onChange={e => setEditForm(f => ({ ...f, first_name: e.target.value }))} /></div>
              </div>
              <div className="form-row mt-16">
                <div className="form-group">
                  <label>出生日期</label>
                  <input className="form-input" value={archive.birth_date} readOnly disabled title="出生日期保存后不能更改" />
                </div>
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
                  <label>居住镇 *</label>
                  <select className="form-input" value={String(editForm.town_code || '')} onChange={e => handleEditTownChange(e.target.value)}>
                    <option value="">请选择</option>
                    {towns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
                  </select>
                </div>
                <div className="form-group">
                  <label>地址段 *</label>
                  <select className="form-input" value={String(editForm.address_unit_id || '')} onChange={e => setEditForm(f => ({ ...f, address_unit_id: e.target.value }))}>
                    <option value="">请选择</option>
                    {addressUnits.map(v => <option key={v.address_unit_id} value={v.address_unit_id}>{v.address_unit_name}</option>)}
                  </select>
                </div>
              </div>
              <div className="form-group mt-16"><label>详细地址 *</label><input className="form-input" maxLength={100} value={String(editForm.address_detail || '')} onChange={e => setEditForm(f => ({ ...f, address_detail: e.target.value }))} /></div>
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
            </>
          ) : (
            <>
              <div className="archive-detail-grid">
                <div><strong>姓氏：</strong>{archive.last_name || '-'}</div>
                <div><strong>名字：</strong>{archive.first_name || '-'}</div>
                <div><strong>性别：</strong>{archive.gender_code === 'M' ? '男' : '女'}</div>
                <div><strong>身高：</strong>{archive.height_cm ? `${archive.height_cm} cm` : '-'}</div>
                <div><strong>出生日期：</strong>{archive.birth_date}</div>
                <div><strong>年龄：</strong>{calcAge(archive.birth_date)}</div>
                <div className="archive-detail-grid__full"><strong>出生地：</strong>{birthplaceText}</div>
                <div><strong>护照号：</strong>{archive.passport_no || '-'}</div>
                <div className="archive-detail-grid__full archive-validity-line">
                  <strong>有效期：</strong>{formatYmdZh(archive.valid_from)} - {formatYmdZh(archive.valid_until)}
                </div>
                <div><strong>居住省份：</strong>{provinceName || archive.province_code}</div>
                <div><strong>居住城市：</strong>{cityName || archive.city_code}</div>
                <div className="archive-detail-grid__full"><strong>居住地址：</strong>{[townName, archive.address_full_snapshot || [addressUnitName || archive.address_unit_name_snapshot, archive.address_detail].filter(Boolean).join(' ')].filter(Boolean).join(' ') || '-'}</div>
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
                      {!archiveDeleted && (
                        <button
                          className="btn btn--primary btn--sm no-print"
                          onClick={openWalletSettings}
                          disabled={walletBusy || !canSetWallet}
                          title={walletActionTitle}
                        >
                          设置
                        </button>
                      )}
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
                          disabled={!archive.voting_eligible}
                          title={walletActionTitle}
                          style={{ flex: 1 }}
                        />
                        {!archiveDeleted && (
                          <button
                            className="btn btn--primary btn--sm no-print"
                            onClick={openWalletSettings}
                            disabled={walletBusy || !canSetWallet}
                            title={walletActionTitle}
                            aria-label={walletActionTitle}
                            style={{ width: 36, height: 36, padding: 0, display: 'inline-flex', alignItems: 'center', justifyContent: 'center' }}
                          >
                            <ScanIcon size={18} />
                          </button>
                        )}
                      </div>
                    </>
                  )}
                </div>
                <div className="archive-election-result">
                  <div><strong>注册市选举公民：</strong>
                    <span className={`tag ${cityElectionRegistered ? 'tag--success' : 'tag--warning'}`}>
                      {cityElectionRegistered ? '已注册' : '未注册'}
                    </span>
                  </div>
                  <div><strong>注册镇选举公民：</strong>
                    <span className={`tag ${townElectionRegistered ? 'tag--success' : 'tag--warning'}`}>
                      {townElectionRegistered ? '已注册' : '未注册'}
                    </span>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>

        <div className="archive-detail-qr">
          {archive.archive_qr_payload ? (
            <div data-archive-qr="" style={{ lineHeight: 0 }}>
              <QRCodeSVG value={archive.archive_qr_payload} size={200} />
            </div>
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
  );

  const materialsSection = (
    <div className="card archive-material-section">
      <div className="card__title flex-between">
        公民资料库
        {!archiveDeleted && (
          <button className="btn btn--primary btn--sm no-print" onClick={openMaterialModal} disabled={materialBusy}>
            上传
          </button>
        )}
      </div>
      {materialError && (
        <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{materialError}</div>
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
  );

  const operationsSection = (
    <div className="card archive-operation-section">
      <div className="card__title flex-between">
        操作记录
        <span className="archive-material-count">{auditLogs.length}</span>
      </div>
      {auditError && (
        <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{auditError}</div>
      )}
      {auditLoading ? (
        <div className="archive-audit-empty">加载中...</div>
      ) : auditLogs.length === 0 ? (
        <div className="archive-audit-empty">暂无操作记录</div>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>操作</th>
              <th>操作者账户</th>
              <th>详情</th>
              <th>时间</th>
            </tr>
          </thead>
          <tbody>
            {auditLogs.map(log => (
              <tr key={log.log_id}>
                <td>{auditActionLabels[log.action] ?? log.action}</td>
                <td className="archive-audit-account" title={log.operator_account || log.operator_user_id || '-'}>
                  {log.operator_account || log.operator_user_id || '-'}
                </td>
                <td>
                  <div className="archive-audit-detail">{auditDetailWithResult(log)}</div>
                </td>
                <td>{new Date(log.created_at * 1000).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );

  const activeContent = activeTab === 'detail'
    ? detailSection
    : activeTab === 'materials'
      ? materialsSection
      : operationsSection;

  return (
    <>
      <div className="archive-detail-shell print-area">
        <div className="archive-detail-header no-print">
          <div>
            <div className="archive-detail-title">{archiveTitle}</div>
            <div className="archive-detail-subtitle">档案号：{archive.archive_no}</div>
          </div>
          <span className={`tag ${archiveDeleted ? 'tag--danger' : archive.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--warning'}`}>
            {archiveDeleted ? '已删除' : archive.citizen_status === 'NORMAL' ? '正常' : '注销'}
          </span>
        </div>
        <div className="archive-detail-layout">
          <aside className="archive-detail-nav no-print" aria-label="公民档案详情导航">
            <button className="archive-detail-nav__button archive-detail-nav__button--back" onClick={() => navigate('/admin')}>
              <span className="archive-detail-nav__icon">↩</span>
              <span>返回列表</span>
            </button>
            {archiveDetailTabs.map(tab => (
              <button
                key={tab.key}
                className={`archive-detail-nav__button ${activeTab === tab.key ? 'archive-detail-nav__button--active' : ''}`}
                onClick={() => setActiveTab(tab.key)}
                aria-current={activeTab === tab.key ? 'page' : undefined}
              >
                <span className="archive-detail-nav__icon">
                  <ArchiveDetailNavIcon type={tab.icon} />
                </span>
                <span>{tab.label}</span>
                {tab.key === 'materials' && <span className="archive-material-count">{materials.length}</span>}
                {tab.key === 'operations' && <span className="archive-material-count">{auditLogs.length}</span>}
              </button>
            ))}
          </aside>
          <section className="archive-detail-content">
            {activeContent}
          </section>
        </div>
      </div>

      {materialModalOpen && (
        <div className="modal-overlay">
          <div className="modal archive-material-modal">
            <div className="modal__title">上传公民资料</div>
            {materialError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{materialError}</div>}
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
            <div className="form-group mt-16">
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
            <div className="form-group mt-16">
              <label>备注</label>
              <input
                className="form-input"
                maxLength={200}
                value={materialNote}
                onChange={e => setMaterialNote(e.target.value)}
                disabled={materialBusy}
              />
            </div>
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeMaterialModal} disabled={materialBusy}>取消</button>
              <button className="btn btn--primary" onClick={handleMaterialUpload} disabled={materialBusy || !materialFile}>
                {materialBusy ? '上传中...' : '上传'}
              </button>
            </div>
          </div>
        </div>
      )}

      {walletModalOpen && (
        <div className="modal-overlay">
          <div className="modal archive-wallet-modal">
            <div className="modal__title">设置投票账户</div>
            {walletScanError && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 8 }}>{walletScanError}</div>}
            <div className="form-group">
              <label>投票账户</label>
              <div className="archive-wallet-draft-row">
                <input
                  className="form-input"
                  value={walletDraftAddress}
                  onChange={e => setWalletDraftAddress(e.target.value)}
                  placeholder="扫描或粘贴钱包账户"
                  disabled={walletBusy}
                />
              </div>
            </div>
            <div className="archive-election-scope archive-election-scope--modal">
              <label className="archive-election-scope__option">
                <input
                  type="checkbox"
                  checked={walletElectionCityChecked}
                  onChange={e => handleWalletElectionCityChange(e.target.checked)}
                  disabled={walletBusy}
                />
                <span>注册市选举公民</span>
              </label>
              <label className="archive-election-scope__option">
                <input
                  type="checkbox"
                  checked={walletElectionTownChecked}
                  onChange={e => handleWalletElectionTownChange(e.target.checked)}
                  disabled={walletBusy}
                />
                <span>注册镇选举公民</span>
              </label>
            </div>
            <div className="archive-wallet-scanner">
              <CameraQrScanner
                active={walletScannerActive}
                onActiveChange={setWalletScannerActive}
                onDetected={handleWalletScanned}
                onError={setWalletScanError}
                size={292}
                busy={walletBusy}
                buttonLabel="开启扫码"
                stopLabel="停止扫码"
                idleText="等待扫描钱包二维码"
                loadingText="摄像头初始化中..."
              />
            </div>
            <div className="modal__footer">
              <button className="btn btn--ghost" onClick={closeWalletModal} disabled={walletBusy}>取消</button>
              <button className="btn btn--primary" onClick={saveWalletSettings} disabled={walletBusy}>
                {walletBusy ? '保存中...' : '保存'}
              </button>
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
                  当前登录管理员使用 citizenwallet 扫码签名
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
                  hint="扫描 citizenwallet 返回的删除签名回执"
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
