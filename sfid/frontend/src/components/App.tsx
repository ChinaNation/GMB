import { useEffect, useRef, useState, useCallback } from 'react';
import QrScanner from 'qr-scanner';

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};
type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

/**
 * 启动摄像头 BarcodeDetector 扫码。返回 cleanup 函数。
 */
function startCameraScanner(
  videoEl: HTMLVideoElement,
  onDetected: (raw: string) => void,
  onReady: () => void,
  onError: (msg: string) => void,
): () => void {
  let stopped = false;
  let stream: MediaStream | null = null;
  let timer: number | undefined;

  const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
  if (!win.BarcodeDetector) {
    onError('当前浏览器不支持摄像头扫码');
    return () => {};
  }
  const detector = new win.BarcodeDetector({ formats: ['qr_code'] });

  (async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment' },
        audio: false,
      });
      if (stopped) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      videoEl.srcObject = stream;
      await videoEl.play();
      onReady();
      timer = window.setInterval(async () => {
        if (stopped) return;
        try {
          const codes = await detector.detect(videoEl);
          const raw = codes[0]?.rawValue?.trim();
          if (raw) {
            window.clearInterval(timer);
            onDetected(raw);
          }
        } catch { /* ignore frame errors */ }
      }, 500);
    } catch {
      onError('无法打开摄像头，请检查权限');
    }
  })();

  return () => {
    stopped = true;
    if (timer !== undefined) window.clearInterval(timer);
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
    }
  };
}
import { DownloadOutlined, ExclamationCircleFilled, QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Divider, Dropdown, Form, Input, Layout, Modal, QRCode, Select, Space, Table, Tag, Typography, message } from 'antd';
import { MoreOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import type {
  AdminAuth,
  AdminQrChallengeResult,
  CitizenBindChallengeResult,
  CitizenRow,
  CpmsSiteRow,
  GenerateCpmsInstitutionSfidResult,
  GenerateMultisigSfidResult,
  KeyringRotateChallengeResult,
  KeyringStateResult,
  MultisigSfidRow,
  OperatorRow,
  SfidCityItem,
  SfidMetaResult,
  SuperAdminRow
} from '../api/client';
import {
  checkAdminAuth,
  citizenBind,
  citizenBindChallenge,
  citizenUnbind,
  completeAdminQrLogin,
  createKeyringRotateChallenge,
  createOperator,
  createAdminQrChallenge,
  deleteCpmsKeys,
  deleteOperator,
  disableCpmsKeys,
  generateCpmsInstitutionSfid,
  getAttestorKeyring,
  getSfidMeta,
  listCitizens,
  listCpmsSites,
  listSfidCities,
  listOperators,
  listSuperAdmins,
  commitKeyringRotate,
  verifyKeyringRotateSignature,
  queryAdminQrLoginResult,
  replaceSuperAdmin,
  registerCpms,
  scanCpmsStatusQr,
  deleteMultisigSfid,
  generateMultisigSfid,
  getChainBalance,
  listMultisigSfids,
  updateOperator,
  updateOperatorStatus
} from '../api/client';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';

/// 区块链全链统一的"用户协议"二维码 proto 标识。
/// 详见：wuminapp/lib/qr/qr_protocols.dart
const WUMIN_USER_PROTOCOL = 'WUMIN_USER_V1.0.0';

/// 通用"扫码识别账户"弹窗。
/// 调用摄像头扫描 WUMIN_USER_V1.0.0 用户码，从中提取 `address` 字段（SS58）回填。
/// 用于：密钥管理新备用账户、新增系统管理员账户、更换机构管理员账户等任意需要账户输入的场景。
function ScanAccountModal(props: {
  open: boolean;
  onClose: () => void;
  onResolved: (ss58Address: string) => void;
}) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scannerRef = useRef<QrScanner | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Antd Modal 的 destroyOnClose 会在 open=true 之后异步挂载内容，
  // 首次 useEffect 执行时 videoRef.current 可能仍为 null，
  // 所以用回调 ref + videoMounted state 等真实 <video> 元素挂上后再启动 scanner。
  const [videoMounted, setVideoMounted] = useState(false);
  const attachVideo = useCallback((el: HTMLVideoElement | null) => {
    videoRef.current = el;
    setVideoMounted(Boolean(el));
  }, []);

  // 关闭时重置 videoMounted，确保下次再打开能重新触发 scanner 初始化
  useEffect(() => {
    if (!props.open) {
      setVideoMounted(false);
      setError(null);
    }
  }, [props.open]);

  useEffect(() => {
    if (!props.open || !videoMounted) return;
    const video = videoRef.current;
    if (!video) return;
    setError(null);
    let cancelled = false;
    const scanner = new QrScanner(
      video,
      (result) => {
        if (cancelled) return;
        try {
          const decoded = JSON.parse(result.data);
          if (decoded?.proto !== WUMIN_USER_PROTOCOL) {
            setError('不是用户协议二维码');
            return;
          }
          const addr = String(decoded.address || '').trim();
          if (!addr) {
            setError('用户码未携带 address 字段');
            return;
          }
          // 用 decodeSs58 校验一次格式（prefix + 校验和）
          try {
            decodeSs58(addr);
          } catch (e) {
            setError(e instanceof Error ? e.message : 'SS58 校验失败');
            return;
          }
          scanner.stop();
          props.onResolved(addr);
        } catch {
          setError('二维码不是有效 JSON');
        }
      },
      { highlightScanRegion: true, highlightCodeOutline: true, returnDetailedScanResult: true },
    );
    scannerRef.current = scanner;
    scanner.start().catch((err) => {
      if (cancelled) return;
      setError(err instanceof Error ? err.message : '摄像头初始化失败');
    });
    return () => {
      cancelled = true;
      const s = scannerRef.current;
      if (s) {
        s.stop();
        s.destroy();
        scannerRef.current = null;
      }
    };
  }, [props.open, videoMounted]);

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>扫描用户码</div>}
      open={props.open}
      onCancel={props.onClose}
      footer={[
        <Button key="cancel" onClick={props.onClose}>
          取消
        </Button>,
      ]}
      destroyOnClose
      width={420}
    >
      <div
        style={{
          width: '100%',
          aspectRatio: '1 / 1',
          background: 'linear-gradient(145deg, #0f172a, #1e293b)',
          borderRadius: 12,
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        <video
          ref={attachVideo}
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
          muted
          playsInline
        />
      </div>
      {error && (
        <Typography.Paragraph type="danger" style={{ marginTop: 12, marginBottom: 0 }}>
          {error}
        </Typography.Paragraph>
      )}
    </Modal>
  );
}

const loginBg = '/assets/login-bg.png';

const { Header, Content } = Layout;

/** 业务卡片统一毛玻璃风格 */
const glassCardStyle: React.CSSProperties = {
  background: 'rgba(255,255,255,0.92)',
  backdropFilter: 'blur(16px)',
  borderRadius: 16,
  boxShadow: '0 4px 24px rgba(0,0,0,0.06)',
  border: '1px solid rgba(255,255,255,0.6)'
};

/** Card title 左侧 teal 竖条 + 加粗 */
const glassCardHeadStyle: React.CSSProperties = {
  borderBottom: '1px solid #e5e7eb',
  borderLeft: '3px solid #0d9488',
  paddingLeft: 20,
  fontWeight: 600
};
const AUTH_STORAGE_KEY = 'sfid_admin_auth_v1';

function readStoredAuth(): AdminAuth | null {
  try {
    const raw = sessionStorage.getItem(AUTH_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<AdminAuth>;
    if (
      typeof parsed === 'object' &&
      parsed &&
      'access_token' in parsed &&
      typeof parsed.access_token === 'string' &&
      typeof parsed.admin_pubkey === 'string' &&
      typeof parsed.role === 'string'
    ) {
      return parsed as AdminAuth;
    }
    return null;
  } catch {
    return null;
  }
}

function writeStoredAuth(auth: AdminAuth) {
  sessionStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(auth));
}

function clearStoredAuth() {
  sessionStorage.removeItem(AUTH_STORAGE_KEY);
}

function isSr25519HexPubkey(value: string): boolean {
  const normalized = value.trim().replace(/^0x/i, '');
  return /^[0-9a-fA-F]{64}$/.test(normalized);
}

function sameHexPubkey(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  return a.trim().replace(/^0x/i, '').toLowerCase() === b.trim().replace(/^0x/i, '').toLowerCase();
}

function resolveAdminName(auth: AdminAuth | null): string {
  if (!auth) return '';
  if (typeof auth.admin_name === 'string' && auth.admin_name.trim()) {
    return auth.admin_name.trim();
  }
  if (auth.role === 'KEY_ADMIN') return '密钥管理员';
  if (auth.role === 'INSTITUTION_ADMIN') return '机构管理员';
  if (auth.role === 'SYSTEM_ADMIN') return '系统管理员';
  return '';
}

function resolveHeaderAdminName(auth: AdminAuth | null): string {
  if (!auth) return '';
  if (auth.role === 'SYSTEM_ADMIN') {
    if (typeof auth.admin_name === 'string' && auth.admin_name.trim()) {
      return `系统管理员：${auth.admin_name.trim()}`;
    }
    return '系统管理员';
  }
  return resolveAdminName(auth);
}

function createSessionId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `sid-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

type SignedLoginPayload = {
  challenge_id: string;
  session_id?: string;
  admin_pubkey: string;
  signer_pubkey?: string;
  signature: string;
};

type KeyringSignedPayload = {
  challenge_id: string;
  signature: string;
};


function parseSignedLoginPayload(raw: string, fallbackChallengeId: string): SignedLoginPayload {
  const payload = JSON.parse(raw) as Record<string, unknown>;
  const challenge_id =
    (typeof payload.request_id === 'string' && payload.request_id.trim()) ||
    (typeof payload.challenge_id === 'string' && payload.challenge_id.trim()) ||
    (typeof payload.challenge === 'string' && payload.challenge.trim()) ||
    fallbackChallengeId;
  const admin_pubkey =
    (typeof payload.account === 'string' && payload.account.trim()) ||
    (typeof payload.admin_pubkey === 'string' && payload.admin_pubkey.trim()) ||
    (typeof payload.public_key === 'string' && payload.public_key.trim()) ||
    (typeof payload.pubkey === 'string' && payload.pubkey.trim()) ||
    '';
  const signer_pubkey =
    (typeof payload.pubkey === 'string' && payload.pubkey.trim()) ||
    (typeof payload.public_key === 'string' && payload.public_key.trim()) ||
    undefined;
  const signature =
    (typeof payload.signature === 'string' && payload.signature.trim()) ||
    (typeof payload.sig === 'string' && payload.sig.trim()) ||
    '';
  const session_id = typeof payload.session_id === 'string' ? payload.session_id.trim() : undefined;
  if (!challenge_id || !admin_pubkey || !signature) {
    throw new Error('签名二维码缺少必要字段(request_id/admin_pubkey/signature)');
  }
  return { challenge_id, session_id, admin_pubkey, signer_pubkey, signature };
}

function parseKeyringSignedPayload(raw: string, fallbackChallengeId: string): KeyringSignedPayload {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error('签名二维码内容为空');
  }
  if (trimmed.startsWith('{')) {
    const payload = JSON.parse(trimmed) as Record<string, unknown>;
    const challenge_id =
      (typeof payload.request_id === 'string' && payload.request_id.trim()) ||
      (typeof payload.challenge_id === 'string' && payload.challenge_id.trim()) ||
      fallbackChallengeId;
    const signature =
      (typeof payload.signature === 'string' && payload.signature.trim()) ||
      (typeof payload.sig === 'string' && payload.sig.trim()) ||
      '';
    if (!challenge_id || !signature) {
      throw new Error('签名二维码缺少必要字段(challenge_id/signature)');
    }
    return { challenge_id, signature };
  }
  return {
    challenge_id: fallbackChallengeId,
    signature: trimmed
  };
}

function defaultInstitutionByA3(a3: string): string {
  if (a3 === 'GMR' || a3 === 'ZNR') return 'ZG';
  if (a3 === 'ZRR') return 'TG';
  if (a3 === 'GFR') return 'ZF';
  if (a3 === 'SFR' || a3 === 'FFR') return 'ZG';
  return 'ZG';
}

function usesReservedProvinceCityByA3(a3: string): boolean {
  return a3 === 'GMR' || a3 === 'ZRR' || a3 === 'ZNR';
}

function institutionCodeToName(code: string): string {
  const map: Record<string, string> = {
    ZG: '中国', ZF: '政府', LF: '立法院', SF: '司法院',
    JC: '监察院', JY: '教育委员会', CB: '储备委员会',
    CH: '储备银行', TG: '他国',
  };
  return map[code] || code;
}

function allowedInstitutionByA3(a3: string): string[] {
  if (a3 === 'GFR') return ['ZF', 'LF', 'SF', 'JC', 'JY', 'CB'];
  if (a3 === 'SFR') return ['ZG', 'CH', 'TG'];
  if (a3 === 'FFR') return ['ZG', 'TG'];
  if (a3 === 'GMR' || a3 === 'ZNR') return ['ZG'];
  if (a3 === 'ZRR') return ['TG'];
  return ['ZG', 'ZF', 'LF', 'SF', 'JC', 'JY', 'CB', 'CH', 'TG'];
}

function defaultP1ByA3(a3: string): string {
  if (a3 === 'GMR' || a3 === 'ZRR') return '1';
  if (a3 === 'GFR') return '0';
  return '0';
}

function p1LockedByA3(a3: string): boolean {
  return a3 === 'GMR' || a3 === 'ZRR' || a3 === 'GFR';
}

function reservedProvinceCityName(cities: SfidCityItem[]): string {
  return cities.find((city) => city.code === '000')?.name || '省辖市';
}

type RoleCapabilities = {
  canViewInstitutions: boolean;
  canViewMultisig: boolean;
  canViewKeyring: boolean;
  canViewInstitutionAdmins: boolean;
  canViewSystemAdmins: boolean;
  canCrudSystemAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canReplaceSuperAdmins: boolean;
  canManageKeyring: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  canViewSystemSettings: boolean;
};


function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isKeyAdmin = role === 'KEY_ADMIN';
  const isInstitutionAdmin = role === 'INSTITUTION_ADMIN';
  const isSystemAdmin = role === 'SYSTEM_ADMIN';
  return {
    canViewInstitutions: isKeyAdmin || isInstitutionAdmin,
    canViewMultisig: isKeyAdmin || isInstitutionAdmin || isSystemAdmin,
    canViewKeyring: isKeyAdmin,
    canViewInstitutionAdmins: isKeyAdmin || isInstitutionAdmin,
    canViewSystemAdmins: isKeyAdmin || isInstitutionAdmin || isSystemAdmin,
    canCrudSystemAdmins: isKeyAdmin || isInstitutionAdmin,
    canManageInstitutions: isKeyAdmin || isInstitutionAdmin,
    canRegisterInstitutions: isKeyAdmin || isInstitutionAdmin,
    canReplaceSuperAdmins: isKeyAdmin,
    canManageKeyring: isKeyAdmin,
    canStatusScan: isKeyAdmin || isInstitutionAdmin || isSystemAdmin,
    canBusinessWrite: true,
    canViewSystemSettings: isKeyAdmin || isInstitutionAdmin || isSystemAdmin,
  };
}

export default function App() {
  const [auth, setAuth] = useState<AdminAuth | null>(() => readStoredAuth());
  const [rows, setRows] = useState<CitizenRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [binding, setBinding] = useState(false);
  const [bindModalOpen, setBindModalOpen] = useState(false);
  const [bindTargetPubkey, setBindTargetPubkey] = useState('');
  const [bootstrapping, setBootstrapping] = useState(true);
  const [pendingQrLogin, setPendingQrLogin] = useState<AdminQrChallengeResult | null>(null);
  const [challengeLoading, setChallengeLoading] = useState(false);
  const [bindMode, setBindMode] = useState<'bind_archive' | 'bind_pubkey'>('bind_archive');
  const [bindTargetRecord, setBindTargetRecord] = useState<CitizenRow | null>(null);
  const [bindChallenge, setBindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [bindChallengeLoading, setBindChallengeLoading] = useState(false);
  const [bindQr4Payload, setBindQr4Payload] = useState<string | null>(null);
  const [bindQr4ScanLoading, setBindQr4ScanLoading] = useState(false);
  const [bindSignature, setBindSignature] = useState<string | null>(null);
  const [bindStep, setBindStep] = useState<'scan_qr4' | 'sign_challenge' | 'scan_signature' | 'input_pubkey' | 'done'>('scan_qr4');
  const [bindNewPubkey, setBindNewPubkey] = useState('');
  const [unbindModalOpen, setUnbindModalOpen] = useState(false);
  const [unbindTarget, setUnbindTarget] = useState<CitizenRow | null>(null);
  const [unbindChallenge, setUnbindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [unbindChallengeLoading, setUnbindChallengeLoading] = useState(false);
  const [unbindScannerActive, setUnbindScannerActive] = useState(false);
  const [unbindScannerReady, setUnbindScannerReady] = useState(false);
  const [unbindSubmitting, setUnbindSubmitting] = useState(false);
  const [unbindStep, setUnbindStep] = useState<'confirm' | 'sign_challenge' | 'scan_signature'>('confirm');
  const unbindVideoRef = useRef<HTMLVideoElement | null>(null);
  const unbindScanCleanupRef = useRef<(() => void) | null>(null);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const [scannerActive, setScannerActive] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [activeView, setActiveView] = useState<'citizens' | 'institutions' | 'multisig' | 'system-settings' | 'keyring' | 'super-admins' | 'operators'>('citizens');
  const [selectedSuperAdmin, setSelectedSuperAdmin] = useState<SuperAdminRow | null>(null);
  const [operatorCities, setOperatorCities] = useState<SfidCityItem[]>([]);
  const [operatorCitiesLoading, setOperatorCitiesLoading] = useState(false);
  // 密钥管理：主账户链上余额（已格式化为 xxx.xx），失败时 mainAccountBalance=null + mainAccountBalanceError
  const [mainAccountBalance, setMainAccountBalance] = useState<string | null>(null);
  const [mainAccountBalanceError, setMainAccountBalanceError] = useState<string | null>(null);
  // 密钥管理：扫码识别"新备用账户"弹窗开关
  const [keyringScanAccountOpen, setKeyringScanAccountOpen] = useState(false);
  // 通用扫码识别账户目标（null = 关闭；'operator' = 新增系统管理员账户；'super-admin' = 更换机构管理员账户）
  const [accountScanTarget, setAccountScanTarget] = useState<null | 'operator' | 'super-admin'>(null);
  // 多签管理：当前选中的 SFID 详情（null = 显示列表；非空 = 显示详情页）
  const [selectedMultisigSfid, setSelectedMultisigSfid] = useState<MultisigSfidRow | null>(null);
  // 机构详情页 sub-tab：'operators'（系统管理员列表，默认）/ 'super-admin'（机构管理员）
  const [adminDetailTab, setAdminDetailTab] = useState<'operators' | 'super-admin'>('operators');
  // 系统管理员列表受控分页
  const [operatorListPage, setOperatorListPage] = useState(1);
  const [operators, setOperators] = useState<OperatorRow[]>([]);
  const [operatorsLoading, setOperatorsLoading] = useState(false);
  const [operatorPage, setOperatorPage] = useState(1);
  const [superAdmins, setSuperAdmins] = useState<SuperAdminRow[]>([]);
  const [superAdminsLoading, setSuperAdminsLoading] = useState(false);
  const [replaceSuperLoading, setReplaceSuperLoading] = useState(false);
  const [addOperatorOpen, setAddOperatorOpen] = useState(false);
  const [addOperatorLoading, setAddOperatorLoading] = useState(false);
  const [cpmsSites, setCpmsSites] = useState<CpmsSiteRow[]>([]);
  const [cpmsSitesLoading, setCpmsSitesLoading] = useState(false);
  const [institutionSfidOpen, setInstitutionSfidOpen] = useState(false);
  const [institutionSfidLoading, setInstitutionSfidLoading] = useState(false);
  const [institutionSfidResult, setInstitutionSfidResult] = useState<GenerateCpmsInstitutionSfidResult | null>(null);
  const [institutionQrPreview, setInstitutionQrPreview] = useState<GenerateCpmsInstitutionSfidResult | null>(null);
  const [opScanOpen, setOpScanOpen] = useState(false);
  const [opScanType, setOpScanType] = useState<'register' | 'status'>('register');
  const [opScannerReady, setOpScannerReady] = useState(false);
  const [opScanSubmitting, setOpScanSubmitting] = useState(false);
  const [keyringState, setKeyringState] = useState<KeyringStateResult | null>(null);
  const [keyringLoading, setKeyringLoading] = useState(false);
  const [keyringActionLoading, setKeyringActionLoading] = useState(false);
  const [keyringChallenge, setKeyringChallenge] = useState<KeyringRotateChallengeResult | null>(null);
  const [keyringSignedPayload, setKeyringSignedPayload] = useState<KeyringSignedPayload | null>(null);
  const [keyringScannerActive, setKeyringScannerActive] = useState(false);
  const [keyringScannerReady, setKeyringScannerReady] = useState(false);
  const [keyringScanSubmitting, setKeyringScanSubmitting] = useState(false);
  const [keyringCommitLoading, setKeyringCommitLoading] = useState(false);
  const [sfidMeta, setSfidMeta] = useState<SfidMetaResult | null>(null);
  const [sfidCities, setSfidCities] = useState<SfidCityItem[]>([]);
  const [sfidCitiesLoading, setSfidCitiesLoading] = useState(false);
  // ── 多签管理 ──
  const [multisigRows, setMultisigRows] = useState<MultisigSfidRow[]>([]);
  const [multisigLoading, setMultisigLoading] = useState(false);
  const [multisigModalOpen, setMultisigModalOpen] = useState(false);
  const [multisigGenerating, setMultisigGenerating] = useState(false);
  const [multisigPage, setMultisigPage] = useState(1);
  const [multisigA3, setMultisigA3] = useState('GFR');
  const [multisigForm] = Form.useForm();
  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string; operator_city: string }>();
  const [institutionSfidForm] = Form.useForm<{
    province: string;
    city: string;
    institution: string;
    institution_name: string;
  }>();
  const [replaceSuperForm] = Form.useForm<{ province: string; admin_pubkey: string }>();
  const [keyringForm] = Form.useForm<{ new_backup_pubkey: string }>();
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const loginScanCleanupRef = useRef<(() => void) | null>(null);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanCleanupRef = useRef<(() => void) | null>(null);
  const opVideoRef = useRef<HTMLVideoElement | null>(null);
  const opScanCleanupRef = useRef<(() => void) | null>(null);
  const keyringVideoRef = useRef<HTMLVideoElement | null>(null);
  const keyringScanCleanupRef = useRef<(() => void) | null>(null);
  const institutionQrRef = useRef<HTMLDivElement | null>(null);
  const institutionQrPreviewRef = useRef<HTMLDivElement | null>(null);

  const capabilities = resolveRoleCapabilities(auth);

  // 切换 selectedSuperAdmin 时：
  //   1. 预加载该机构所属省份的城市列表（独立于 sfidCities，避免和 CPMS / 多签弹窗冲突）
  //   2. 重置详情页 sub-tab 到默认（系统管理员列表）
  //   3. 重置系统管理员列表分页到第 1 页
  useEffect(() => {
    if (!selectedSuperAdmin || !auth) {
      setOperatorCities([]);
      return;
    }
    setOperatorCities([]);
    setAdminDetailTab('operators');
    setOperatorListPage(1);
    setOperatorCitiesLoading(true);
    let cancelled = false;
    listSfidCities(auth, selectedSuperAdmin.province)
      .then((rows) => {
        if (!cancelled) setOperatorCities(rows);
      })
      .catch(() => {
        if (!cancelled) setOperatorCities([]);
      })
      .finally(() => {
        if (!cancelled) setOperatorCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [selectedSuperAdmin?.admin_pubkey, auth?.access_token]);

  useEffect(() => {
    let cancelled = false;
    const bootstrap = async () => {
      if (!auth) {
        setBootstrapping(false);
        return;
      }
      try {
        const checked = await checkAdminAuth(auth);
        const refreshedAuth: AdminAuth = {
          ...auth,
          admin_pubkey: checked.admin_pubkey,
          role: checked.role,
          admin_name: checked.admin_name,
          admin_province: checked.admin_province ?? null,
          admin_city: checked.admin_city ?? null
        };
        setAuth(refreshedAuth);
        writeStoredAuth(refreshedAuth);
        const list = await listCitizens(refreshedAuth);
        if (!cancelled) {
          setRows(list);
        }
      } catch {
        if (!cancelled) {
          clearStoredAuth();
          setAuth(null);
          setRows([]);
          message.warning('登录状态已失效，请重新登录');
        }
      } finally {
        if (!cancelled) {
          setBootstrapping(false);
        }
      }
    };
    bootstrap();
    return () => {
      cancelled = true;
    };
  }, []);

  const onCreateQrLogin = async () => {
    setChallengeLoading(true);
    try {
      const sessionId = createSessionId();
      const origin = window.location.origin;
      const challenge = await createAdminQrChallenge({
        origin,
        session_id: sessionId
      });
      setPendingQrLogin(challenge);
      setScannerActive(false);
      stopScanner();
      message.success('登录二维码已生成');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成登录二维码失败';
      message.error(msg);
      setPendingQrLogin(null);
    } finally {
      setChallengeLoading(false);
    }
  };

  const stopScanner = () => {
    if (loginScanCleanupRef.current) {
      loginScanCleanupRef.current();
      loginScanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  const onCompleteSignedLogin = async (raw: string) => {
    if (!pendingQrLogin) {
      message.error('请先生成登录二维码');
      return;
    }
    setScanSubmitting(true);
    try {
      const payload = parseSignedLoginPayload(raw, pendingQrLogin.challenge_id);
      await completeAdminQrLogin({
        challenge_id: payload.challenge_id,
        session_id: payload.session_id || pendingQrLogin.session_id,
        admin_pubkey: payload.admin_pubkey,
        signer_pubkey: payload.signer_pubkey,
        signature: payload.signature
      });
      message.success('签名已提交，正在确认登录结果');
      stopScanner();
      setScannerActive(false);
      const status = await queryAdminQrLoginResult(pendingQrLogin.challenge_id, pendingQrLogin.session_id);
      if (status.status === 'SUCCESS' && status.access_token && status.admin) {
        const nextAuth: AdminAuth = {
          access_token: status.access_token,
          admin_pubkey: status.admin.admin_pubkey,
          role: status.admin.role,
          admin_name: status.admin.admin_name,
          admin_province: status.admin.admin_province ?? null,
          admin_city: status.admin.admin_city ?? null
        };
        setAuth(nextAuth);
        writeStoredAuth(nextAuth);
        setPendingQrLogin(null);
        message.success('登录成功');
        await refreshList(nextAuth, undefined, true);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '签名二维码处理失败';
      if (msg.includes('admin not found')) {
        message.error('非管理员禁止登录本系统');
      } else {
        message.error(msg);
      }
    } finally {
      setScanSubmitting(false);
    }
  };

  useEffect(() => {
    if (!scannerActive || !pendingQrLogin || !videoRef.current) {
      stopScanner();
      return;
    }
    loginScanCleanupRef.current = startCameraScanner(
      videoRef.current,
      (raw) => { setScannerActive(false); stopScanner(); void onCompleteSignedLogin(raw); },
      () => setScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopScanner();
  }, [scannerActive, pendingQrLogin]);

  useEffect(() => {
    if (!opScanOpen || !opVideoRef.current) {
      stopOpScanner();
      return;
    }
    opScanCleanupRef.current = startCameraScanner(
      opVideoRef.current,
      (raw) => void onHandleOperationQr(raw),
      () => setOpScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopOpScanner();
  }, [opScanOpen, opScanType, auth]);

  useEffect(() => {
    if (!keyringScannerActive || !keyringChallenge || !keyringVideoRef.current) {
      stopKeyringScanner();
      return;
    }
    keyringScanCleanupRef.current = startCameraScanner(
      keyringVideoRef.current,
      (raw) => { setKeyringScannerActive(false); stopKeyringScanner(); void onCompleteKeyringRotate(raw); },
      () => setKeyringScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopKeyringScanner();
  }, [keyringScannerActive, keyringChallenge]);

  const onToggleScanner = () => {
    if (!pendingQrLogin) {
      message.warning('请先生成登录二维码');
      return;
    }
    setScannerActive((v) => !v);
  };

  useEffect(() => {
    if (auth || !pendingQrLogin) return;
    let cancelled = false;
    const timer = window.setInterval(async () => {
      try {
        const status = await queryAdminQrLoginResult(
          pendingQrLogin.challenge_id,
          pendingQrLogin.session_id
        );
        if (cancelled) return;
        if (status.status === 'PENDING') return;
        if (status.status === 'EXPIRED') {
          message.warning('二维码已过期，请重新生成');
          setPendingQrLogin(null);
          return;
        }
        if (status.status === 'SUCCESS' && status.access_token && status.admin) {
          const nextAuth: AdminAuth = {
            access_token: status.access_token,
            admin_pubkey: status.admin.admin_pubkey,
            role: status.admin.role,
            admin_name: status.admin.admin_name,
            admin_province: status.admin.admin_province ?? null,
            admin_city: status.admin.admin_city ?? null
          };
          setAuth(nextAuth);
          writeStoredAuth(nextAuth);
          setPendingQrLogin(null);
          message.success('登录成功');
          await refreshList(nextAuth, undefined, true);
        }
      } catch {
        // keep polling
      }
    }, 1200);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [auth, pendingQrLogin]);

  const onLogout = () => {
    setAuth(null);
    clearStoredAuth();
    setRows([]);
    setBindModalOpen(false);
    setBindTargetPubkey('');
    setPendingQrLogin(null);
    setActiveView('citizens');
    setOperators([]);
    setSuperAdmins([]);
    setCpmsSites([]);
    setOpScanOpen(false);
    stopOpScanner();
    setUnbindModalOpen(false);
    setUnbindTarget(null);
    setUnbindChallenge(null);
    stopUnbindScanner();
    setKeyringState(null);
    setKeyringChallenge(null);
    setKeyringSignedPayload(null);
    setKeyringScannerActive(false);
    stopKeyringScanner();
    keyringForm.resetFields();
    message.success('已退出登录');
  };

  const refreshList = async (currentAuth: AdminAuth, keyword?: string, silent?: boolean) => {
    setLoading(true);
    try {
      const raw = await listCitizens(currentAuth, keyword);
      const list = Array.isArray(raw) ? raw : [];
      setRows(list);
      if (keyword && list.length === 0) {
        Modal.warning({
          title: '查询结果',
          content: '没有的公民信息'
        });
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '查询失败';
      if (!silent) {
        message.error(msg);
      }
    } finally {
      setLoading(false);
    }
  };

  const onSearch = async (values: { keyword: string }) => {
    if (!auth) return;
    let keyword = values.keyword?.trim() || '';
    // 用户输入可能是 SS58 账户 / hex 公钥 / 档案号 / SFID 码
    // 前端先尝试 SS58 解码，成功则转成 hex 公钥提交；失败则原样提交（后端按老规则匹配）
    if (keyword) {
      try {
        keyword = decodeSs58(keyword);
      } catch {
        // 非 SS58 格式，保留原值
      }
    }
    await refreshList(auth, keyword);
  };

  const refreshOperators = async (currentAuth: AdminAuth) => {
    setOperatorsLoading(true);
    try {
      const rows = await listOperators(currentAuth);
      setOperators(Array.isArray(rows) ? rows : []);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载系统管理员失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  const refreshSuperAdmins = async (currentAuth: AdminAuth) => {
    setSuperAdminsLoading(true);
    try {
      const rows = await listSuperAdmins(currentAuth);
      setSuperAdmins(Array.isArray(rows) ? rows : []);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载机构管理员失败';
      message.error(msg);
    } finally {
      setSuperAdminsLoading(false);
    }
  };

  const refreshCpmsSites = async (currentAuth: AdminAuth) => {
    setCpmsSitesLoading(true);
    try {
      const rows = await listCpmsSites(currentAuth);
      setCpmsSites(Array.isArray(rows) ? rows : []);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载机构列表失败';
      message.error(msg);
    } finally {
      setCpmsSitesLoading(false);
    }
  };

  // ── 多签管理 ──
  const refreshMultisigSfids = async (currentAuth: AdminAuth) => {
    setMultisigLoading(true);
    try {
      const rows = await listMultisigSfids(currentAuth);
      setMultisigRows(Array.isArray(rows) ? rows : []);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载多签机构列表失败';
      message.error(msg);
    } finally {
      setMultisigLoading(false);
    }
  };

  const onDeleteMultisigSfid = (row: MultisigSfidRow) => {
    if (!auth) return;
    if (row.chain_status === 'REGISTERED') {
      message.warning('该 SFID 已在链上注册，不能删除');
      return;
    }
    Modal.confirm({
      title: '删除注册机构 SFID',
      content: (
        <div>
          <div>确认删除以下记录？该操作不可恢复。</div>
          <div style={{ marginTop: 8, fontFamily: 'monospace', fontSize: 12, wordBreak: 'break-all' }}>
            {row.site_sfid}
          </div>
          <div style={{ marginTop: 4, fontSize: 12, color: 'rgba(0,0,0,0.6)' }}>
            {row.institution_name} · {row.province} {row.city}
          </div>
        </div>
      ),
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        try {
          await deleteMultisigSfid(auth, row.site_sfid);
          message.success('删除成功');
          await refreshMultisigSfids(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除失败';
          message.error(msg);
          throw err;
        }
      },
    });
  };

  const openMultisigModal = async () => {
    if (!auth) return;
    try {
      const meta = await getSfidMeta(auth);
      setSfidMeta(meta);
      const defaultA3 = 'GFR';
      setMultisigA3(defaultA3);
      const provinceDefault = auth.admin_province || meta.provinces[0]?.name || '';
      // SystemAdmin 的市由 session 锁定，默认填入
      const cityDefault = auth.role === 'SYSTEM_ADMIN' ? (auth.admin_city || '') : '';
      multisigForm.setFieldsValue({
        a3: defaultA3,
        p1: '0',
        province: provinceDefault,
        city: cityDefault,
        institution: defaultInstitutionByA3(defaultA3),
        institution_name: '',
      });
      if (provinceDefault) {
        await loadSfidCities(provinceDefault);
      } else {
        setSfidCities([]);
      }
      setMultisigModalOpen(true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载SFID工具配置失败';
      message.error(msg);
    }
  };

  const onGenerateMultisigSfid = async (values: {
    a3: string;
    p1?: string;
    province: string;
    city: string;
    institution: string;
    institution_name: string;
  }) => {
    if (!auth) return;
    setMultisigGenerating(true);
    try {
      await generateMultisigSfid(auth, {
        a3: values.a3.trim(),
        p1: values.p1?.trim(),
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim(),
        institution_name: values.institution_name.trim(),
      });
      message.success('多签机构 SFID 已生成并上链注册');
      setMultisigModalOpen(false);
      multisigForm.resetFields();
      await refreshMultisigSfids(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成多签机构 SFID 失败';
      message.error(msg);
    } finally {
      setMultisigGenerating(false);
    }
  };

  // ── 系统设置（机构管理员列表 / 详情页） ──
  // KeyAdmin 进入后看列表；InstitutionAdmin 和 SystemAdmin 自动跳到自己（或所属）的详情页。
  const openSystemSettings = async (currentAuth: AdminAuth) => {
    if (currentAuth.role === 'KEY_ADMIN') {
      setSelectedSuperAdmin(null);
      await refreshSuperAdmins(currentAuth);
      await refreshOperators(currentAuth);
      return;
    }
    // 拿全量 super-admins，找到自己（InstitutionAdmin）或所属机构（SystemAdmin）
    let rows: SuperAdminRow[] = [];
    try {
      rows = await listSuperAdmins(currentAuth);
      setSuperAdmins(rows);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载机构管理员列表失败';
      message.error(msg);
      return;
    }
    await refreshOperators(currentAuth);

    let target: SuperAdminRow | null = null;
    if (currentAuth.role === 'INSTITUTION_ADMIN') {
      target = rows.find((r) => sameHexPubkey(r.admin_pubkey, currentAuth.admin_pubkey)) || null;
    } else if (currentAuth.role === 'SYSTEM_ADMIN') {
      // 通过当前操作员的 created_by 找到所属机构管理员
      try {
        const ops = await listOperators(currentAuth);
        const me = ops.find((o) => sameHexPubkey(o.admin_pubkey, currentAuth.admin_pubkey));
        if (me) {
          target = rows.find((r) => sameHexPubkey(r.admin_pubkey, me.created_by)) || null;
        }
      } catch {
        // 静默：无法定位时回退到列表（虽然 SystemAdmin 大概率看不到列表）
      }
    }
    setSelectedSuperAdmin(target);
  };

  const refreshKeyringState = async (currentAuth: AdminAuth) => {
    setKeyringLoading(true);
    try {
      const state = await getAttestorKeyring(currentAuth);
      setKeyringState(state);
      // 拉到主账户后立即查链上余额（每次进入密钥管理页都查一次，不缓存）
      if (state?.main_pubkey) {
        setMainAccountBalance(null);
        setMainAccountBalanceError(null);
        try {
          const bal = await getChainBalance(currentAuth, state.main_pubkey);
          setMainAccountBalance(bal.balance_text);
        } catch (err) {
          setMainAccountBalanceError(err instanceof Error ? err.message : String(err));
        }
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载密钥状态失败';
      message.error(msg);
    } finally {
      setKeyringLoading(false);
    }
  };

  const stopKeyringScanner = () => {
    if (keyringScanCleanupRef.current) {
      keyringScanCleanupRef.current();
      keyringScanCleanupRef.current = null;
    }
    setKeyringScannerReady(false);
  };

  const onCreateKeyringRotateChallenge = async (values: { new_backup_pubkey: string }) => {
    if (!auth) return;
    // 主公钥不能发起轮换
    if (keyringState && auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() === keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase()) {
      message.error('主密钥不能发起轮换，请用备用密钥登录');
      return;
    }
    setKeyringActionLoading(true);
    try {
      const challenge = await createKeyringRotateChallenge(auth, {
        initiator_pubkey: auth.admin_pubkey
      });
      setKeyringChallenge(challenge);
      setKeyringSignedPayload(null);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('轮换签名二维码已生成，请用备用私钥钱包扫码签名');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成轮换挑战失败';
      message.error(msg);
      setKeyringChallenge(null);
    } finally {
      setKeyringActionLoading(false);
    }
  };

  const onCompleteKeyringRotate = async (raw: string) => {
    if (!auth || !keyringChallenge) {
      message.error('请先生成轮换二维码');
      return;
    }
    const newBackupAddress = keyringForm.getFieldValue('new_backup_pubkey')?.trim();
    if (!newBackupAddress) {
      message.error('新备用账户不能为空');
      return;
    }
    // 用户输入的是 SS58 地址，链上接口仍是 32 字节 hex 公钥，提交前转码
    let newBackupPubkey: string;
    try {
      newBackupPubkey = decodeSs58(newBackupAddress);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setKeyringScanSubmitting(true);
    try {
      const payload = parseKeyringSignedPayload(raw, keyringChallenge.challenge_id);
      await verifyKeyringRotateSignature(auth, {
        challenge_id: payload.challenge_id,
        signature: payload.signature
      });
      setKeyringSignedPayload(payload);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('签名校验通过，正在提交轮换...');
      // 自动提交 commit
      setKeyringCommitLoading(true);
      try {
        const result = await commitKeyringRotate(auth, {
          challenge_id: payload.challenge_id,
          signature: payload.signature,
          new_backup_pubkey: newBackupPubkey
        });
        if (result.chain_submit_ok) {
          message.success(`主密钥轮换成功，新版本：${result.version}`);
        } else {
          message.warning(
            `主密钥已本地轮换为版本 ${result.version}，但上链提交失败：${result.chain_submit_error || '未知错误'}`
          );
        }
        setKeyringChallenge(null);
        setKeyringSignedPayload(null);
        keyringForm.resetFields();
        await refreshKeyringState(auth);
      } catch (commitErr) {
        const commitMsg = commitErr instanceof Error ? commitErr.message : '提交轮换失败';
        message.error(commitMsg);
      } finally {
        setKeyringCommitLoading(false);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '提交轮换签名失败';
      message.error(msg);
    } finally {
      setKeyringScanSubmitting(false);
    }
  };


  const onToggleKeyringScanner = () => {
    if (!keyringChallenge) {
      message.warning('请先生成轮换二维码');
      return;
    }
    setKeyringScannerActive((v) => !v);
  };

  const onCreateOperator = async (values: { operator_pubkey: string; operator_name: string; city?: string; created_by?: string }) => {
    if (!auth) return;
    const inputAddr = values.operator_pubkey?.trim();
    const admin_name = values.operator_name?.trim();
    const city = (values.city ?? '').trim();
    if (!inputAddr) {
      message.error('请输入管理员账户');
      return;
    }
    if (!admin_name) {
      message.error('请输入管理员姓名');
      return;
    }
    if (!city) {
      message.error('请选择市');
      return;
    }
    // 用户输入的是 SS58 地址，后端 API 要求 32 字节 hex 公钥
    let admin_pubkey: string;
    try {
      admin_pubkey = decodeSs58(inputAddr);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setAddOperatorLoading(true);
    try {
      const created = await createOperator(auth, { admin_pubkey, admin_name, city, created_by: values.created_by });
      message.success('管理员新增成功');
      addOperatorForm.resetFields();
      setAddOperatorOpen(false);
      setOperatorPage(1);
      setOperators((prev) => {
        const rest = prev.filter((item) => item.admin_pubkey !== created.admin_pubkey);
        return [created, ...rest];
      });
      await refreshOperators(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '新增管理员失败';
      message.error(msg);
    } finally {
      setAddOperatorLoading(false);
    }
  };

  const stopOpScanner = () => {
    if (opScanCleanupRef.current) {
      opScanCleanupRef.current();
      opScanCleanupRef.current = null;
    }
    setOpScannerReady(false);
  };

  const onHandleOperationQr = async (raw: string) => {
    if (!auth) return;
    setOpScanSubmitting(true);
    try {
      if (opScanType === 'register') {
        const result = await registerCpms(auth, { qr_payload: raw });
        message.success(`机构 ${result.qr3_payload ? '公钥登记成功' : '登记成功'}`);
        await refreshCpmsSites(auth);
      } else {
        const result = await scanCpmsStatusQr(auth, { qr_payload: raw });
        message.success(`状态已更新：${result.archive_no} -> ${result.status}`);
        await refreshList(auth, undefined, true);
      }
      setOpScanOpen(false);
      stopOpScanner();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '扫码处理失败';
      message.error(msg);
    } finally {
      setOpScanSubmitting(false);
    }
  };


  const onDisableCpmsSite = (row: CpmsSiteRow) => {
    if (!auth) return;
    let reason = '';
    Modal.confirm({
      title: `禁用机构 (${row.site_sfid})`,
      content: (
        <Input
          placeholder="禁用原因（可选）"
          onChange={(event) => {
            reason = event.target.value;
          }}
        />
      ),
      okText: '确认禁用',
      cancelText: '取消',
      onOk: async () => {
        setCpmsSitesLoading(true);
        try {
          await disableCpmsKeys(auth, row.site_sfid, reason.trim() || undefined);
          message.success(`机构 ${row.site_sfid} 已禁用`);
          await refreshCpmsSites(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '禁用机构失败';
          message.error(msg);
          throw err;
        } finally {
          setCpmsSitesLoading(false);
        }
      }
    });
  };

  const onDeleteCpmsSite = (row: CpmsSiteRow) => {
    if (!auth) return;
    Modal.confirm({
      title: '删除机构',
      content: `确认删除该机构？\n${row.site_sfid}`,
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setCpmsSitesLoading(true);
        try {
          await deleteCpmsKeys(auth, row.site_sfid);
          message.success(`机构 ${row.site_sfid} 已删除`);
          await refreshCpmsSites(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除机构失败';
          message.error(msg);
          throw err;
        } finally {
          setCpmsSitesLoading(false);
        }
      }
    });
  };

  const onToggleOperatorStatus = async (row: OperatorRow) => {
    if (!auth) return;
    const target = row.status === 'ACTIVE' ? 'DISABLED' : 'ACTIVE';
    setOperatorsLoading(true);
    try {
      await updateOperatorStatus(auth, row.id, target);
      message.success(target === 'ACTIVE' ? '已启用系统管理员' : '已停用系统管理员');
      await refreshOperators(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更新系统管理员状态失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  const onUpdateOperator = (row: OperatorRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    let nextAddr = tryEncodeSs58(row.admin_pubkey);
    let nextCity = row.city;
    const cityOptions = operatorCities
      .filter((c) => c.code !== '000')
      .map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }));
    Modal.confirm({
      title: '修改系统管理员',
      content: (
        <Space direction="vertical" style={{ width: '100%' }}>
          <Input
            defaultValue={row.admin_name}
            placeholder="请输入管理员姓名"
            onChange={(event) => {
              nextName = event.target.value;
            }}
          />
          <Select
            defaultValue={row.city || undefined}
            placeholder="请选择市"
            style={{ width: '100%' }}
            options={cityOptions}
            onChange={(value: string) => {
              nextCity = value;
            }}
          />
          <Input
            defaultValue={tryEncodeSs58(row.admin_pubkey)}
            placeholder="请输入新的管理员账户（SS58）"
            onChange={(event) => {
              nextAddr = event.target.value;
            }}
          />
        </Space>
      ),
      okText: '确认修改',
      cancelText: '取消',
      onOk: async () => {
        const admin_name = nextName.trim();
        const inputAddr = nextAddr.trim();
        const city = (nextCity || '').trim();
        if (!admin_name) {
          message.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        if (!inputAddr) {
          message.error('请输入管理员账户');
          throw new Error('admin_pubkey is required');
        }
        if (!city) {
          message.error('请选择市');
          throw new Error('city is required');
        }
        let admin_pubkey: string;
        try {
          admin_pubkey = decodeSs58(inputAddr);
        } catch (err) {
          message.error(err instanceof Error ? err.message : '账户格式无效');
          throw err;
        }
        setOperatorsLoading(true);
        try {
          await updateOperator(auth, row.id, { admin_name, admin_pubkey, city });
          message.success('系统管理员信息已更新');
          await refreshOperators(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '更新系统管理员信息失败';
          message.error(msg);
          throw err;
        } finally {
          setOperatorsLoading(false);
        }
      }
    });
  };

  const loadSfidCities = async (province: string) => {
    if (!auth || !province.trim()) return [] as SfidCityItem[];
    setSfidCitiesLoading(true);
    try {
      const rows = await listSfidCities(auth, province);
      setSfidCities(rows);
      return rows;
    } catch (err) {
      setSfidCities([]);
      const msg = err instanceof Error ? err.message : '加载城市列表失败';
      message.error(msg);
      return [] as SfidCityItem[];
    } finally {
      setSfidCitiesLoading(false);
    }
  };


  const onDeleteOperator = (row: OperatorRow) => {
    if (!auth) return;
    Modal.confirm({
      title: '删除系统管理员',
      content: `确认删除该系统管理员？\n${row.admin_pubkey}`,
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setOperatorsLoading(true);
        try {
          await deleteOperator(auth, row.id);
          message.success('系统管理员已删除');
          await refreshOperators(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除系统管理员失败';
          message.error(msg);
        } finally {
          setOperatorsLoading(false);
        }
      }
    });
  };

  const onReplaceSuperAdmin = async (values: { province: string; admin_pubkey: string }) => {
    if (!auth) return;
    // 用户输入的是 SS58 地址，后端 API 仍接受 32 字节 hex 公钥
    const inputAddr = values.admin_pubkey.trim();
    let hexPubkey: string;
    try {
      hexPubkey = decodeSs58(inputAddr);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setReplaceSuperLoading(true);
    try {
      await replaceSuperAdmin(auth, values.province.trim(), hexPubkey);
      message.success(`已更新 ${values.province} 机构管理员`);
      replaceSuperForm.resetFields();
      await refreshSuperAdmins(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更换机构管理员失败';
      message.error(msg);
    } finally {
      setReplaceSuperLoading(false);
    }
  };

  const openBindModal = (record: CitizenRow) => {
    const mode = record.status === 'UNLINKED' ? 'bind_pubkey' : 'bind_archive';
    setBindTargetPubkey(record.account_pubkey || '');
    setBindTargetRecord(record);
    setBindMode(mode);
    setBindChallenge(null);
    setBindQr4Payload(null);
    setBindSignature(null);
    setBindNewPubkey('');
    setBindStep(mode === 'bind_archive' ? 'scan_qr4' : 'input_pubkey');
    setBindScannerActive(false);
    stopBindScanner();
    setBindModalOpen(true);
  };

  const openRegisterScanner = () => {
    if (!capabilities.canRegisterInstitutions) {
      message.error('仅机构管理员可录入机构');
      return;
    }
    setOpScanType('register');
    setOpScanOpen(true);
  };

  const openInstitutionSfidModal = async () => {
    if (!capabilities.canRegisterInstitutions) {
      message.error('仅机构管理员可生成机构身份识别码');
      return;
    }
    if (!auth) return;
    try {
      const meta = await getSfidMeta(auth);
      setSfidMeta(meta);
      const provinceDefault = auth.admin_province || meta.provinces[0]?.name || '';
      institutionSfidForm.setFieldsValue({
        province: provinceDefault,
        city: '',
        institution: defaultInstitutionByA3('GFR')
      });
      setInstitutionSfidResult(null);
      if (provinceDefault) {
        await loadSfidCities(provinceDefault);
      } else {
        setSfidCities([]);
      }
      setInstitutionSfidOpen(true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载SFID工具配置失败';
      message.error(msg);
    }
  };

  const onGenerateInstitutionSfid = async (values: { province: string; city: string; institution: string; institution_name: string }) => {
    if (!auth) return;
    setInstitutionSfidLoading(true);
    try {
      const result = await generateCpmsInstitutionSfid(auth, {
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim(),
        institution_name: values.institution_name.trim()
      });
      setInstitutionSfidResult(result);
      message.success(`身份识别码已生成：${result.site_sfid}`);
      setInstitutionSfidOpen(false);
      await refreshCpmsSites(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成机构身份识别码失败';
      message.error(msg);
    } finally {
      setInstitutionSfidLoading(false);
    }
  };

  const onFinishInstitutionSfid = async () => {
    if (!institutionSfidResult) return;
    setInstitutionSfidOpen(false);
    if (auth) {
      await refreshCpmsSites(auth);
    }
  };

  const downloadQrFromRef = (container: HTMLDivElement | null, fileBase: string) => {
    if (!container) {
      message.error('二维码未就绪，无法下载');
      return;
    }
    const safeName = fileBase.replace(/[^\w.-]+/g, '_');
    const padding = 32;
    const sourceCanvas = container.querySelector('canvas');
    if (sourceCanvas) {
      const w = sourceCanvas.width;
      const h = sourceCanvas.height;
      const outCanvas = document.createElement('canvas');
      outCanvas.width = w + padding * 2;
      outCanvas.height = h + padding * 2;
      const ctx = outCanvas.getContext('2d')!;
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, outCanvas.width, outCanvas.height);
      ctx.drawImage(sourceCanvas, padding, padding);
      const link = document.createElement('a');
      link.href = outCanvas.toDataURL('image/png');
      link.download = `${safeName}.png`;
      link.click();
      return;
    }
    const svg = container.querySelector('svg');
    if (svg) {
      const w = svg.getAttribute('width') ? Number(svg.getAttribute('width')) : 260;
      const h = svg.getAttribute('height') ? Number(svg.getAttribute('height')) : 260;
      const outCanvas = document.createElement('canvas');
      outCanvas.width = w + padding * 2;
      outCanvas.height = h + padding * 2;
      const ctx = outCanvas.getContext('2d')!;
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, outCanvas.width, outCanvas.height);
      const svgText = new XMLSerializer().serializeToString(svg);
      const blob = new Blob([svgText], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const img = new Image();
      img.onload = () => {
        ctx.drawImage(img, padding, padding, w, h);
        URL.revokeObjectURL(url);
        const link = document.createElement('a');
        link.href = outCanvas.toDataURL('image/png');
        link.download = `${safeName}.png`;
        link.click();
      };
      img.src = url;
      return;
    }
    message.error('二维码未就绪，无法下载');
  };

  const onDownloadInstitutionSfid = () => {
    if (!institutionSfidResult) {
      message.warning('请先生成机构身份识别码');
      return;
    }
    downloadQrFromRef(institutionQrRef.current, `institution-sfid-${institutionSfidResult.site_sfid}`);
  };

  const onDownloadInstitutionPreview = () => {
    if (!institutionQrPreview) return;
    downloadQrFromRef(institutionQrPreviewRef.current, `institution-sfid-${institutionQrPreview.site_sfid}`);
  };

  const openStatusScanner = () => {
    setOpScanType('status');
    setOpScanOpen(true);
  };

  const onScanBindQr4 = async (qrPayload: string) => {
    if (!auth) return;
    if (!qrPayload.trim()) {
      message.error('二维码识别失败');
      return;
    }
    setBindQr4ScanLoading(true);
    try {
      setBindQr4Payload(qrPayload);
      message.success('QR4 扫码成功，正在生成签名挑战...');
      setBindScannerActive(false);
      stopBindScanner();
      // 自动获取 challenge
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'QR4 扫码处理失败';
      message.error(msg);
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  const onBindPubkeyNext = async () => {
    if (!auth) return;
    if (!bindNewPubkey.trim()) {
      message.error('请输入新账户');
      return;
    }
    setBindChallengeLoading(true);
    try {
      const challenge = await citizenBindChallenge(auth);
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成签名挑战失败';
      message.error(msg);
    } finally {
      setBindChallengeLoading(false);
    }
  };

  const onScanBindSignature = async (raw: string) => {
    if (!auth || !bindChallenge) return;
    const trimmed = raw.trim();
    if (!trimmed) {
      message.error('签名二维码识别失败');
      return;
    }
    setBindQr4ScanLoading(true);
    try {
      const payload = parseKeyringSignedPayload(trimmed, bindChallenge.challenge_id);
      setBindSignature(payload.signature);
      setBindScannerActive(false);
      stopBindScanner();
      // 自动提交绑定
      const userAddress = bindMode === 'bind_pubkey' ? bindNewPubkey.trim() : (bindTargetPubkey || '');
      const result = await citizenBind(auth, {
        mode: bindMode,
        user_address: userAddress,
        qr4_payload: bindQr4Payload || undefined,
        citizen_id: bindTargetRecord?.id,
        challenge_id: payload.challenge_id,
        signature: payload.signature
      });
      message.success(`绑定成功${result.sfid_code ? `，SFID码：${result.sfid_code}` : ''}`);
      setBindModalOpen(false);
      await refreshList(auth, undefined, true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '绑定失败';
      message.error(msg);
    } finally {
      setBindQr4ScanLoading(false);
    }
  };

  const stopBindScanner = () => {
    if (bindScanCleanupRef.current) {
      bindScanCleanupRef.current();
      bindScanCleanupRef.current = null;
    }
    setBindScannerReady(false);
  };

  const onToggleBindScanner = () => {
    if (!bindModalOpen) return;
    if (bindScannerActive) {
      setBindScannerActive(false);
      stopBindScanner();
      return;
    }
    setBindScannerActive(true);
  };

  useEffect(() => {
    if (!bindModalOpen || !bindScannerActive || !bindVideoRef.current) {
      stopBindScanner();
      return;
    }
    const currentStep = bindStep;
    bindScanCleanupRef.current = startCameraScanner(
      bindVideoRef.current,
      (raw) => {
        setBindScannerActive(false);
        stopBindScanner();
        if (currentStep === 'scan_qr4') {
          void onScanBindQr4(raw);
        } else if (currentStep === 'scan_signature') {
          void onScanBindSignature(raw);
        }
      },
      () => setBindScannerReady(true),
      (msg) => { message.error(msg); setBindScannerActive(false); },
    );
    return () => stopBindScanner();
  }, [bindModalOpen, bindScannerActive, bindStep]);

  const openUnbindModal = (record: CitizenRow) => {
    setUnbindTarget(record);
    setUnbindChallenge(null);
    setUnbindStep('confirm');
    setUnbindScannerActive(false);
    stopUnbindScanner();
    setUnbindModalOpen(true);
  };

  const stopUnbindScanner = () => {
    if (unbindScanCleanupRef.current) {
      unbindScanCleanupRef.current();
      unbindScanCleanupRef.current = null;
    }
    setUnbindScannerReady(false);
  };

  const onUnbindGenerateChallenge = async () => {
    if (!auth) return;
    setUnbindChallengeLoading(true);
    try {
      const challenge = await citizenBindChallenge(auth);
      setUnbindChallenge(challenge);
      setUnbindStep('sign_challenge');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成解绑签名挑战失败';
      message.error(msg);
    } finally {
      setUnbindChallengeLoading(false);
    }
  };

  const onScanUnbindSignature = async (raw: string) => {
    if (!auth || !unbindChallenge || !unbindTarget) return;
    const trimmed = raw.trim();
    if (!trimmed) {
      message.error('签名二维码识别失败');
      return;
    }
    setUnbindSubmitting(true);
    try {
      const payload = parseKeyringSignedPayload(trimmed, unbindChallenge.challenge_id);
      setUnbindScannerActive(false);
      stopUnbindScanner();
      await citizenUnbind(auth, {
        citizen_id: unbindTarget.id,
        challenge_id: payload.challenge_id,
        signature: payload.signature
      });
      message.success('解绑成功');
      setUnbindModalOpen(false);
      await refreshList(auth, undefined, true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '解绑失败';
      message.error(msg);
    } finally {
      setUnbindSubmitting(false);
    }
  };

  useEffect(() => {
    if (!unbindModalOpen || !unbindScannerActive || !unbindVideoRef.current) {
      stopUnbindScanner();
      return;
    }
    unbindScanCleanupRef.current = startCameraScanner(
      unbindVideoRef.current,
      (raw) => { setUnbindScannerActive(false); stopUnbindScanner(); void onScanUnbindSignature(raw); },
      () => setUnbindScannerReady(true),
      (msg) => { message.error(msg); setUnbindScannerActive(false); },
    );
    return () => stopUnbindScanner();
  }, [unbindModalOpen, unbindScannerActive]);

  const citizenColumns: ColumnsType<CitizenRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v: unknown, _r: CitizenRow, idx: number) => idx + 1
    },
    {
      title: '账户',
      dataIndex: 'account_pubkey',
      align: 'center',
      render: (v: string | undefined) => (v ? tryEncodeSs58(v) : '-')
    },
    {
      title: '档案号',
      dataIndex: 'archive_no',
      align: 'center',
      render: (v: string | undefined) => v ?? '-'
    },
    {
      title: 'SFID码',
      dataIndex: 'sfid_code',
      align: 'center',
      render: (v: string | undefined) => v ?? '-'
    },
    {
      title: '状态',
      dataIndex: 'status',
      width: 100,
      align: 'center',
      render: (v: string) => {
        if (v === 'BOUND') return '已绑定';
        if (v === 'UNLINKED') return '已解绑';
        return '未绑定';
      }
    }
  ];
  if (capabilities.canBusinessWrite) {
    citizenColumns.push({
      title: '操作',
      width: 200,
      align: 'center',
      render: (_v: unknown, row: CitizenRow) => (
        <Space size={8}>
          {row.status === 'BOUND' ? (
            <Button danger onClick={() => openUnbindModal(row)}>
              解绑
            </Button>
          ) : (
            <Button type="primary" onClick={() => openBindModal(row)}>
              绑定
            </Button>
          )}
        </Space>
      )
    });
  }

  const institutionRows = cpmsSites;

  return (
    <Layout
      style={{
        minHeight: '100vh',
        background: 'linear-gradient(145deg, #0f172a 0%, #134e4a 40%, #0f766e 70%, #115e59 100%)',
        backgroundAttachment: 'fixed',
        position: 'relative'
      }}
    >
      {/* 背景装饰层 */}
      <div
        style={{
          position: 'fixed',
          inset: 0,
          pointerEvents: 'none',
          zIndex: 0,
          overflow: 'hidden'
        }}
      >
        {/* 右上光晕 */}
        <div
          style={{
            position: 'absolute',
            top: '-20%',
            right: '-10%',
            width: '50vw',
            height: '50vw',
            borderRadius: '50%',
            background: 'radial-gradient(circle, rgba(13,148,136,0.25) 0%, transparent 70%)',
          }}
        />
        {/* 左下光晕 */}
        <div
          style={{
            position: 'absolute',
            bottom: '-15%',
            left: '-10%',
            width: '45vw',
            height: '45vw',
            borderRadius: '50%',
            background: 'radial-gradient(circle, rgba(20,184,166,0.15) 0%, transparent 70%)',
          }}
        />
        {/* 网格纹理 */}
        <div
          style={{
            position: 'absolute',
            inset: 0,
            backgroundImage:
              'linear-gradient(rgba(255,255,255,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px)',
            backgroundSize: '60px 60px',
          }}
        />
        {/* 对角线装饰 */}
        <div
          style={{
            position: 'absolute',
            inset: 0,
            backgroundImage:
              'linear-gradient(135deg, transparent 48.5%, rgba(255,255,255,0.015) 48.5%, rgba(255,255,255,0.015) 51.5%, transparent 51.5%)',
            backgroundSize: '120px 120px',
          }}
        />
      </div>
      <Header
        style={{
          position: 'relative',
          zIndex: 1,
          background: 'linear-gradient(135deg, rgba(13,148,136,0.7) 0%, rgba(15,118,110,0.8) 100%)',
          backdropFilter: 'blur(12px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          paddingInline: 32,
          height: 72,
          borderBottom: '1px solid rgba(255,255,255,0.15)',
          boxShadow: '0 2px 16px rgba(0,0,0,0.12)'
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div
            style={{
              width: 44,
              height: 44,
              borderRadius: 10,
              background: 'rgba(255,255,255,0.18)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: 22,
              border: '1px solid rgba(255,255,255,0.25)'
            }}
          >
            <QrcodeOutlined style={{ color: '#fff' }} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.2 }}>
            <Typography.Text
              style={{
                color: '#ffffff',
                fontSize: 20,
                fontWeight: 700,
                letterSpacing: 2
              }}
            >
              中华民族联邦共和国
            </Typography.Text>
            <Typography.Text
              style={{
                color: 'rgba(255,255,255,0.8)',
                fontSize: 13,
                fontWeight: 500,
                letterSpacing: 4,
                marginTop: 2
              }}
            >
              身份识别码系统
            </Typography.Text>
          </div>
        </div>
        {auth && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <Typography.Text
              style={{
                color: '#ffffff',
                fontSize: 14,
                fontWeight: 500,
                background: 'rgba(255,255,255,0.12)',
                padding: '6px 16px',
                borderRadius: 8,
                border: '1px solid rgba(255,255,255,0.15)'
              }}
            >
              {resolveHeaderAdminName(auth)}
            </Typography.Text>
            <Button
              size="small"
              danger
              onClick={onLogout}
              style={{
                background: 'rgba(255,255,255,0.1)',
                borderColor: 'rgba(255,255,255,0.25)',
                color: '#fca5a5',
                fontWeight: 500,
                borderRadius: 8
              }}
            >
              退出
            </Button>
          </div>
        )}
      </Header>

      {bootstrapping ? (
        <Content style={{ position: 'relative', zIndex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
          <Card bordered={false} style={{ width: 520, maxWidth: '92vw' }} loading />
        </Content>
      ) : !auth ? (
        <Content
          style={{
            position: 'relative',
            zIndex: 1,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 24,
            minHeight: 'calc(100vh - 72px)'
          }}
        >
          <div
            style={{
              width: 780,
              maxWidth: '95vw',
              background: 'rgba(255,255,255,0.92)',
              backdropFilter: 'blur(20px)',
              borderRadius: 20,
              boxShadow: '0 8px 40px rgba(0,0,0,0.12), 0 1px 3px rgba(0,0,0,0.06)',
              border: '1px solid rgba(255,255,255,0.6)',
              overflow: 'hidden'
            }}
          >
            {/* 登录卡片顶部 */}
            <div
              style={{
                background: 'linear-gradient(135deg, #0d9488 0%, #0f766e 50%, #115e59 100%)',
                padding: '28px 32px',
                textAlign: 'center'
              }}
            >
              <div
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  width: 52,
                  height: 52,
                  borderRadius: 14,
                  background: 'rgba(255,255,255,0.18)',
                  marginBottom: 12,
                  border: '1px solid rgba(255,255,255,0.25)'
                }}
              >
                <QrcodeOutlined style={{ fontSize: 26, color: '#fff' }} />
              </div>
              <Typography.Title
                level={4}
                style={{ color: '#fff', margin: 0, fontWeight: 600, letterSpacing: 1 }}
              >
                管理员扫码登录
              </Typography.Title>
              <Typography.Text style={{ color: 'rgba(255,255,255,0.7)', fontSize: 13 }}>
                使用 公民 移动端扫描二维码完成身份验证
              </Typography.Text>
            </div>

            {/* 登录内容区域 */}
            <div style={{ padding: '32px 36px 36px' }}>
              <div style={{ display: 'flex', gap: 32, alignItems: 'stretch', flexWrap: 'wrap' }}>
                {/* 左侧：QR 码生成 */}
                <div
                  style={{
                    flex: '1 1 300px',
                    minWidth: 280,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center'
                  }}
                >
                  <Typography.Text
                    strong
                    style={{ fontSize: 14, marginBottom: 16, color: '#374151' }}
                  >
                    登录二维码
                  </Typography.Text>
                  <div
                    style={{
                      position: 'relative',
                      width: 260,
                      height: 260,
                      background: '#f8fffe',
                      borderRadius: 16,
                      border: '2px solid #e6f7f5',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      overflow: 'hidden'
                    }}
                  >
                    <div style={{ position: 'absolute', top: 0, left: 0, width: 20, height: 20, borderTop: '3px solid #0d9488', borderLeft: '3px solid #0d9488', borderTopLeftRadius: 8 }} />
                    <div style={{ position: 'absolute', top: 0, right: 0, width: 20, height: 20, borderTop: '3px solid #0d9488', borderRight: '3px solid #0d9488', borderTopRightRadius: 8 }} />
                    <div style={{ position: 'absolute', bottom: 0, left: 0, width: 20, height: 20, borderBottom: '3px solid #0d9488', borderLeft: '3px solid #0d9488', borderBottomLeftRadius: 8 }} />
                    <div style={{ position: 'absolute', bottom: 0, right: 0, width: 20, height: 20, borderBottom: '3px solid #0d9488', borderRight: '3px solid #0d9488', borderBottomRightRadius: 8 }} />
                    <div
                      style={{
                        filter: pendingQrLogin ? 'none' : 'blur(3px) opacity(0.4)',
                        transition: 'filter 0.3s ease'
                      }}
                    >
                      <QRCode
                        value={pendingQrLogin?.login_qr_payload || 'SFID_LOGIN_PENDING'}
                        size={228}
                        color="#134e4a"
                      />
                    </div>
                  </div>
                  <div style={{ marginTop: 14, textAlign: 'center' }}>
                    <Typography.Text
                      type="secondary"
                      style={{ fontSize: 12, display: 'block', marginBottom: 12 }}
                    >
                      {pendingQrLogin
                        ? `有效期至 ${new Date(pendingQrLogin.expire_at * 1000).toLocaleTimeString()}`
                        : '请点击按钮生成二维码'}
                    </Typography.Text>
                    <Button
                      type="primary"
                      size="large"
                      onClick={onCreateQrLogin}
                      loading={challengeLoading}
                      style={{
                        borderRadius: 10,
                        fontWeight: 500,
                        width: 200,
                        boxShadow: '0 2px 8px rgba(13,148,136,0.3)'
                      }}
                    >
                      {pendingQrLogin ? '重新生成' : '生成二维码'}
                    </Button>
                  </div>
                </div>

                {/* 分割线 */}
                <div
                  style={{
                    width: 1,
                    background: 'linear-gradient(to bottom, transparent, #e5e7eb, transparent)',
                    alignSelf: 'stretch',
                    margin: '0 4px'
                  }}
                />

                {/* 右侧：摄像头扫码 */}
                <div
                  style={{
                    flex: '1 1 300px',
                    minWidth: 280,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center'
                  }}
                >
                  <Typography.Text
                    strong
                    style={{ fontSize: 14, marginBottom: 16, color: '#374151' }}
                  >
                    扫码窗口
                  </Typography.Text>
                  <div
                    style={{
                      width: 260,
                      height: 260,
                      background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                      borderRadius: 16,
                      overflow: 'hidden',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      position: 'relative',
                      border: '2px solid #334155',
                      boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
                    }}
                  >
                    <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                    <video
                      ref={videoRef}
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                      muted
                      playsInline
                    />
                    {!scannerReady && (
                      <div
                        style={{
                          position: 'absolute',
                          inset: 0,
                          display: 'flex',
                          flexDirection: 'column',
                          alignItems: 'center',
                          justifyContent: 'center',
                          gap: 8
                        }}
                      >
                        <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                        <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                          {scannerActive ? '摄像头初始化中...' : '等待开启摄像头'}
                        </Typography.Text>
                      </div>
                    )}
                  </div>
                  <div style={{ marginTop: 14, textAlign: 'center' }}>
                    <Typography.Text
                      type="secondary"
                      style={{ fontSize: 12, display: 'block', marginBottom: 12 }}
                    >
                      开启摄像头扫描已签名的二维码
                    </Typography.Text>
                    <Button
                      size="large"
                      onClick={onToggleScanner}
                      disabled={scanSubmitting}
                      style={{
                        borderRadius: 10,
                        fontWeight: 500,
                        width: 200
                      }}
                    >
                      {scannerActive ? '停止扫码' : '开启扫码'}
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Content>
      ) : (
        <Content style={{ position: 'relative', zIndex: 1, padding: '16px 24px 24px' }}>
          <div
              style={{
                display: 'flex',
                gap: 6,
                marginBottom: 20,
                padding: '8px 12px',
                background: 'rgba(255,255,255,0.08)',
                backdropFilter: 'blur(12px)',
                borderRadius: 14,
                border: '1px solid rgba(255,255,255,0.1)',
                width: 'fit-content'
              }}
            >
              {([
                { key: 'citizens' as const, label: '首页', visible: true, onClick: () => setActiveView('citizens') },
                {
                  key: 'institutions' as const,
                  label: '公权机构',
                  visible: capabilities.canViewInstitutions,
                  onClick: async () => {
                    setActiveView('institutions');
                    if (auth) {
                      await refreshCpmsSites(auth);
                    }
                  }
                },
                {
                  key: 'multisig' as const,
                  label: '多签管理',
                  visible: capabilities.canViewMultisig,
                  onClick: async () => {
                    setActiveView('multisig');
                    setSelectedMultisigSfid(null);
                    if (auth) {
                      await refreshMultisigSfids(auth);
                    }
                  }
                },
                {
                  key: 'keyring' as const,
                  label: '密钥管理',
                  visible: capabilities.canViewKeyring,
                  onClick: async () => {
                    setActiveView('keyring');
                    if (auth) {
                      await refreshKeyringState(auth);
                    }
                  }
                },
                {
                  key: 'system-settings' as const,
                  label: '机构管理',
                  visible: capabilities.canViewSystemSettings,
                  onClick: async () => {
                    setActiveView('system-settings');
                    if (!auth) return;
                    await openSystemSettings(auth);
                  }
                },
              ] as const).filter((tab) => tab.visible).map((tab) => (
                <button
                  key={tab.key}
                  onClick={tab.onClick}
                  style={{
                    padding: '8px 20px',
                    borderRadius: 10,
                    border: 'none',
                    cursor: 'pointer',
                    fontSize: 14,
                    fontWeight: 500,
                    transition: 'all 0.2s ease',
                    ...(activeView === tab.key
                      ? {
                          background: 'linear-gradient(135deg, #0d9488, #0f766e)',
                          color: '#fff',
                          boxShadow: '0 2px 8px rgba(13,148,136,0.35)'
                        }
                      : {
                          background: 'transparent',
                          color: 'rgba(255,255,255,0.7)'
                        })
                  }}
                >
                  {tab.label}
                </button>
              ))}
            </div>
          {activeView === 'operators' && capabilities.canViewSystemAdmins ? (
            <Card
              title="系统管理员列表"
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              extra={
                capabilities.canCrudSystemAdmins ? (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        flexWrap: 'nowrap',
                        gap: 8,
                        width: addOperatorOpen ? 860 : 0,
                        opacity: addOperatorOpen ? 1 : 0,
                        overflow: 'hidden',
                        transform: `translateX(${addOperatorOpen ? 0 : 12}px)`,
                        transition: 'all 0.2s ease'
                      }}
                    >
                      <Button
                        type="link"
                        onClick={() => {
                          addOperatorForm.resetFields();
                          setAddOperatorOpen(false);
                        }}
                      >
                        取消新增
                      </Button>
                      <Form
                        form={addOperatorForm}
                        layout="inline"
                        onFinish={onCreateOperator}
                        style={{ display: 'flex', flexWrap: 'nowrap', alignItems: 'center', gap: 8 }}
                      >
                        <Form.Item
                          name="operator_name"
                          rules={[{ required: true, message: '请输入系统管理员姓名' }]}
                          style={{ marginBottom: 0 }}
                        >
                          <Input style={{ width: 180 }} placeholder="请输入系统管理员姓名" />
                        </Form.Item>
                        <Form.Item
                          name="operator_pubkey"
                          rules={[
                            { required: true, message: '请输入系统管理员公钥' },
                            {
                              validator: async (_rule, value) => {
                                if (!value || isSr25519HexPubkey(String(value))) return;
                                throw new Error('公钥格式必须为 32 字节十六进制');
                              }
                            }
                          ]}
                          style={{ marginBottom: 0 }}
                        >
                          <Input style={{ width: 520 }} placeholder="请输入系统管理员公钥" />
                        </Form.Item>
                      </Form>
                    </div>
                    <Button
                      type="primary"
                      loading={addOperatorLoading}
                      onClick={() => {
                        if (!addOperatorOpen) {
                          setAddOperatorOpen(true);
                          return;
                        }
                        addOperatorForm.submit();
                      }}
                    >
                      {addOperatorOpen ? '确认新增' : '新增系统管理员'}
                    </Button>
                  </div>
                ) : null
              }
            >
              <Table<OperatorRow>
                rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
                loading={operatorsLoading}
                dataSource={operators}
                pagination={{
                  pageSize: 10,
                  current: operatorPage,
                  onChange: (page) => setOperatorPage(page)
                }}
                columns={[
                  {
                    title: '序号',
                    width: 80,
                    align: 'center',
                    render: (_v, _row, index) => (operatorPage - 1) * 10 + index + 1
                  },
                  { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
                  { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
                  { title: '状态', dataIndex: 'status', width: 120, align: 'center' },
                  {
                    title: '创建者',
                    align: 'center',
                    render: (_v, row) => row.created_by_name || row.created_by || '-'
                  },
                  ...(capabilities.canCrudSystemAdmins
                    ? [
                        {
                          title: '操作',
                          width: 220,
                          align: 'center' as const,
                          render: (_v: unknown, row: OperatorRow) => (
                            <Space>
                              <Button size="small" onClick={() => onUpdateOperator(row)}>
                                修改
                              </Button>
                              <Button size="small" onClick={() => onToggleOperatorStatus(row)}>
                                {row.status === 'ACTIVE' ? '停用' : '启用'}
                              </Button>
                              <Button size="small" danger onClick={() => onDeleteOperator(row)}>
                                删除
                              </Button>
                            </Space>
                          )
                        }
                      ]
                    : [])
                ]}
              />
            </Card>
          ) : activeView === 'super-admins' && capabilities.canViewInstitutionAdmins ? (
            <Card
              title="机构管理员列表"
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              extra={
                capabilities.canReplaceSuperAdmins ? (
                  <Form
                    form={replaceSuperForm}
                    layout="inline"
                    onFinish={onReplaceSuperAdmin}
                    style={{ rowGap: 8 }}
                  >
                    <Form.Item
                      name="province"
                      rules={[{ required: true, message: '请选择省份' }]}
                      style={{ marginBottom: 0 }}
                    >
                      <Select
                        style={{ width: 160 }}
                        placeholder="选择省份"
                        options={superAdmins.map((item) => ({ value: item.province, label: item.province }))}
                      />
                    </Form.Item>
                    <Form.Item
                      name="admin_pubkey"
                      rules={[
                        { required: true, message: '请输入新机构管理员公钥' },
                        {
                          validator: async (_rule, value) => {
                            if (!value || isSr25519HexPubkey(String(value))) return;
                            throw new Error('公钥格式必须为 32 字节十六进制');
                          }
                        }
                      ]}
                      style={{ marginBottom: 0 }}
                    >
                      <Input style={{ width: 420, maxWidth: '60vw' }} placeholder="新机构管理员公钥" />
                    </Form.Item>
                    <Form.Item style={{ marginBottom: 0 }}>
                      <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
                        更换机构管理员
                      </Button>
                    </Form.Item>
                  </Form>
                ) : null
              }
            >
              <Table<SuperAdminRow>
                rowKey={(r) => `${r.province}-${r.admin_pubkey}`}
                loading={superAdminsLoading}
                dataSource={superAdmins}
                pagination={{ pageSize: 10 }}
                columns={[
                  {
                    title: '序号',
                    width: 80,
                    align: 'center',
                    render: (_v: unknown, _row: SuperAdminRow, index: number) => index + 1
                  },
                  { title: '省份', dataIndex: 'province', align: 'center', width: 140 },
                  { title: '名称', dataIndex: 'admin_name', align: 'center', width: 180 },
                  { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
                  { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
                  {
                    title: '类型',
                    width: 100,
                    align: 'center',
                    render: (_v: unknown, row: SuperAdminRow) => row.built_in ? '内置' : '自定义'
                  }
                ]}
              />
            </Card>
          ) : activeView === 'institutions' && capabilities.canManageInstitutions ? (
            <Card
              title="公权机构列表"
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              extra={
                capabilities.canRegisterInstitutions ? (
                  <Space>
                    <Button type="primary" onClick={openInstitutionSfidModal} loading={institutionSfidLoading}>
                      新增公权机构
                    </Button>
                  </Space>
                ) : null
              }
            >
              <Table<CpmsSiteRow>
                rowKey={(r) => r.site_sfid}
                loading={cpmsSitesLoading}
                dataSource={institutionRows}
                pagination={{ pageSize: 10 }}
                columns={[
                  {
                    title: '身份识别码',
                    dataIndex: 'site_sfid',
                    width: 260,
                    align: 'center'
                  },
                  {
                    title: '安装令牌',
                    dataIndex: 'install_token_status',
                    width: 100,
                    align: 'center',
                    render: (v: string) => {
                      if (v === 'PENDING') return '待使用';
                      if (v === 'USED') return '已使用';
                      if (v === 'REVOKED') return '已撤销';
                      return v || '-';
                    }
                  },
                  {
                    title: '状态',
                    width: 90,
                    align: 'center',
                    render: (_v, row) => {
                      const s = row.status || 'PENDING';
                      if (s === 'PENDING') return '待录入';
                      if (s === 'ACTIVE') return '正常';
                      if (s === 'DISABLED') return '已禁用';
                      if (s === 'REVOKED') return '已撤销';
                      return s;
                    }
                  },
                  {
                    title: '所属机构',
                    align: 'center',
                    render: (_v, row) => {
                      const province = row.admin_province || '-';
                      const city = (row as CpmsSiteRow).city_name || '-';
                      const inst = institutionCodeToName((row as CpmsSiteRow).institution_code || '');
                      return `${province}/${city}/${inst}`;
                    }
                  },
                  {
                    title: '机构名称',
                    align: 'center',
                    width: 160,
                    render: (_v, row) => (row as CpmsSiteRow).institution_name || '-'
                  },
                  {
                    title: '登记人',
                    align: 'center',
                    width: 160,
                    render: (_v, row) => (row as CpmsSiteRow).created_by_name || `${row.admin_province || ''}管理员`
                  },
                  {
                    title: '二维码',
                    width: 80,
                    align: 'center',
                    render: (_v, row) => {
                      const payload = (row as CpmsSiteRow).qr1_payload;
                      if (!payload) return '-';
                      return (
                        <Button
                          size="small"
                          type="text"
                          icon={<QrcodeOutlined />}
                          onClick={() => setInstitutionQrPreview({ site_sfid: row.site_sfid, qr1_payload: payload })}
                        />
                      );
                    }
                  },
                  {
                    title: '操作',
                    width: 80,
                    align: 'center',
                    render: (_v, row) => {
                      const status = row.status || 'PENDING';
                      const isDisabled = status === 'DISABLED';
                      const isRevoked = status === 'REVOKED';
                      const items = [
                        {
                          key: 'disable',
                          label: isDisabled ? '已禁用' : '禁用',
                          disabled: isDisabled || isRevoked,
                          onClick: () => onDisableCpmsSite(row),
                        },
                        {
                          key: 'delete',
                          label: '删除',
                          danger: true,
                          onClick: () => onDeleteCpmsSite(row),
                        },
                        {
                          key: 'scan',
                          label: '扫码登记',
                          disabled: status !== 'PENDING',
                          onClick: () => openRegisterScanner(),
                        },
                      ];
                      return (
                        <Dropdown menu={{ items }} trigger={['click']}>
                          <Button size="small" icon={<MoreOutlined />} />
                        </Dropdown>
                      );
                    }
                  }
                ]}
              />
            </Card>
          ) : activeView === 'multisig' && capabilities.canViewMultisig ? (
            selectedMultisigSfid ? (
              // ── 多签 SFID 详情页 ──
              (() => {
                const r = selectedMultisigSfid;
                const statusColor: Record<string, string> = { REGISTERED: 'green', PENDING: 'orange', FAILED: 'red' };
                const statusLabel: Record<string, string> = { REGISTERED: '已注册', PENDING: '等待中', FAILED: '失败' };
                return (
                  <Card
                    bordered={false}
                    style={glassCardStyle}
                    headStyle={glassCardHeadStyle}
                    title={
                      <Space>
                        <Button type="link" onClick={() => setSelectedMultisigSfid(null)}>
                          ← 返回列表
                        </Button>
                        <span>注册机构 SFID 详情</span>
                      </Space>
                    }
                  >
                    <div style={{ display: 'grid', gridTemplateColumns: '140px 1fr', rowGap: 10, columnGap: 16 }}>
                      <Typography.Text type="secondary">SFID 号</Typography.Text>
                      <Typography.Text code style={{ wordBreak: 'break-all' }}>{r.site_sfid}</Typography.Text>

                      <Typography.Text type="secondary">A3 主体属性</Typography.Text>
                      <Typography.Text>{r.a3}</Typography.Text>

                      <Typography.Text type="secondary">机构类型</Typography.Text>
                      <Typography.Text>{r.institution_code}</Typography.Text>

                      <Typography.Text type="secondary">机构名称</Typography.Text>
                      <Typography.Text>{r.institution_name}</Typography.Text>

                      <Typography.Text type="secondary">省</Typography.Text>
                      <Typography.Text>{r.province}（{r.province_code}）</Typography.Text>

                      <Typography.Text type="secondary">市</Typography.Text>
                      <Typography.Text>{r.city}</Typography.Text>

                      <Typography.Text type="secondary">链上状态</Typography.Text>
                      <Typography.Text>
                        <Tag color={statusColor[r.chain_status] || 'default'}>{statusLabel[r.chain_status] || r.chain_status}</Tag>
                      </Typography.Text>

                      <Typography.Text type="secondary">交易哈希</Typography.Text>
                      <Typography.Text code style={{ wordBreak: 'break-all' }}>{r.chain_tx_hash || '-'}</Typography.Text>

                      <Typography.Text type="secondary">区块高度</Typography.Text>
                      <Typography.Text>{r.chain_block_number ?? '-'}</Typography.Text>

                      <Typography.Text type="secondary">创建者</Typography.Text>
                      <Typography.Text>{r.created_by_name || r.created_by}</Typography.Text>

                      <Typography.Text type="secondary">创建时间</Typography.Text>
                      <Typography.Text>{r.created_at ? new Date(r.created_at).toLocaleString('zh-CN') : '-'}</Typography.Text>
                    </div>
                  </Card>
                );
              })()
            ) : (
              <Card
                title="注册机构列表"
                bordered={false}
                style={glassCardStyle}
                headStyle={glassCardHeadStyle}
                extra={
                  <Button type="primary" onClick={openMultisigModal}>
                    生成机构SFID
                  </Button>
                }
              >
                <Table<MultisigSfidRow>
                  rowKey={(r) => r.site_sfid}
                  loading={multisigLoading}
                  dataSource={multisigRows}
                  pagination={{
                    pageSize: 10,
                    current: multisigPage,
                    onChange: (page) => setMultisigPage(page)
                  }}
                  onRow={(row) => ({
                    onClick: (e) => {
                      // 点击操作列里的按钮时不进入详情
                      const target = e.target as HTMLElement;
                      if (target.closest('button')) return;
                      setSelectedMultisigSfid(row);
                    },
                    style: { cursor: 'pointer' },
                  })}
                  columns={[
                    {
                      title: '序号',
                      width: 70,
                      align: 'center',
                      render: (_v, _row, index) => (multisigPage - 1) * 10 + index + 1
                    },
                    { title: 'SFID 号', dataIndex: 'site_sfid', width: 280 },
                    { title: '机构名称', dataIndex: 'institution_name', width: 160, align: 'center' },
                    { title: '省', dataIndex: 'province', width: 100, align: 'center' },
                    { title: '市', dataIndex: 'city', width: 100, align: 'center' },
                    {
                      title: '链上状态',
                      dataIndex: 'chain_status',
                      width: 110,
                      align: 'center',
                      render: (status: string) => {
                        const colorMap: Record<string, string> = { REGISTERED: 'green', PENDING: 'orange', FAILED: 'red' };
                        const labelMap: Record<string, string> = { REGISTERED: '已注册', PENDING: '等待中', FAILED: '失败' };
                        return <Tag color={colorMap[status] || 'default'}>{labelMap[status] || status}</Tag>;
                      }
                    },
                    { title: '创建者', dataIndex: 'created_by_name', width: 120, align: 'center' },
                    {
                      title: '操作',
                      width: 100,
                      align: 'center',
                      render: (_v: unknown, row: MultisigSfidRow) => {
                        const canDelete = row.chain_status !== 'REGISTERED';
                        return (
                          <Button
                            size="small"
                            danger
                            disabled={!canDelete}
                            onClick={(e) => {
                              e.stopPropagation();
                              onDeleteMultisigSfid(row);
                            }}
                          >
                            删除
                          </Button>
                        );
                      }
                    },
                  ]}
                />
              </Card>
            )
          ) : activeView === 'keyring' && capabilities.canManageKeyring ? (
            <Card
              title="签名密钥管理（一主两备）"
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              extra={
                <Button
                  onClick={() => {
                    if (auth) {
                      void refreshKeyringState(auth);
                    }
                  }}
                  loading={keyringLoading}
                >
                  刷新状态
                </Button>
              }
            >
              {(() => {
                // 主密钥登录时，一切轮换相关控件（输入框/按钮/扫码图标）都禁用
                const isMainKeySigned = Boolean(
                  keyringState &&
                    auth &&
                    auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() ===
                      keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase()
                );
                return (
              <Form
                form={keyringForm}
                layout="inline"
                onFinish={onCreateKeyringRotateChallenge}
                style={{ marginBottom: 12, rowGap: 8 }}
              >
                <Form.Item
                  name="new_backup_pubkey"
                  rules={[
                    { required: true, message: '请输入新备用账户' },
                    {
                      validator: async (_rule, value) => {
                        if (!value) return;
                        try {
                          decodeSs58(String(value));
                        } catch (err) {
                          throw new Error(err instanceof Error ? err.message : '账户格式无效');
                        }
                      }
                    }
                  ]}
                >
                  <Input
                    style={{ width: 420, maxWidth: '72vw' }}
                    placeholder="新备用账户（SS58 地址）"
                    disabled={isMainKeySigned}
                    suffix={
                      <span
                        title={isMainKeySigned ? '主密钥登录无法轮换' : '扫码识别用户码'}
                        style={{
                          cursor: isMainKeySigned ? 'not-allowed' : 'pointer',
                          display: 'inline-flex',
                          color: isMainKeySigned ? 'rgba(148,163,184,0.6)' : '#0d9488'
                        }}
                        onClick={() => {
                          if (isMainKeySigned) return;
                          setKeyringScanAccountOpen(true);
                        }}
                      >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>
                          <rect x="7" y="7" width="10" height="10" rx="1"/>
                        </svg>
                      </span>
                    }
                  />
                </Form.Item>
                <Form.Item style={{ marginBottom: 0 }}>
                  <Button
                    type="primary"
                    htmlType="submit"
                    loading={keyringActionLoading}
                    disabled={isMainKeySigned}
                  >
                    发起轮换
                  </Button>
                </Form.Item>
              </Form>
                );
              })()}

              <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
                {'流程：输入新备用账户 -> 生成轮换二维码 -> 备用钱包扫码签名 -> 本页面扫码验签 -> 自动完成轮换并推链。'}
              </Typography.Paragraph>

              <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start', flexWrap: 'wrap', marginBottom: 12 }}>
                <div style={{ flex: '1 1 320px', minWidth: 300 }}>
                  <Typography.Text strong style={{ fontSize: 14, color: '#374151', display: 'block', marginBottom: 8 }}>轮换二维码</Typography.Text>
                  <div style={{ display: 'flex', justifyContent: 'center' }}>
                    <div style={{
                      filter: keyringChallenge ? 'none' : 'blur(3px) opacity(0.4)',
                      transition: 'filter 0.3s ease'
                    }}>
                      <QRCode value={keyringChallenge?.challenge_text || 'SFID_KEYRING_ROTATE_PENDING'} size={260} color="#134e4a" bordered={false} />
                    </div>
                  </div>
                  <Typography.Paragraph type="secondary" style={{ marginTop: 10, marginBottom: 8 }}>
                    {keyringChallenge
                      ? `轮换挑战有效期至：${new Date(keyringChallenge.expire_at * 1000).toLocaleTimeString()}`
                      : ''}
                  </Typography.Paragraph>
                </div>
                <div style={{ flex: '1 1 260px', minWidth: 260 }}>
                  <Typography.Text strong style={{ fontSize: 14, color: '#374151', display: 'block', marginBottom: 8 }}>扫码窗口</Typography.Text>
                  <div
                    style={{
                      width: 260,
                      height: 260,
                      boxSizing: 'border-box',
                      background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                      borderRadius: 16,
                      overflow: 'hidden',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      position: 'relative',
                      border: '2px solid #334155',
                      boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
                    }}
                  >
                    <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                    <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                    <video
                      ref={keyringVideoRef}
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                      muted
                      playsInline
                    />
                    {!keyringScannerReady && (
                      <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
                        <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                        <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                          {keyringScannerActive ? '摄像头初始化中...' : '等待开启摄像头'}
                        </Typography.Text>
                      </div>
                    )}
                  </div>
                  <div style={{ marginTop: 12 }}>
                    <Button onClick={onToggleKeyringScanner} disabled={keyringScanSubmitting} style={{ borderRadius: 10 }}>
                      {keyringScannerActive ? '停止扫码' : '开启扫码'}
                    </Button>
                  </div>
                </div>
              </div>


              <Card
                size="small"
                loading={keyringLoading}
                style={{
                  background: '#f0fdfa',
                  borderRadius: 12,
                  borderLeft: '3px solid #0d9488',
                  border: '1px solid #ccfbf1'
                }}
              >
                <Typography.Text strong style={{ fontSize: 13, color: '#134e4a', display: 'block', marginBottom: 10 }}>
                  当前密钥状态
                </Typography.Text>
                <Typography.Paragraph style={{ marginBottom: 6 }}>
                  主账户：<Typography.Text code>{tryEncodeSs58(keyringState?.main_pubkey)}</Typography.Text>
                  {mainAccountBalance != null && (
                    <span style={{ marginLeft: 12, color: '#0d9488', fontWeight: 600 }}>
                      余额：{mainAccountBalance} 元
                    </span>
                  )}
                  {mainAccountBalanceError && (
                    <span style={{ marginLeft: 12, color: '#ef4444', fontSize: 12 }}>
                      余额查询失败：{mainAccountBalanceError}
                    </span>
                  )}
                </Typography.Paragraph>
                <Typography.Paragraph style={{ marginBottom: 6 }}>
                  备用A 账户：<Typography.Text code>{tryEncodeSs58(keyringState?.backup_a_pubkey)}</Typography.Text>
                </Typography.Paragraph>
                <Typography.Paragraph style={{ marginBottom: 0 }}>
                  备用B 账户：<Typography.Text code>{tryEncodeSs58(keyringState?.backup_b_pubkey)}</Typography.Text>
                </Typography.Paragraph>
              </Card>
            </Card>
          ) : activeView === 'system-settings' && capabilities.canViewSystemSettings ? (
            selectedSuperAdmin ? (
              // ── 机构详情页（sub-tab：系统管理员列表 / 机构管理员） ──
              (() => {
                const isKeyAdmin = auth?.role === 'KEY_ADMIN';
                const isSelf = auth ? sameHexPubkey(selectedSuperAdmin.admin_pubkey, auth.admin_pubkey) : false;
                const canEditOperators = isKeyAdmin || (auth?.role === 'INSTITUTION_ADMIN' && isSelf);
                const canReplaceThisAdmin = isKeyAdmin;
                const operatorsForThisAdmin = operators.filter((op) =>
                  sameHexPubkey(op.created_by, selectedSuperAdmin.admin_pubkey)
                );
                const subTabs: Array<{ key: 'operators' | 'super-admin'; label: string }> = [
                  { key: 'operators', label: '系统管理员列表' },
                  { key: 'super-admin', label: '机构管理员' },
                ];
                return (
                  <Card
                    bordered={false}
                    style={glassCardStyle}
                    headStyle={glassCardHeadStyle}
                    title={
                      <Space>
                        {isKeyAdmin && (
                          <Button type="link" onClick={() => setSelectedSuperAdmin(null)}>
                            ← 返回列表
                          </Button>
                        )}
                        <span>机构详情 — {selectedSuperAdmin.province}</span>
                      </Space>
                    }
                  >
                    {/* sub-tabs：左 系统管理员列表（默认） / 右 机构管理员
                       外层卡片是白底玻璃质感，未选中按钮需用深色文字 + 浅底边框才能被看见 */}
                    <div style={{
                      display: 'flex',
                      gap: 8,
                      padding: 6,
                      background: 'rgba(15,23,42,0.06)',
                      borderRadius: 10,
                      border: '1px solid rgba(15,23,42,0.12)',
                      width: 'fit-content',
                      marginBottom: 16,
                    }}>
                      {subTabs.map((t) => (
                        <button
                          key={t.key}
                          onClick={() => setAdminDetailTab(t.key)}
                          style={{
                            padding: '6px 18px',
                            borderRadius: 8,
                            border: 'none',
                            cursor: 'pointer',
                            fontSize: 13,
                            fontWeight: 500,
                            transition: 'all 0.2s ease',
                            ...(adminDetailTab === t.key
                              ? {
                                  background: 'linear-gradient(135deg, #0d9488, #0f766e)',
                                  color: '#fff',
                                  boxShadow: '0 2px 6px rgba(13,148,136,0.35)',
                                }
                              : {
                                  background: 'rgba(255,255,255,0.7)',
                                  color: 'rgba(15,23,42,0.75)',
                                }),
                          }}
                        >
                          {t.label}
                        </button>
                      ))}
                    </div>

                    {adminDetailTab === 'operators' ? (
                      // ── 系统管理员列表 ──
                      <Card
                        type="inner"
                        title="系统管理员列表"
                        extra={
                          canEditOperators ? (
                            <Button type="primary" onClick={() => setAddOperatorOpen(true)}>
                              新增系统管理员
                            </Button>
                          ) : null
                        }
                      >
                        <Table<OperatorRow>
                          rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
                          loading={operatorsLoading}
                          dataSource={operatorsForThisAdmin}
                          pagination={{
                            pageSize: 10,
                            current: operatorListPage,
                            onChange: (page) => setOperatorListPage(page),
                            showSizeChanger: false,
                            showTotal: (total) => `共 ${total} 条`,
                          }}
                          columns={[
                            {
                              title: '序号',
                              width: 70,
                              align: 'center',
                              render: (_v, _row, index) => (operatorListPage - 1) * 10 + index + 1,
                            },
                            { title: '市', dataIndex: 'city', align: 'center', width: 120 },
                            { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
                            {
                              title: '账户',
                              dataIndex: 'admin_pubkey',
                              align: 'center',
                              render: (v: string) => tryEncodeSs58(v),
                            },
                            { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
                            ...(canEditOperators
                              ? [
                                  {
                                    title: '操作',
                                    width: 220,
                                    align: 'center' as const,
                                    render: (_v: unknown, row: OperatorRow) => (
                                      <Space>
                                        <Button size="small" onClick={() => onUpdateOperator(row)}>
                                          修改
                                        </Button>
                                        <Button size="small" onClick={() => onToggleOperatorStatus(row)}>
                                          {row.status === 'ACTIVE' ? '停用' : '启用'}
                                        </Button>
                                        <Button size="small" danger onClick={() => onDeleteOperator(row)}>
                                          删除
                                        </Button>
                                      </Space>
                                    ),
                                  },
                                ]
                              : []),
                          ]}
                        />
                      </Card>
                    ) : (
                      // ── 机构管理员（基本信息 + 更换） ──
                      <Card
                        type="inner"
                        title="机构管理员"
                        extra={
                          canReplaceThisAdmin ? (
                            <Form
                              form={replaceSuperForm}
                              layout="inline"
                              onFinish={(values: { admin_pubkey: string }) =>
                                onReplaceSuperAdmin({ province: selectedSuperAdmin.province, admin_pubkey: values.admin_pubkey })
                              }
                              style={{ rowGap: 8 }}
                            >
                              <Form.Item
                                name="admin_pubkey"
                                rules={[
                                  { required: true, message: '请输入新机构管理员账户' },
                                  {
                                    validator: async (_rule, value) => {
                                      if (!value) return;
                                      try {
                                        decodeSs58(String(value));
                                      } catch (e) {
                                        throw new Error(e instanceof Error ? e.message : '账户格式无效');
                                      }
                                    },
                                  },
                                ]}
                                style={{ marginBottom: 0 }}
                              >
                                <Input
                                  style={{ width: 420, maxWidth: '60vw' }}
                                  placeholder="新机构管理员账户（SS58）"
                                  suffix={
                                    <span
                                      title="扫码识别用户码"
                                      style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                                      onClick={() => setAccountScanTarget('super-admin')}
                                    >
                                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                        <path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>
                                        <rect x="7" y="7" width="10" height="10" rx="1"/>
                                      </svg>
                                    </span>
                                  }
                                />
                              </Form.Item>
                              <Form.Item style={{ marginBottom: 0 }}>
                                <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
                                  更换机构管理员
                                </Button>
                              </Form.Item>
                            </Form>
                          ) : null
                        }
                      >
                        <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 8, columnGap: 12 }}>
                          <Typography.Text type="secondary">省份</Typography.Text>
                          <Typography.Text>{selectedSuperAdmin.province}</Typography.Text>
                          <Typography.Text type="secondary">名称</Typography.Text>
                          <Typography.Text>{selectedSuperAdmin.admin_name}</Typography.Text>
                          <Typography.Text type="secondary">账户</Typography.Text>
                          <Typography.Text code style={{ wordBreak: 'break-all' }}>
                            {tryEncodeSs58(selectedSuperAdmin.admin_pubkey)}
                          </Typography.Text>
                        </div>
                      </Card>
                    )}
                  </Card>
                );
              })()
            ) : (
              // ── KeyAdmin: 机构管理员卡片列表 ──
              <Card
                title="机构列表"
                bordered={false}
                style={glassCardStyle}
                headStyle={glassCardHeadStyle}
              >
                {superAdminsLoading ? (
                  <Typography.Text type="secondary">加载中...</Typography.Text>
                ) : (
                  <div style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fill, minmax(210px, 1fr))',
                    gap: 12
                  }}>
                    {superAdmins.map((row) => (
                      <div
                        key={`${row.province}-${row.admin_pubkey}`}
                        onClick={() => setSelectedSuperAdmin(row)}
                        style={{
                          padding: 14,
                          borderRadius: 12,
                          // 更浅的卡片底色，避免在玻璃质感整体面板上过于显眼
                          border: '1px solid rgba(15,23,42,0.22)',
                          background: 'rgba(226,232,240,0.55)',
                          boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
                          cursor: 'pointer',
                          transition: 'all 0.2s ease',
                        }}
                        onMouseEnter={(e) => {
                          (e.currentTarget as HTMLDivElement).style.background = 'rgba(13,148,136,0.22)';
                          (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(13,148,136,0.55)';
                        }}
                        onMouseLeave={(e) => {
                          (e.currentTarget as HTMLDivElement).style.background = 'rgba(226,232,240,0.55)';
                          (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(15,23,42,0.22)';
                        }}
                      >
                        <div style={{ fontSize: 16, fontWeight: 600, color: '#0f172a', marginBottom: 8 }}>
                          {row.province}
                        </div>
                        <div
                          style={{
                            fontSize: 10,
                            lineHeight: 1.4,
                            color: 'rgba(15,23,42,0.7)',
                            fontFamily: 'monospace',
                            wordBreak: 'break-all',
                          }}
                        >
                          {tryEncodeSs58(row.admin_pubkey)}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </Card>
            )
          ) : (
            <>
          <Card
            title={'公民身份列表'}
            bordered={false}
            style={glassCardStyle}
            headStyle={glassCardHeadStyle}
            extra={
              <Form layout="inline" onFinish={onSearch}>
                <Form.Item name="keyword" style={{ marginBottom: 0 }}>
                  <Input style={{ width: 420 }} placeholder="请输入账户、档案号或SFID号" allowClear />
                </Form.Item>
                <Form.Item style={{ marginBottom: 0 }}>
                  <Button htmlType="submit" type="primary" loading={loading}>
                    查询
                  </Button>
                </Form.Item>
              </Form>
            }
          >
            <Table<CitizenRow>
              rowKey={(r) => `${r.id}`}
              dataSource={rows}
              loading={loading}
              pagination={{ pageSize: 10 }}
              columns={citizenColumns}
            />
          </Card>
            </>
          )}
        </Content>
      )}

      {capabilities.canBusinessWrite && (
        <Modal
          title={<span style={{ fontSize: 20, fontWeight: 600 }}>绑定身份</span>}
          open={bindModalOpen}
          footer={null}
          onCancel={() => {
            setBindModalOpen(false);
            setBindScannerActive(false);
            stopBindScanner();
          }}
          destroyOnClose
          width={520}
        >
          {/* 步骤指示 */}
          <Typography.Paragraph type="secondary" style={{ marginBottom: 16 }}>
            {bindMode === 'bind_archive'
              ? '模式：有账户绑档案（扫描 CPMS 档案二维码 → 签名验证 → 完成绑定）'
              : '模式：有档案绑账户（输入新账户 → 签名验证 → 完成绑定）'}
          </Typography.Paragraph>

          {/* 模式1：有账户绑档案 - 第一步扫 QR4 */}
          {bindMode === 'bind_archive' && bindStep === 'scan_qr4' && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                第一步：扫描 CPMS 档案二维码（QR4）
              </Typography.Text>
              <div
                style={{
                  width: '84%',
                  maxWidth: 320,
                  aspectRatio: '1 / 1',
                  background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                  borderRadius: 16,
                  overflow: 'hidden',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  position: 'relative',
                  margin: '14px auto 12px',
                  border: '2px solid #334155',
                  boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
                }}
              >
                <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!bindScannerReady && (
                  <div
                    style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: bindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                    onClick={() => { if (!bindScannerActive) onToggleBindScanner(); }}
                  >
                    <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                    <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      {bindScannerActive ? '摄像头初始化中...' : '点击扫描档案二维码'}
                    </Typography.Text>
                  </div>
                )}
              </div>
              <div style={{ textAlign: 'center' }}>
                <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
                  {bindScannerActive ? '停止扫码' : '开启扫码'}
                </Button>
              </div>
            </>
          )}

          {/* 模式2：有档案绑账户 - 第一步输入账户 */}
          {bindMode === 'bind_pubkey' && bindStep === 'input_pubkey' && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                第一步：输入新账户
              </Typography.Text>
              <Form layout="vertical">
                <Form.Item label="记录ID">
                  <Input value={bindTargetRecord?.id ?? ''} disabled />
                </Form.Item>
                <Form.Item label="档案号">
                  <Input value={bindTargetRecord?.archive_no ?? ''} disabled />
                </Form.Item>
                <Form.Item label="SFID码">
                  <Input value={bindTargetRecord?.sfid_code ?? ''} disabled />
                </Form.Item>
                <Form.Item label="新账户" required>
                  <Input
                    value={bindNewPubkey}
                    onChange={(e) => setBindNewPubkey(e.target.value)}
                    placeholder="请输入新账户（SS58 地址）"
                  />
                </Form.Item>
                <Button type="primary" onClick={onBindPubkeyNext} loading={bindChallengeLoading}>
                  下一步：生成签名挑战
                </Button>
              </Form>
            </>
          )}

          {/* 第二步：展示签名挑战二维码 */}
          {bindStep === 'sign_challenge' && bindChallenge && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                第二步：用 公民 钱包扫码签名
              </Typography.Text>
              <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
                <QRCode value={bindChallenge.sign_request} size={260} color="#134e4a" />
              </div>
              <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
                有效期至：{new Date(bindChallenge.expire_at * 1000).toLocaleTimeString()}
              </Typography.Paragraph>
              <div style={{ textAlign: 'center' }}>
                <Button
                  type="primary"
                  onClick={() => {
                    setBindStep('scan_signature');
                    setBindScannerActive(true);
                  }}
                >
                  下一步：扫描签名结果
                </Button>
              </div>
            </>
          )}

          {/* 第三步：扫描签名结果 */}
          {bindStep === 'scan_signature' && (
            <>
              <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                第三步：扫描签名结果二维码
              </Typography.Text>
              <div
                style={{
                  width: '84%',
                  maxWidth: 320,
                  aspectRatio: '1 / 1',
                  background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                  borderRadius: 16,
                  overflow: 'hidden',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  position: 'relative',
                  margin: '14px auto 12px',
                  border: '2px solid #334155',
                  boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
                }}
              >
                <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                {!bindScannerReady && (
                  <div
                    style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: bindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                    onClick={() => { if (!bindScannerActive) onToggleBindScanner(); }}
                  >
                    <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                    <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                      {bindScannerActive ? '摄像头初始化中...' : '点击扫描签名二维码'}
                    </Typography.Text>
                  </div>
                )}
              </div>
              <div style={{ textAlign: 'center' }}>
                <Button onClick={onToggleBindScanner} loading={bindQr4ScanLoading}>
                  {bindScannerActive ? '停止扫码' : '开启扫码'}
                </Button>
              </div>
            </>
          )}
        </Modal>
      )}

      {/* 解绑弹窗 */}
      <Modal
        title={<span style={{ fontSize: 20, fontWeight: 600 }}>解绑身份</span>}
        open={unbindModalOpen}
        footer={null}
        onCancel={() => {
          setUnbindModalOpen(false);
          setUnbindScannerActive(false);
          stopUnbindScanner();
        }}
        destroyOnClose
        width={520}
      >
        {unbindTarget && (
          <>
            <div style={{ marginBottom: 16, padding: '12px 16px', background: '#fff7ed', borderRadius: 8, border: '1px solid #fed7aa' }}>
              <div style={{ color: '#9a3412', fontWeight: 500, marginBottom: 4 }}>
                解绑后账户将被清除，档案号和SFID码保留。
              </div>
              <div style={{ color: '#78716c', fontSize: 13 }}>
                账户：{unbindTarget.account_pubkey ? tryEncodeSs58(unbindTarget.account_pubkey) : '-'}
              </div>
            </div>

            {/* 第一步：确认并生成 challenge */}
            {unbindStep === 'confirm' && (
              <div style={{ textAlign: 'center' }}>
                <Button
                  type="primary"
                  danger
                  onClick={onUnbindGenerateChallenge}
                  loading={unbindChallengeLoading}
                >
                  确认解绑 — 生成签名挑战
                </Button>
              </div>
            )}

            {/* 第二步：展示签名二维码 */}
            {unbindStep === 'sign_challenge' && unbindChallenge && (
              <>
                <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                  请用该公钥的 公民 钱包扫码签名
                </Typography.Text>
                <div style={{ display: 'flex', justifyContent: 'center', margin: '12px 0' }}>
                  <QRCode value={unbindChallenge.sign_request} size={260} color="#134e4a" />
                </div>
                <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
                  有效期至：{new Date(unbindChallenge.expire_at * 1000).toLocaleTimeString()}
                </Typography.Paragraph>
                <div style={{ textAlign: 'center' }}>
                  <Button
                    type="primary"
                    onClick={() => {
                      setUnbindStep('scan_signature');
                      setUnbindScannerActive(true);
                    }}
                  >
                    下一步：扫描签名结果
                  </Button>
                </div>
              </>
            )}

            {/* 第三步：扫描签名结果 */}
            {unbindStep === 'scan_signature' && (
              <>
                <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>
                  扫描签名结果二维码
                </Typography.Text>
                <div
                  style={{
                    width: '84%',
                    maxWidth: 320,
                    aspectRatio: '1 / 1',
                    background: 'linear-gradient(145deg, #0f172a, #1e293b)',
                    borderRadius: 16,
                    overflow: 'hidden',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    position: 'relative',
                    margin: '14px auto 12px',
                    border: '2px solid #334155',
                    boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
                  }}
                >
                  <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
                  <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
                  <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
                  <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
                  <video ref={unbindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
                  {!unbindScannerReady && (
                    <div
                      style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, cursor: unbindScannerActive ? 'default' : 'pointer', userSelect: 'none' }}
                      onClick={() => { if (!unbindScannerActive) setUnbindScannerActive(true); }}
                    >
                      <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                      <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                        {unbindScannerActive ? '摄像头初始化中...' : '点击扫描签名二维码'}
                      </Typography.Text>
                    </div>
                  )}
                </div>
                <div style={{ textAlign: 'center' }}>
                  <Button
                    onClick={() => setUnbindScannerActive((v) => !v)}
                    loading={unbindSubmitting}
                  >
                    {unbindScannerActive ? '停止扫码' : '开启扫码'}
                  </Button>
                </div>
              </>
            )}
          </>
        )}
      </Modal>

      <Modal
        title={opScanType === 'register' ? '新增机构（扫码）' : '状态变更扫码'}
        open={opScanOpen}
        footer={null}
        onCancel={() => {
          setOpScanOpen(false);
          stopOpScanner();
        }}
        destroyOnClose
      >
        <Typography.Paragraph type="secondary">
          请使用本机摄像头扫描二维码。
        </Typography.Paragraph>
        <div
          style={{
            width: '100%',
            aspectRatio: '1 / 1',
            background: 'linear-gradient(145deg, #0f172a, #1e293b)',
            borderRadius: 16,
            overflow: 'hidden',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            position: 'relative',
            border: '2px solid #334155',
            boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)'
          }}
        >
          <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
          <video ref={opVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
          {!opScannerReady && (
            <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
              <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
              <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>摄像头初始化中...</Typography.Text>
            </div>
          )}
        </div>
      </Modal>

      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增公权机构</div>}
        open={institutionSfidOpen}
        onCancel={() => setInstitutionSfidOpen(false)}
        footer={[
          <Button
            key="download"
            icon={<DownloadOutlined />}
            disabled={!institutionSfidResult}
            onClick={onDownloadInstitutionSfid}
          >
            下载
          </Button>,
          <Button
            key="primary"
            type="primary"
            loading={institutionSfidLoading}
            onClick={() => {
              if (institutionSfidResult) {
                onFinishInstitutionSfid();
                return;
              }
              institutionSfidForm.submit();
            }}
          >
            {institutionSfidResult ? '完成' : '生成'}
          </Button>
        ]}
        destroyOnClose
      >
        <Form form={institutionSfidForm} layout="vertical" onFinish={onGenerateInstitutionSfid}>
          <Form.Item label="A3 主体属性">
            <Input value="公法人 (GFR)" disabled />
          </Form.Item>
          <Form.Item label="P1 盈利属性">
            <Input value="非盈利 (0)" disabled />
          </Form.Item>
          <Form.Item label="省" name="province" rules={[{ required: true, message: '请选择省' }]}>
            <Select
              options={(sfidMeta?.provinces || []).map((p) => ({ label: `${p.name} (${p.code})`, value: p.name }))}
              placeholder="请选择省"
              disabled={Boolean(auth?.admin_province)}
              onChange={(provinceName) => {
                institutionSfidForm.setFieldsValue({ city: '' });
                void loadSfidCities(provinceName);
              }}
            />
          </Form.Item>
          <Form.Item label="市" name="city" rules={[{ required: true, message: '请选择市' }]}>
            <Select
              loading={sfidCitiesLoading}
              options={sfidCities.filter((c) => c.code !== '000').map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
              placeholder="请选择该省下的市"
            />
          </Form.Item>
          <Form.Item label="机构" name="institution" rules={[{ required: true, message: '请选择机构类型' }]}>
            <Select
              options={(sfidMeta?.institution_options || [])
                .filter((o) => allowedInstitutionByA3('GFR').includes(o.value))
                .map((o) => ({ label: `${o.label} (${o.value})`, value: o.value }))}
              placeholder="请选择机构类型"
            />
          </Form.Item>
          <Form.Item
            label="机构名称"
            name="institution_name"
            rules={[
              { required: true, message: '请输入机构名称' },
              { max: 30, message: '机构名称最多30个字' }
            ]}
          >
            <Input placeholder="请输入机构名称（最多30个字）" maxLength={30} />
          </Form.Item>
        </Form>
        {institutionSfidResult && (
          <Space direction="vertical" size={8} style={{ width: '100%' }}>
            <Typography.Text strong>身份识别码：{institutionSfidResult.site_sfid}</Typography.Text>
            <div style={{ display: 'flex', justifyContent: 'center' }}>
              <div ref={institutionQrRef} style={{ background: '#fff', padding: 16 }}>
                <QRCode value={institutionSfidResult.qr1_payload} size={260} />
              </div>
            </div>
          </Space>
        )}
      </Modal>

      <Modal
        title="身份识别码二维码"
        open={Boolean(institutionQrPreview)}
        onCancel={() => setInstitutionQrPreview(null)}
        footer={[
          <Button key="download-preview" icon={<DownloadOutlined />} onClick={onDownloadInstitutionPreview}>
            下载
          </Button>,
          <Button key="close-preview" type="primary" onClick={() => setInstitutionQrPreview(null)}>
            关闭
          </Button>
        ]}
        destroyOnClose
      >
        {institutionQrPreview && (
          <Space direction="vertical" size={8} style={{ width: '100%' }}>
            <Typography.Text strong>身份识别码：{institutionQrPreview.site_sfid}</Typography.Text>
            <div style={{ display: 'flex', justifyContent: 'center' }}>
              <div ref={institutionQrPreviewRef} style={{ background: '#fff', padding: 16 }}>
                <QRCode value={institutionQrPreview.qr1_payload} size={260} />
              </div>
            </div>
          </Space>
        )}
      </Modal>

      {/* ── 多签机构 SFID 生成弹窗 ── */}
      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>生成机构SFID</div>}
        open={multisigModalOpen}
        onCancel={() => setMultisigModalOpen(false)}
        footer={[
          <Button key="cancel" onClick={() => setMultisigModalOpen(false)}>取消</Button>,
          <Button
            key="submit"
            type="primary"
            loading={multisigGenerating}
            onClick={() => multisigForm.submit()}
          >
            {multisigGenerating ? '上链中，请等待...' : '生成并上链注册'}
          </Button>
        ]}
        destroyOnClose
      >
        {multisigGenerating && (
          <Typography.Text type="warning" style={{ display: 'block', marginBottom: 12 }}>
            正在提交到区块链，最长等待约 2 分钟，请勿关闭...
          </Typography.Text>
        )}
        <Form form={multisigForm} layout="vertical" onFinish={onGenerateMultisigSfid}>
          <Form.Item label="A3 主体属性" name="a3" rules={[{ required: true, message: '请选择 A3 类型' }]}>
            <Select
              options={(sfidMeta?.a3_options || []).filter((o) => ['GFR', 'SFR', 'FFR'].includes(o.value)).map((o) => ({ label: `${o.label} (${o.value})`, value: o.value }))}
              placeholder="请选择 A3 主体属性"
              onChange={(a3Value: string) => {
                setMultisigA3(a3Value);
                multisigForm.setFieldsValue({
                  institution: defaultInstitutionByA3(a3Value),
                  p1: p1LockedByA3(a3Value) ? (a3Value === 'GFR' ? '0' : '1') : undefined,
                });
              }}
            />
          </Form.Item>
          {!p1LockedByA3(multisigA3) ? (
            <Form.Item label="P1 盈利属性" name="p1" rules={[{ required: true, message: '请选择盈利属性' }]}>
              <Select
                options={[
                  { label: '非盈利 (0)', value: '0' },
                  { label: '盈利 (1)', value: '1' },
                ]}
                placeholder="请选择盈利属性"
              />
            </Form.Item>
          ) : (
            <Form.Item label="P1 盈利属性">
              <Input value={multisigA3 === 'GFR' ? '非盈利 (0)' : '盈利 (1)'} disabled />
            </Form.Item>
          )}
          <Form.Item label="省" name="province" rules={[{ required: true, message: '请选择省' }]}>
            <Select
              options={(sfidMeta?.provinces || []).map((p) => ({ label: `${p.name} (${p.code})`, value: p.name }))}
              placeholder="请选择省"
              disabled={Boolean(auth?.admin_province)}
              onChange={(provinceName: string) => {
                multisigForm.setFieldsValue({ city: '' });
                void loadSfidCities(provinceName);
              }}
            />
          </Form.Item>
          <Form.Item label="市" name="city" rules={[{ required: true, message: '请选择市' }]}>
            <Select
              loading={sfidCitiesLoading}
              options={sfidCities.filter((c) => c.code !== '000').map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
              placeholder="请选择该省下的市"
              disabled={auth?.role === 'SYSTEM_ADMIN'}
            />
          </Form.Item>
          <Form.Item label="机构类型" name="institution" rules={[{ required: true, message: '请选择机构类型' }]}>
            <Select
              options={(sfidMeta?.institution_options || [])
                .filter((o) => allowedInstitutionByA3(multisigA3).includes(o.value))
                .map((o) => ({ label: `${o.label} (${o.value})`, value: o.value }))}
              placeholder="请选择机构类型"
            />
          </Form.Item>
          <Form.Item
            label="机构名称"
            name="institution_name"
            rules={[
              { required: true, message: '请输入机构名称' },
              { max: 30, message: '机构名称最多30个字' },
            ]}
          >
            <Input placeholder="请输入机构名称（最多30个字）" maxLength={30} />
          </Form.Item>
        </Form>
      </Modal>

      {/* ── 新增系统管理员 Modal（在机构详情页触发） ── */}
      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增系统管理员</div>}
        open={addOperatorOpen}
        onCancel={() => {
          addOperatorForm.resetFields();
          setAddOperatorOpen(false);
        }}
        footer={[
          <Button
            key="cancel"
            onClick={() => {
              addOperatorForm.resetFields();
              setAddOperatorOpen(false);
            }}
          >
            取消新增
          </Button>,
          <Button
            key="submit"
            type="primary"
            loading={addOperatorLoading}
            onClick={() => addOperatorForm.submit()}
          >
            确认新增
          </Button>,
        ]}
        destroyOnClose
      >
        <Form
          form={addOperatorForm}
          layout="vertical"
          onFinish={(values: { operator_name: string; operator_pubkey: string; operator_city: string }) =>
            onCreateOperator({
              operator_name: values.operator_name,
              operator_pubkey: values.operator_pubkey,
              city: values.operator_city,
              created_by: selectedSuperAdmin?.admin_pubkey,
            })
          }
        >
          <Form.Item
            label="姓名"
            name="operator_name"
            rules={[{ required: true, message: '请输入系统管理员姓名' }]}
          >
            <Input placeholder="请输入系统管理员姓名" />
          </Form.Item>
          <Form.Item
            label="市"
            name="operator_city"
            rules={[{ required: true, message: '请选择市' }]}
          >
            <Select
              placeholder="请选择市"
              loading={operatorCitiesLoading}
              options={operatorCities
                .filter((c) => c.code !== '000')
                .map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
            />
          </Form.Item>
          <Form.Item
            label="账户"
            name="operator_pubkey"
            rules={[
              { required: true, message: '请输入系统管理员账户' },
              {
                validator: async (_rule, value) => {
                  if (!value) return;
                  try {
                    decodeSs58(String(value));
                  } catch (err) {
                    throw new Error(err instanceof Error ? err.message : '账户格式无效');
                  }
                },
              },
            ]}
          >
            <Input
              placeholder="请输入系统管理员账户（SS58）"
              suffix={
                <span
                  title="扫码识别用户码"
                  style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                  onClick={() => setAccountScanTarget('operator')}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>
                    <rect x="7" y="7" width="10" height="10" rx="1"/>
                  </svg>
                </span>
              }
            />
          </Form.Item>
        </Form>
      </Modal>

      {/* ── 密钥管理：扫码识别"新备用账户"弹窗 ── */}
      <ScanAccountModal
        open={keyringScanAccountOpen}
        onClose={() => setKeyringScanAccountOpen(false)}
        onResolved={(addr) => {
          keyringForm.setFieldsValue({ new_backup_pubkey: addr });
          setKeyringScanAccountOpen(false);
        }}
      />

      {/* ── 通用扫码识别账户弹窗（新增系统管理员 / 更换机构管理员等） ── */}
      <ScanAccountModal
        open={accountScanTarget !== null}
        onClose={() => setAccountScanTarget(null)}
        onResolved={(addr) => {
          if (accountScanTarget === 'operator') {
            addOperatorForm.setFieldsValue({ operator_pubkey: addr });
          } else if (accountScanTarget === 'super-admin') {
            replaceSuperForm.setFieldsValue({ admin_pubkey: addr });
          }
          setAccountScanTarget(null);
        }}
      />

    </Layout>
  );
}
