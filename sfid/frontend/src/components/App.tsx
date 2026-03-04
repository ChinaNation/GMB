import { useEffect, useRef, useState } from 'react';
import { DownloadOutlined, ExclamationCircleFilled, QrcodeOutlined } from '@ant-design/icons';
import { Button, Card, Divider, Form, Input, Layout, Modal, QRCode, Select, Space, Table, Typography, message } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import type {
  AdminAuth,
  AdminQrChallengeResult,
  CitizenRow,
  CpmsSiteRow,
  GenerateCpmsInstitutionSfidResult,
  KeyringRotateChallengeResult,
  KeyringStateResult,
  OperatorRow,
  SfidCityItem,
  SfidMetaResult,
  SuperAdminRow
} from '../api/client';
import {
  checkAdminAuth,
  completeAdminQrLogin,
  confirmBind,
  createKeyringRotateChallenge,
  createOperator,
  createAdminQrChallenge,
  deleteCpmsKeys,
  deleteOperator,
  disableCpmsKeys,
  generateCpmsInstitutionSfid,
  generateSfid,
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
  registerCpmsKeysScan,
  scanBindQr,
  scanCpmsStatusQr,
  unbind,
  updateCpmsKeys,
  updateOperator,
  updateOperatorStatus
} from '../api/client';
const loginBg = '/assets/login-bg.png';

const { Header, Content } = Layout;
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

function resolveAdminName(auth: AdminAuth | null): string {
  if (!auth) return '';
  if (typeof auth.admin_name === 'string' && auth.admin_name.trim()) {
    return auth.admin_name.trim();
  }
  if (auth.role === 'KEY_ADMIN') return '密钥管理员';
  if (auth.role === 'SUPER_ADMIN') return '超级管理员';
  if (auth.role === 'OPERATOR_ADMIN') return '操作管理员';
  return '查询管理员';
}

function resolveHeaderAdminName(auth: AdminAuth | null): string {
  if (!auth) return '';
  if (auth.role === 'OPERATOR_ADMIN') {
    if (typeof auth.admin_name === 'string' && auth.admin_name.trim()) {
      return `操作管理员：${auth.admin_name.trim()}`;
    }
    return '操作管理员';
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

type BarcodeDetectorLike = {
  detect: (source: ImageBitmapSource) => Promise<Array<{ rawValue?: string }>>;
};

type BarcodeDetectorCtor = new (opts: { formats: string[] }) => BarcodeDetectorLike;

function parseSignedLoginPayload(raw: string, fallbackChallengeId: string): SignedLoginPayload {
  const payload = JSON.parse(raw) as Record<string, unknown>;
  const challenge_id =
    (typeof payload.request_id === 'string' && payload.request_id.trim()) ||
    (typeof payload.challenge_id === 'string' && payload.challenge_id.trim()) ||
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

type RoleCapabilities = {
  canViewAdminNav: boolean;
  canManageOperators: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canManageKeyring: boolean;
  canReplaceSuperAdmins: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  isQueryOnly: boolean;
};

function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isKeyAdmin = role === 'KEY_ADMIN';
  const isSuperAdmin = role === 'SUPER_ADMIN';
  const isOperatorAdmin = role === 'OPERATOR_ADMIN';
  const isQueryOnly = role === 'QUERY_ONLY';
  return {
    canViewAdminNav: isKeyAdmin || isSuperAdmin,
    canManageOperators: isKeyAdmin || isSuperAdmin,
    canManageInstitutions: isSuperAdmin,
    canRegisterInstitutions: isSuperAdmin,
    canManageKeyring: isKeyAdmin,
    canReplaceSuperAdmins: isKeyAdmin,
    canStatusScan: isKeyAdmin || isSuperAdmin || isOperatorAdmin,
    canBusinessWrite: Boolean(role) && !isQueryOnly,
    isQueryOnly
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
  const [bindScanLoading, setBindScanLoading] = useState(false);
  const [bindScanResult, setBindScanResult] = useState<{
    archive_no: string;
    site_sfid: string;
    qr_id: string;
  } | null>(null);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const [scannerActive, setScannerActive] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [activeView, setActiveView] = useState<'citizens' | 'operators' | 'institutions' | 'keyring'>('citizens');
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
  const [institutionSfidDrafts, setInstitutionSfidDrafts] = useState<GenerateCpmsInstitutionSfidResult[]>([]);
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
  const [sfidToolOpen, setSfidToolOpen] = useState(false);
  const [sfidToolTargetPubkey, setSfidToolTargetPubkey] = useState('');
  const [sfidToolLoading, setSfidToolLoading] = useState(false);
  const [sfidMeta, setSfidMeta] = useState<SfidMetaResult | null>(null);
  const [sfidCities, setSfidCities] = useState<SfidCityItem[]>([]);
  const [sfidCitiesLoading, setSfidCitiesLoading] = useState(false);
  const [addOperatorForm] = Form.useForm<{ operator_pubkey: string; operator_name: string }>();
  const [institutionSfidForm] = Form.useForm<{
    province: string;
    city: string;
    institution: string;
  }>();
  const [replaceSuperForm] = Form.useForm<{ province: string; admin_pubkey: string }>();
  const [keyringForm] = Form.useForm<{ initiator_pubkey: string }>();
  const [keyringCommitForm] = Form.useForm<{ new_backup_pubkey: string }>();
  const [sfidToolForm] = Form.useForm<{
    a3: string;
    p1: string;
    province: string;
    city: string;
    institution: string;
  }>();
  const sfidToolA3 = Form.useWatch('a3', sfidToolForm);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanStreamRef = useRef<MediaStream | null>(null);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanStreamRef = useRef<MediaStream | null>(null);
  const opVideoRef = useRef<HTMLVideoElement | null>(null);
  const opScanStreamRef = useRef<MediaStream | null>(null);
  const keyringVideoRef = useRef<HTMLVideoElement | null>(null);
  const keyringScanStreamRef = useRef<MediaStream | null>(null);
  const institutionQrRef = useRef<HTMLDivElement | null>(null);
  const institutionQrPreviewRef = useRef<HTMLDivElement | null>(null);

  const capabilities = resolveRoleCapabilities(auth);

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
          admin_province: checked.admin_province ?? null
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
    if (scanStreamRef.current) {
      scanStreamRef.current.getTracks().forEach((t) => t.stop());
      scanStreamRef.current = null;
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
          admin_province: status.admin.admin_province ?? null
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
    if (!scannerActive || !pendingQrLogin) {
      stopScanner();
      return;
    }
    let cancelled = false;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      message.warning('当前浏览器不支持摄像头扫码');
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: 'environment' },
          audio: false
        });
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        scanStreamRef.current = stream;
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          await videoRef.current.play();
          setScannerReady(true);
        }
        const timer = window.setInterval(async () => {
          if (!videoRef.current || scanSubmitting) return;
          try {
            const codes = await detector.detect(videoRef.current);
            const raw = codes[0]?.rawValue?.trim();
            if (raw) {
              window.clearInterval(timer);
              await onCompleteSignedLogin(raw);
            }
          } catch {
            // ignore frame failures
          }
        }, 700);
        return () => window.clearInterval(timer);
      } catch {
        message.error('无法打开摄像头，请检查权限或改用粘贴方式');
      }
    };
    let clear: (() => void) | undefined;
    start().then((fn) => {
      clear = fn;
    });
    return () => {
      cancelled = true;
      if (clear) clear();
      stopScanner();
    };
  }, [scannerActive, scanSubmitting, pendingQrLogin]);

  useEffect(() => {
    if (!opScanOpen) {
      stopOpScanner();
      return;
    }
    let cancelled = false;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      message.warning('当前浏览器不支持摄像头扫码');
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: 'environment' },
          audio: false
        });
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        opScanStreamRef.current = stream;
        if (opVideoRef.current) {
          opVideoRef.current.srcObject = stream;
          await opVideoRef.current.play();
          setOpScannerReady(true);
        }
        const timer = window.setInterval(async () => {
          if (!opVideoRef.current || opScanSubmitting) return;
          try {
            const codes = await detector.detect(opVideoRef.current);
            const raw = codes[0]?.rawValue?.trim();
            if (raw) {
              window.clearInterval(timer);
              await onHandleOperationQr(raw);
            }
          } catch {
            // ignore frame failures
          }
        }, 700);
        return () => window.clearInterval(timer);
      } catch {
        message.error('无法打开摄像头，请检查权限');
      }
    };
    let clear: (() => void) | undefined;
    start().then((fn) => {
      clear = fn;
    });
    return () => {
      cancelled = true;
      if (clear) clear();
      stopOpScanner();
    };
  }, [opScanOpen, opScanSubmitting, opScanType, auth]);

  useEffect(() => {
    if (!keyringScannerActive || !keyringChallenge) {
      stopKeyringScanner();
      return;
    }
    let cancelled = false;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      message.warning('当前浏览器不支持摄像头扫码');
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: 'environment' },
          audio: false
        });
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        keyringScanStreamRef.current = stream;
        if (keyringVideoRef.current) {
          keyringVideoRef.current.srcObject = stream;
          await keyringVideoRef.current.play();
          setKeyringScannerReady(true);
        }
        const timer = window.setInterval(async () => {
          if (!keyringVideoRef.current || keyringScanSubmitting) return;
          try {
            const codes = await detector.detect(keyringVideoRef.current);
            const raw = codes[0]?.rawValue?.trim();
            if (raw) {
              window.clearInterval(timer);
              await onCompleteKeyringRotate(raw);
            }
          } catch {
            // ignore frame failures
          }
        }, 700);
        return () => window.clearInterval(timer);
      } catch {
        message.error('无法打开摄像头，请检查权限');
      }
    };
    let clear: (() => void) | undefined;
    start().then((fn) => {
      clear = fn;
    });
    return () => {
      cancelled = true;
      if (clear) clear();
      stopKeyringScanner();
    };
  }, [keyringScannerActive, keyringChallenge, keyringScanSubmitting]);

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
            admin_province: status.admin.admin_province ?? null
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
    setKeyringState(null);
    setKeyringChallenge(null);
    setKeyringSignedPayload(null);
    setKeyringScannerActive(false);
    stopKeyringScanner();
    keyringForm.resetFields();
    keyringCommitForm.resetFields();
    message.success('已退出登录');
  };

  const refreshList = async (currentAuth: AdminAuth, keyword?: string, silent?: boolean) => {
    setLoading(true);
    try {
      const list = await listCitizens(currentAuth, keyword);
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
    await refreshList(auth, values.keyword?.trim());
  };

  const refreshOperators = async (currentAuth: AdminAuth) => {
    setOperatorsLoading(true);
    try {
      const rows = await listOperators(currentAuth);
      setOperators(rows);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载操作管理员失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  const refreshSuperAdmins = async (currentAuth: AdminAuth) => {
    setSuperAdminsLoading(true);
    try {
      const rows = await listSuperAdmins(currentAuth);
      setSuperAdmins(rows);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载超级管理员失败';
      message.error(msg);
    } finally {
      setSuperAdminsLoading(false);
    }
  };

  const refreshCpmsSites = async (currentAuth: AdminAuth) => {
    setCpmsSitesLoading(true);
    try {
      const rows = await listCpmsSites(currentAuth);
      setCpmsSites(rows);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载机构列表失败';
      message.error(msg);
    } finally {
      setCpmsSitesLoading(false);
    }
  };

  const refreshKeyringState = async (currentAuth: AdminAuth) => {
    setKeyringLoading(true);
    try {
      const state = await getAttestorKeyring(currentAuth);
      setKeyringState(state);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载密钥状态失败';
      message.error(msg);
    } finally {
      setKeyringLoading(false);
    }
  };

  const stopKeyringScanner = () => {
    if (keyringScanStreamRef.current) {
      keyringScanStreamRef.current.getTracks().forEach((t) => t.stop());
      keyringScanStreamRef.current = null;
    }
    setKeyringScannerReady(false);
  };

  const onCreateKeyringRotateChallenge = async (values: { initiator_pubkey: string }) => {
    if (!auth) return;
    setKeyringActionLoading(true);
    try {
      const challenge = await createKeyringRotateChallenge(auth, {
        initiator_pubkey: values.initiator_pubkey.trim()
      });
      setKeyringChallenge(challenge);
      setKeyringSignedPayload(null);
      keyringCommitForm.resetFields();
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
      message.success('签名校验通过，请输入新备用公钥确认轮换');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '提交轮换签名失败';
      message.error(msg);
    } finally {
      setKeyringScanSubmitting(false);
    }
  };

  const onCommitKeyringRotate = async (values: { new_backup_pubkey: string }) => {
    if (!auth || !keyringSignedPayload || !keyringChallenge) {
      message.error('请先完成签名校验');
      return;
    }
    setKeyringCommitLoading(true);
    try {
      const result = await commitKeyringRotate(auth, {
        challenge_id: keyringSignedPayload.challenge_id,
        signature: keyringSignedPayload.signature,
        new_backup_pubkey: values.new_backup_pubkey.trim()
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
      setKeyringScannerActive(false);
      stopKeyringScanner();
      keyringForm.resetFields(['initiator_pubkey']);
      keyringCommitForm.resetFields();
      await refreshKeyringState(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '提交轮换失败';
      message.error(msg);
    } finally {
      setKeyringCommitLoading(false);
    }
  };

  const onToggleKeyringScanner = () => {
    if (!keyringChallenge) {
      message.warning('请先生成轮换二维码');
      return;
    }
    setKeyringScannerActive((v) => !v);
  };

  const onCreateOperator = async (values: { operator_pubkey: string; operator_name: string }) => {
    if (!auth) return;
    const admin_pubkey = values.operator_pubkey?.trim();
    const admin_name = values.operator_name?.trim();
    if (!admin_pubkey) {
      message.error('请输入管理员公钥');
      return;
    }
    if (!admin_name) {
      message.error('请输入管理员姓名');
      return;
    }
    setAddOperatorLoading(true);
    try {
      const created = await createOperator(auth, { admin_pubkey, admin_name });
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
    if (opScanStreamRef.current) {
      opScanStreamRef.current.getTracks().forEach((t) => t.stop());
      opScanStreamRef.current = null;
    }
    setOpScannerReady(false);
  };

  const onHandleOperationQr = async (raw: string) => {
    if (!auth) return;
    setOpScanSubmitting(true);
    try {
      if (opScanType === 'register') {
        const result = await registerCpmsKeysScan(auth, { qr_payload: raw });
        message.success(`机构 ${result.site_sfid} 公钥登记成功`);
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

  const onUpdateCpmsSiteKeys = (row: CpmsSiteRow) => {
    if (!auth) return;
    let pubkey1 = row.pubkey_1;
    let pubkey2 = row.pubkey_2;
    let pubkey3 = row.pubkey_3;
    Modal.confirm({
      title: `更新机构公钥 (${row.site_sfid})`,
      width: 720,
      content: (
        <Space direction="vertical" style={{ width: '100%' }}>
          <Input
            defaultValue={row.pubkey_1}
            placeholder="公钥1"
            onChange={(event) => {
              pubkey1 = event.target.value;
            }}
          />
          <Input
            defaultValue={row.pubkey_2}
            placeholder="公钥2"
            onChange={(event) => {
              pubkey2 = event.target.value;
            }}
          />
          <Input
            defaultValue={row.pubkey_3}
            placeholder="公钥3"
            onChange={(event) => {
              pubkey3 = event.target.value;
            }}
          />
        </Space>
      ),
      okText: '确认更新',
      cancelText: '取消',
      onOk: async () => {
        const payload = {
          pubkey_1: pubkey1.trim(),
          pubkey_2: pubkey2.trim(),
          pubkey_3: pubkey3.trim()
        };
        if (!payload.pubkey_1 || !payload.pubkey_2 || !payload.pubkey_3) {
          message.error('三个公钥都必须填写');
          throw new Error('cpms pubkeys required');
        }
        setCpmsSitesLoading(true);
        try {
          await updateCpmsKeys(auth, row.site_sfid, payload);
          message.success(`机构 ${row.site_sfid} 公钥已更新`);
          await refreshCpmsSites(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '更新机构公钥失败';
          message.error(msg);
          throw err;
        } finally {
          setCpmsSitesLoading(false);
        }
      }
    });
  };

  const onUpdateCpmsSiteKey = (row: CpmsSiteRow, keySlot: 1 | 2 | 3) => {
    if (!auth) return;
    let nextValue =
      keySlot === 1 ? row.pubkey_1 : keySlot === 2 ? row.pubkey_2 : row.pubkey_3;
    Modal.confirm({
      title: `更新公钥${keySlot} (${row.site_sfid})`,
      content: (
        <Input
          defaultValue={nextValue}
          placeholder={`请输入新的公钥${keySlot}`}
          onChange={(event) => {
            nextValue = event.target.value;
          }}
        />
      ),
      okText: '确认更新',
      cancelText: '取消',
      onOk: async () => {
        const value = nextValue.trim();
        if (!value) {
          message.error(`公钥${keySlot}不能为空`);
          throw new Error('cpms pubkey required');
        }
        setCpmsSitesLoading(true);
        try {
          await updateCpmsKeys(auth, row.site_sfid, {
            pubkey_1: keySlot === 1 ? value : row.pubkey_1,
            pubkey_2: keySlot === 2 ? value : row.pubkey_2,
            pubkey_3: keySlot === 3 ? value : row.pubkey_3
          });
          message.success(`机构 ${row.site_sfid} 公钥${keySlot}已更新`);
          await refreshCpmsSites(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '更新机构公钥失败';
          message.error(msg);
          throw err;
        } finally {
          setCpmsSitesLoading(false);
        }
      }
    });
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
      message.success(target === 'ACTIVE' ? '已启用操作管理员' : '已停用操作管理员');
      await refreshOperators(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更新操作管理员状态失败';
      message.error(msg);
    } finally {
      setOperatorsLoading(false);
    }
  };

  const onUpdateOperator = (row: OperatorRow) => {
    if (!auth) return;
    let nextName = row.admin_name;
    let nextPubkey = row.admin_pubkey;
    Modal.confirm({
      title: '修改操作管理员',
      content: (
        <Space direction="vertical" style={{ width: '100%' }}>
          <Input
            defaultValue={row.admin_name}
            placeholder="请输入管理员姓名"
            onChange={(event) => {
              nextName = event.target.value;
            }}
          />
          <Input
            defaultValue={row.admin_pubkey}
            placeholder="请输入新的管理员公钥"
            onChange={(event) => {
              nextPubkey = event.target.value;
            }}
          />
        </Space>
      ),
      okText: '确认修改',
      cancelText: '取消',
      onOk: async () => {
        const admin_name = nextName.trim();
        const admin_pubkey = nextPubkey.trim();
        if (!admin_name) {
          message.error('请输入管理员姓名');
          throw new Error('admin_name is required');
        }
        if (!admin_pubkey) {
          message.error('请输入新的管理员公钥');
          throw new Error('admin_pubkey is required');
        }
        setOperatorsLoading(true);
        try {
          await updateOperator(auth, row.id, { admin_name, admin_pubkey });
          message.success('操作管理员信息已更新');
          await refreshOperators(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '更新操作管理员信息失败';
          message.error(msg);
          throw err;
        } finally {
          setOperatorsLoading(false);
        }
      }
    });
  };

  const loadSfidCities = async (province: string) => {
    if (!auth || !province.trim()) return;
    setSfidCitiesLoading(true);
    try {
      const rows = await listSfidCities(auth, province);
      setSfidCities(rows);
    } catch (err) {
      setSfidCities([]);
      const msg = err instanceof Error ? err.message : '加载城市列表失败';
      message.error(msg);
    } finally {
      setSfidCitiesLoading(false);
    }
  };

  const openSfidTool = async (accountPubkey: string) => {
    setSfidToolTargetPubkey(accountPubkey);
    if (auth) {
      try {
        const meta = await getSfidMeta(auth);
        setSfidMeta(meta);
        const provinceDefault = meta.scoped_province || meta.provinces[0]?.name || '';
        sfidToolForm.setFieldsValue({
          a3: meta.a3_options[0]?.value || 'GFR',
          p1: defaultP1ByA3(meta.a3_options[0]?.value || 'GFR'),
          province: provinceDefault,
          city: '',
          institution: defaultInstitutionByA3(meta.a3_options[0]?.value || 'GFR')
        });
        if (provinceDefault) {
          await loadSfidCities(provinceDefault);
        } else {
          setSfidCities([]);
        }
      } catch (err) {
        const msg = err instanceof Error ? err.message : '加载SFID工具配置失败';
        message.error(msg);
        return;
      }
    }
    setSfidToolOpen(true);
  };

  const onGenerateSfidCode = (values: {
    a3: string;
    p1: string;
    province: string;
    city: string;
    institution: string;
  }) => {
    if (!sfidToolTargetPubkey) return;
    if (!auth) return;
    setSfidToolLoading(true);
    generateSfid(auth, {
      account_pubkey: sfidToolTargetPubkey,
      a3: values.a3,
      p1: values.p1,
      province: values.province,
      city: values.city,
      institution: values.institution
    })
      .then(async (res) => {
        message.success(`SFID码已生成：${res.sfid_code}`);
        setSfidToolOpen(false);
        await refreshList(auth, undefined, true);
      })
      .catch((err) => {
        const msg = err instanceof Error ? err.message : 'SFID码生成失败';
        message.error(msg);
      })
      .finally(() => setSfidToolLoading(false));
  };

  const sfidInstitutionOptions = (sfidMeta?.institution_options || [])
    .filter((o) => allowedInstitutionByA3(sfidToolA3 || '').includes(o.value))
    .map((o) => ({
      value: o.value,
      label: `${o.label} (${o.value})`
    }));
  const institutionLocked = sfidInstitutionOptions.length <= 1;

  const onDeleteOperator = (row: OperatorRow) => {
    if (!auth) return;
    Modal.confirm({
      title: '删除操作管理员',
      content: `确认删除该操作管理员？\n${row.admin_pubkey}`,
      okText: '确认删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        setOperatorsLoading(true);
        try {
          await deleteOperator(auth, row.id);
          message.success('操作管理员已删除');
          await refreshOperators(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '删除操作管理员失败';
          message.error(msg);
        } finally {
          setOperatorsLoading(false);
        }
      }
    });
  };

  const onReplaceSuperAdmin = async (values: { province: string; admin_pubkey: string }) => {
    if (!auth) return;
    setReplaceSuperLoading(true);
    try {
      await replaceSuperAdmin(auth, values.province.trim(), values.admin_pubkey.trim());
      message.success(`已更新 ${values.province} 超级管理员`);
      replaceSuperForm.resetFields();
      await refreshSuperAdmins(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '更换超级管理员失败';
      message.error(msg);
    } finally {
      setReplaceSuperLoading(false);
    }
  };

  const openBindModal = (pubkey: string) => {
    setBindTargetPubkey(pubkey);
    setBindScanResult(null);
    setBindScannerActive(false);
    stopBindScanner();
    setBindModalOpen(true);
  };

  const openRegisterScanner = () => {
    if (!capabilities.canRegisterInstitutions) {
      message.error('仅超级管理员可录入机构');
      return;
    }
    setOpScanType('register');
    setOpScanOpen(true);
  };

  const openInstitutionSfidModal = async () => {
    if (!capabilities.canRegisterInstitutions) {
      message.error('仅超级管理员可生成身份识别码');
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

  const onGenerateInstitutionSfid = async (values: { province: string; city: string; institution: string }) => {
    if (!auth) return;
    setInstitutionSfidLoading(true);
    try {
      const result = await generateCpmsInstitutionSfid(auth, {
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim()
      });
      setInstitutionSfidResult(result);
      message.success(`身份识别码已生成：${result.site_sfid}`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成身份识别码失败';
      message.error(msg);
    } finally {
      setInstitutionSfidLoading(false);
    }
  };

  const onFinishInstitutionSfid = () => {
    if (!institutionSfidResult) return;
    setInstitutionSfidDrafts((prev) => {
      if (prev.some((item) => item.site_sfid === institutionSfidResult.site_sfid)) return prev;
      return [institutionSfidResult, ...prev];
    });
    if (auth) {
      void refreshCpmsSites(auth);
    }
    setInstitutionSfidOpen(false);
  };

  const downloadQrFromRef = (container: HTMLDivElement | null, fileBase: string) => {
    if (!container) {
      message.error('二维码未就绪，无法下载');
      return;
    }
    const safeName = fileBase.replace(/[^\w.-]+/g, '_');
    const canvas = container.querySelector('canvas');
    if (canvas) {
      const link = document.createElement('a');
      link.href = canvas.toDataURL('image/png');
      link.download = `${safeName}.png`;
      link.click();
      return;
    }
    const svg = container.querySelector('svg');
    if (svg) {
      const svgText = new XMLSerializer().serializeToString(svg);
      const blob = new Blob([svgText], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${safeName}.svg`;
      link.click();
      URL.revokeObjectURL(url);
      return;
    }
    message.error('二维码未就绪，无法下载');
  };

  const onDownloadInstitutionSfid = () => {
    if (!institutionSfidResult) {
      message.warning('请先生成身份识别码');
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

  const onScanBindQrRaw = async (qrPayload: string) => {
    if (!auth) return;
    if (!qrPayload.trim()) {
      message.error('二维码识别失败');
      return;
    }
    setBindScanLoading(true);
    try {
      const result = await scanBindQr(auth, { qr_payload: qrPayload });
      setBindScanResult({
        archive_no: result.archive_no,
        site_sfid: result.site_sfid,
        qr_id: result.qr_id
      });
      message.success(`验签通过，档案号：${result.archive_no}，状态：${result.status}`);
      setBindScannerActive(false);
      stopBindScanner();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '二维码验签失败';
      message.error(msg);
    } finally {
      setBindScanLoading(false);
    }
  };

  const stopBindScanner = () => {
    if (bindScanStreamRef.current) {
      bindScanStreamRef.current.getTracks().forEach((t) => t.stop());
      bindScanStreamRef.current = null;
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
    if (!bindModalOpen || !bindScannerActive) {
      stopBindScanner();
      return;
    }
    let cancelled = false;
    const win = window as Window & { BarcodeDetector?: BarcodeDetectorCtor };
    if (!win.BarcodeDetector) {
      message.warning('当前浏览器不支持摄像头扫码');
      setBindScannerActive(false);
      return;
    }
    const detector = new win.BarcodeDetector({ formats: ['qr_code'] });
    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: 'environment' },
          audio: false
        });
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        bindScanStreamRef.current = stream;
        if (bindVideoRef.current) {
          bindVideoRef.current.srcObject = stream;
          await bindVideoRef.current.play();
          setBindScannerReady(true);
        }
        const timer = window.setInterval(async () => {
          if (!bindVideoRef.current || bindScanLoading) return;
          try {
            const barcodes = await detector.detect(bindVideoRef.current);
            const raw = barcodes.find((item) => item.rawValue)?.rawValue?.trim();
            if (!raw) return;
            window.clearInterval(timer);
            await onScanBindQrRaw(raw);
          } catch {
            // ignore frame errors
          }
        }, 300);
      } catch (err) {
        const msg = err instanceof Error ? err.message : '无法打开摄像头';
        message.error(msg);
        setBindScannerActive(false);
        stopBindScanner();
      }
    };
    start();
    return () => {
      cancelled = true;
      stopBindScanner();
    };
  }, [bindModalOpen, bindScannerActive, bindScanLoading]);

  const onConfirmBind = async () => {
    if (!auth) return;
    if (!bindTargetPubkey) return;
    const archiveIndex = bindScanResult?.archive_no?.trim();
    const qrId = bindScanResult?.qr_id?.trim();
    if (!archiveIndex || !qrId) {
      message.error('请先扫码验签后再确认绑定');
      return;
    }
    setBinding(true);
    try {
      const res = await confirmBind(auth, {
        account_pubkey: bindTargetPubkey,
        archive_index: archiveIndex,
        qr_id: qrId
      });
      message.success(`绑定成功，SFID码：${res.sfid_code}`);
      setBindModalOpen(false);
      setBindTargetPubkey('');
      setBindScanResult(null);
      await refreshList(auth);
    } catch (err) {
      const msg = err instanceof Error ? err.message : '绑定失败';
      message.error(msg);
    } finally {
      setBinding(false);
    }
  };

  const onUnbind = async (pubkey: string) => {
    if (!auth) return;
    Modal.confirm({
      centered: true,
      icon: null,
      title: null,
      content: (
        <div style={{ textAlign: 'center', paddingTop: 8 }}>
          <ExclamationCircleFilled style={{ color: '#faad14', fontSize: 28, marginBottom: 8 }} />
          <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>确认解绑</div>
          <div style={{ color: '#4b5563', lineHeight: 1.6 }}>
            确定要解绑并删除该公民信息吗？
            <br />
            公钥：{pubkey}
          </div>
        </div>
      ),
      okText: '确认解绑',
      okButtonProps: { danger: true },
      cancelText: '取 消',
      footer: (_, { OkBtn, CancelBtn }) => (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
          <CancelBtn />
          <OkBtn />
        </div>
      ),
      onOk: async () => {
        setLoading(true);
        try {
          await unbind(auth, pubkey);
          message.success('解绑成功');
          await refreshList(auth);
        } catch (err) {
          const msg = err instanceof Error ? err.message : '解绑失败';
          message.error(msg);
        } finally {
          setLoading(false);
        }
      }
    });
  };

  const citizenColumns: ColumnsType<CitizenRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v: unknown, _r: CitizenRow, idx: number) => idx + 1
    },
    {
      title: '公钥',
      dataIndex: 'account_pubkey',
      align: 'center'
    },
    {
      title: '档案号',
      dataIndex: 'archive_index',
      align: 'center',
      render: (v: string | undefined) => v ?? '-'
    },
    {
      title: 'SFID码',
      dataIndex: 'sfid_code',
      align: 'center',
      render: (v: string | undefined, row: CitizenRow) => {
        if (v) return v;
        if (capabilities.canBusinessWrite && !row.is_bound) {
          return (
            <Button size="small" type="primary" onClick={() => openSfidTool(row.account_pubkey)}>
              生成
            </Button>
          );
        }
        return '-';
      }
    }
  ];
  if (capabilities.canBusinessWrite) {
    citizenColumns.push({
      title: '操作',
      width: 240,
      align: 'center',
      render: (_v: unknown, row: CitizenRow) => (
        <Space size={8}>
          {row.is_bound ? (
            <Button danger onClick={() => onUnbind(row.account_pubkey)}>
              绑定
            </Button>
          ) : (
            <Button
              type="primary"
              disabled={!row.sfid_code}
              onClick={() => openBindModal(row.account_pubkey)}
            >
              绑定
            </Button>
          )}
          <Button onClick={openStatusScanner} disabled={!row.is_bound}>
            变更
          </Button>
        </Space>
      )
    });
  }

  const institutionDraftRows: CpmsSiteRow[] = institutionSfidDrafts
    .filter((item) => !cpmsSites.some((row) => row.site_sfid === item.site_sfid))
    .map((item) => ({
      site_sfid: item.site_sfid,
      pubkey_1: '-',
      pubkey_2: '-',
      pubkey_3: '-',
      status: undefined,
      created_by: auth?.admin_pubkey || '-',
      created_at: new Date(item.issued_at * 1000).toISOString(),
      updated_by: null,
      updated_at: null
    }));
  const institutionRows = [...institutionDraftRows, ...cpmsSites];
  const previewForSite = (siteSfid: string): GenerateCpmsInstitutionSfidResult | null => {
    const draft = institutionSfidDrafts.find((item) => item.site_sfid === siteSfid);
    if (draft) return draft;
    const fromRow = cpmsSites.find((row) => row.site_sfid === siteSfid)?.init_qr_payload?.trim();
    if (!fromRow) return null;
    return {
      site_sfid: siteSfid,
      issued_at: 0,
      expire_at: 0,
      qr_payload: fromRow
    };
  };

  return (
    <Layout
      style={{
        minHeight: '100vh',
        backgroundImage: `url(${loginBg})`,
        backgroundSize: 'cover',
        backgroundPosition: 'center',
        backgroundRepeat: 'no-repeat',
        backgroundAttachment: 'fixed'
      }}
    >
      <Header
        style={{
          background: 'transparent',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          paddingInline: 24
        }}
      >
        <div style={{ display: 'inline-flex', flexDirection: 'column', lineHeight: 1.1 }}>
          <Typography.Text
            style={{
              color: '#ffffff',
              fontSize: 24,
              fontWeight: 700,
              textShadow: '0 2px 8px rgba(0,0,0,0.45)'
            }}
          >
            中华民族联邦共和国
          </Typography.Text>
          <Typography.Title
            level={4}
            style={{
              color: '#ffffff',
              margin: '2px 0 0 144px',
              fontSize: 20,
              textShadow: '0 2px 8px rgba(0,0,0,0.45)'
            }}
          >
            身份识别码系统
          </Typography.Title>
        </div>
        {auth && (
          <Typography.Text
            style={{
              color: '#ffffff',
              fontSize: 18,
              fontWeight: 600,
              textShadow: '0 2px 8px rgba(0,0,0,0.45)'
            }}
          >
            {resolveHeaderAdminName(auth)}
          </Typography.Text>
        )}
      </Header>

      {bootstrapping ? (
        <Content style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
          <Card bordered={false} style={{ width: 520, maxWidth: '92vw' }} loading />
        </Content>
      ) : !auth ? (
        <Content
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 24
          }}
        >
          <Card
            title={<span style={{ fontSize: 20, fontWeight: 600 }}>扫码登录</span>}
            bordered={false}
            headStyle={{ textAlign: 'center' }}
            style={{
              width: 720,
              maxWidth: '95vw',
              background: 'rgba(255,255,255,0.88)',
              backdropFilter: 'blur(4px)'
            }}
          >
            <Divider />
            <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start', flexWrap: 'wrap' }}>
              <div style={{ flex: '1 1 320px', minWidth: 300 }}>
                <div style={{ display: 'flex', justifyContent: 'center', position: 'relative' }}>
                  <div style={{ width: 220, height: 220, overflow: 'hidden' }}>
                    <div
                      style={{
                        filter: pendingQrLogin ? 'none' : 'blur(2px)',
                        transition: 'filter 0.25s ease'
                      }}
                    >
                      <QRCode value={pendingQrLogin?.login_qr_payload || 'SFID_LOGIN_PENDING'} size={220} />
                    </div>
                  </div>
                </div>
                <Typography.Paragraph type="secondary" style={{ marginTop: 10, marginBottom: 8 }}>
                  {pendingQrLogin
                    ? `二维码有效期至：${new Date(pendingQrLogin.expire_at * 1000).toLocaleTimeString()}`
                    : '二维码未生成'}
                </Typography.Paragraph>
                <Button type="primary" onClick={onCreateQrLogin} loading={challengeLoading}>
                  {pendingQrLogin ? '重新生成二维码' : '生成二维码'}
                </Button>
              </div>
              <div style={{ flex: '1 1 320px', minWidth: 300 }}>
                <Typography.Text strong>扫码窗口</Typography.Text>
                <div
                  style={{
                    marginTop: 8,
                    width: '100%',
                    aspectRatio: '1 / 1',
                    background: '#111827',
                    borderRadius: 8,
                    overflow: 'hidden',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    position: 'relative'
                  }}
                >
                  <video
                    ref={videoRef}
                    style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                    muted
                    playsInline
                  />
                  {!scannerReady && (
                    <Typography.Text style={{ color: '#fff', position: 'absolute' }}>
                      {scannerActive ? '摄像头初始化中...' : '点击下方按钮开始扫码'}
                    </Typography.Text>
                  )}
                </div>
                <Space style={{ marginTop: 10 }}>
                  <Button onClick={onToggleScanner} disabled={scanSubmitting}>
                    {scannerActive ? '停止扫码' : '扫码签名二维码'}
                  </Button>
                </Space>
              </div>
            </div>
          </Card>
        </Content>
      ) : (
        <Content style={{ padding: 24 }}>
          <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: 12 }}>
            <Button danger onClick={onLogout}>
              退出登录
            </Button>
          </div>
          {capabilities.canViewAdminNav && (
            <Card bordered={false} style={{ marginBottom: 16 }}>
              <Space>
                <Button
                  type={activeView === 'citizens' ? 'primary' : 'default'}
                  onClick={() => setActiveView('citizens')}
                >
                  首页
                </Button>
                <Button
                  type={activeView === 'operators' ? 'primary' : 'default'}
                  onClick={async () => {
                    setActiveView('operators');
                    setOperatorPage(1);
                    if (auth) {
                      await refreshOperators(auth);
                      if (capabilities.canReplaceSuperAdmins) {
                        await refreshSuperAdmins(auth);
                      }
                    }
                  }}
                >
                  管理员
                </Button>
                <Button
                  type={activeView === 'institutions' ? 'primary' : 'default'}
                  onClick={async () => {
                    setActiveView('institutions');
                    if (auth) {
                      await refreshCpmsSites(auth);
                    }
                  }}
                >
                  机构管理
                </Button>
                {capabilities.canManageKeyring && (
                  <Button
                    type={activeView === 'keyring' ? 'primary' : 'default'}
                    onClick={async () => {
                      setActiveView('keyring');
                      if (auth) {
                        await refreshKeyringState(auth);
                      }
                    }}
                  >
                    密钥管理
                  </Button>
                )}
              </Space>
            </Card>
          )}
          {activeView === 'operators' && capabilities.canManageOperators ? (
            <>
              <Card
                title="管理员列表"
                bordered={false}
                extra={
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
                          rules={[{ required: true, message: '请输入管理员姓名' }]}
                          style={{ marginBottom: 0 }}
                        >
                          <Input style={{ width: 180 }} placeholder="请输入管理员姓名" />
                        </Form.Item>
                        <Form.Item
                          name="operator_pubkey"
                          rules={[
                            { required: true, message: '请输入管理员公钥' },
                            {
                              validator: async (_rule, value) => {
                                if (!value || isSr25519HexPubkey(String(value))) return;
                                throw new Error('公钥格式必须为 32 字节十六进制');
                              }
                            }
                          ]}
                          style={{ marginBottom: 0 }}
                        >
                          <Input style={{ width: 520 }} placeholder="请输入管理员公钥" />
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
                      {addOperatorOpen ? '确认新增' : '新增管理员'}
                    </Button>
                  </div>
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
                    {
                      title: '操作',
                      width: 220,
                      align: 'center',
                      render: (_v, row) => (
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
                  ]}
                />
              </Card>
              {capabilities.canReplaceSuperAdmins && (
                <Card
                  title="省级超级管理员列表"
                  bordered={false}
                  style={{ marginTop: 16 }}
                  extra={
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
                          { required: true, message: '请输入新超级管理员公钥' },
                          {
                            validator: async (_rule, value) => {
                              if (!value || isSr25519HexPubkey(String(value))) return;
                              throw new Error('公钥格式必须为 32 字节十六进制');
                            }
                          }
                        ]}
                        style={{ marginBottom: 0 }}
                      >
                        <Input style={{ width: 420, maxWidth: '60vw' }} placeholder="新超级管理员公钥" />
                      </Form.Item>
                      <Form.Item style={{ marginBottom: 0 }}>
                        <Button type="primary" htmlType="submit" loading={replaceSuperLoading}>
                          更换超级管理员
                        </Button>
                      </Form.Item>
                    </Form>
                  }
                >
                  <Table<SuperAdminRow>
                    rowKey={(r) => `${r.province}-${r.admin_pubkey}`}
                    loading={superAdminsLoading}
                    dataSource={superAdmins}
                    pagination={{ pageSize: 10 }}
                    columns={[
                      { title: '省份', dataIndex: 'province', align: 'center', width: 160 },
                      { title: '公钥', dataIndex: 'admin_pubkey', align: 'center' },
                      { title: '状态', dataIndex: 'status', align: 'center', width: 120 }
                    ]}
                  />
                </Card>
              )}
            </>
          ) : activeView === 'institutions' && capabilities.canManageInstitutions ? (
            <Card
              title="机构列表"
              bordered={false}
              extra={
                capabilities.canRegisterInstitutions ? (
                  <Space>
                    <Button type="primary" onClick={openInstitutionSfidModal} loading={institutionSfidLoading}>
                      生成身份识别码
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
                    align: 'center',
                    render: (v: string) => {
                      const issued = previewForSite(v);
                      return (
                        <Space size={6}>
                          <span>{v}</span>
                          {issued && (
                            <Button
                              size="small"
                              type="text"
                              icon={<QrcodeOutlined />}
                              onClick={() => setInstitutionQrPreview(issued)}
                            />
                          )}
                        </Space>
                      );
                    }
                  },
                  {
                    title: '状态',
                    width: 110,
                    align: 'center',
                    render: (_v, row) => (row.status === 'PENDING' || !row.status ? '待录入' : row.status)
                  },
                  {
                    title: '公钥1',
                    dataIndex: 'pubkey_1',
                    align: 'center',
                    render: (v: string, row) => (
                      <Space size={6}>
                        <span>{v || '-'}</span>
                        <Button
                          size="small"
                          onClick={() => onUpdateCpmsSiteKey(row, 1)}
                          disabled={!row.status || row.status === 'PENDING'}
                        >
                          更新
                        </Button>
                      </Space>
                    )
                  },
                  {
                    title: '公钥2',
                    dataIndex: 'pubkey_2',
                    align: 'center',
                    render: (v: string, row) => (
                      <Space size={6}>
                        <span>{v || '-'}</span>
                        <Button
                          size="small"
                          onClick={() => onUpdateCpmsSiteKey(row, 2)}
                          disabled={!row.status || row.status === 'PENDING'}
                        >
                          更新
                        </Button>
                      </Space>
                    )
                  },
                  {
                    title: '公钥3',
                    dataIndex: 'pubkey_3',
                    align: 'center',
                    render: (v: string, row) => (
                      <Space size={6}>
                        <span>{v || '-'}</span>
                        <Button
                          size="small"
                          onClick={() => onUpdateCpmsSiteKey(row, 3)}
                          disabled={!row.status || row.status === 'PENDING'}
                        >
                          更新
                        </Button>
                      </Space>
                    )
                  },
                  {
                    title: '登记人',
                    align: 'center',
                    render: (_v, row) => `${row.admin_province || ''}超级管理员`
                  },
                  {
                    title: '操作',
                    width: 300,
                    align: 'center',
                    render: (_v, row) => {
                      const draft = row.status === 'PENDING' || institutionSfidDrafts.some((item) => item.site_sfid === row.site_sfid);
                      const status = row.status || 'ACTIVE';
                      const isDisabled = status === 'DISABLED';
                      return (
                        <Space size={4} wrap>
                          <Button size="small" onClick={() => onDisableCpmsSite(row)} disabled={isDisabled || draft}>
                            禁用
                          </Button>
                          <Button size="small" danger onClick={() => onDeleteCpmsSite(row)} disabled={draft}>
                            删除
                          </Button>
                          <Button size="small" type="primary" onClick={openRegisterScanner} disabled={!draft}>
                            扫码
                          </Button>
                        </Space>
                      );
                    }
                  }
                ]}
              />
            </Card>
          ) : activeView === 'keyring' && capabilities.canManageKeyring ? (
            <Card
              title="签名密钥管理（一主两备）"
              bordered={false}
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
              <Form
                form={keyringForm}
                layout="inline"
                onFinish={onCreateKeyringRotateChallenge}
                style={{ marginBottom: 12, rowGap: 8 }}
              >
                <Form.Item
                  name="initiator_pubkey"
                  rules={[{ required: true, message: '请输入发起轮换的备用公钥' }]}
                >
                  <Input style={{ width: 420, maxWidth: '72vw' }} placeholder="发起轮换的备用公钥" />
                </Form.Item>
                <Form.Item style={{ marginBottom: 0 }}>
                  <Button type="primary" htmlType="submit" loading={keyringActionLoading}>
                    生成轮换二维码
                  </Button>
                </Form.Item>
              </Form>

              <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
                {'流程：生成轮换二维码 -> 备用私钥钱包扫码签名 -> 本页面扫码验签 -> 输入新备用公钥 -> 完成一主两备轮换并异步推链。'}
              </Typography.Paragraph>

              <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start', flexWrap: 'wrap', marginBottom: 12 }}>
                <div style={{ flex: '1 1 320px', minWidth: 300 }}>
                  <div style={{ display: 'flex', justifyContent: 'center' }}>
                    <QRCode value={keyringChallenge?.challenge_text || 'SFID_KEYRING_ROTATE_PENDING'} size={220} />
                  </div>
                  <Typography.Paragraph type="secondary" style={{ marginTop: 10, marginBottom: 8 }}>
                    {keyringChallenge
                      ? `轮换挑战有效期至：${new Date(keyringChallenge.expire_at * 1000).toLocaleTimeString()}`
                      : '尚未生成轮换挑战'}
                  </Typography.Paragraph>
                </div>
                <div style={{ flex: '1 1 320px', minWidth: 300 }}>
                  <Typography.Text strong>扫码窗口</Typography.Text>
                  <div
                    style={{
                      marginTop: 8,
                      width: '100%',
                      aspectRatio: '1 / 1',
                      background: '#111827',
                      borderRadius: 8,
                      overflow: 'hidden',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      position: 'relative'
                    }}
                  >
                    <video
                      ref={keyringVideoRef}
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                      muted
                      playsInline
                    />
                    {!keyringScannerReady && (
                      <Typography.Text style={{ color: '#fff', position: 'absolute' }}>
                        {keyringScannerActive ? '摄像头初始化中...' : '点击下方按钮开始扫码'}
                      </Typography.Text>
                    )}
                  </div>
                  <Space style={{ marginTop: 10 }}>
                    <Button onClick={onToggleKeyringScanner} disabled={keyringScanSubmitting}>
                      {keyringScannerActive ? '停止扫码' : '扫码签名二维码'}
                    </Button>
                  </Space>
                </div>
              </div>

              <Modal
                open={Boolean(keyringSignedPayload)}
                title="签名已通过，确认轮换参数"
                onCancel={() => {
                  setKeyringSignedPayload(null);
                  keyringCommitForm.resetFields();
                }}
                footer={null}
                destroyOnClose
              >
                <Form form={keyringCommitForm} layout="vertical" onFinish={onCommitKeyringRotate}>
                  <Form.Item
                    name="new_backup_pubkey"
                    label="新备用公钥"
                    rules={[
                      { required: true, message: '请输入新备用公钥' },
                      {
                        validator: async (_rule, value) => {
                          if (!value || isSr25519HexPubkey(String(value))) return;
                          throw new Error('公钥格式必须为 32 字节十六进制');
                        }
                      }
                    ]}
                  >
                    <Input placeholder="新备用公钥" />
                  </Form.Item>
                  <Space>
                    <Button
                      onClick={() => {
                        setKeyringSignedPayload(null);
                        keyringCommitForm.resetFields();
                      }}
                    >
                      取消
                    </Button>
                    <Button type="primary" htmlType="submit" loading={keyringCommitLoading}>
                      确认轮换
                    </Button>
                  </Space>
                </Form>
              </Modal>

              <Card size="small" loading={keyringLoading}>
                <Typography.Paragraph style={{ marginBottom: 6 }}>
                  版本：{keyringState?.version ?? '-'}
                </Typography.Paragraph>
                <Typography.Paragraph style={{ marginBottom: 6 }}>
                  主公钥：<Typography.Text code>{keyringState?.main_pubkey ?? '-'}</Typography.Text>
                </Typography.Paragraph>
                <Typography.Paragraph style={{ marginBottom: 6 }}>
                  备用A：<Typography.Text code>{keyringState?.backup_a_pubkey ?? '-'}</Typography.Text>
                </Typography.Paragraph>
                <Typography.Paragraph style={{ marginBottom: 0 }}>
                  备用B：<Typography.Text code>{keyringState?.backup_b_pubkey ?? '-'}</Typography.Text>
                </Typography.Paragraph>
              </Card>
            </Card>
          ) : (
            <>
          <Card
            title={capabilities.isQueryOnly ? '身份信息（只读）' : '身份信息'}
            bordered={false}
            extra={
              <Form layout="inline" onFinish={onSearch}>
                <Form.Item name="keyword" style={{ marginBottom: 0 }}>
                  <Input style={{ width: 420 }} placeholder="请输入公钥、档案号或SFID号" allowClear />
                </Form.Item>
                <Form.Item style={{ marginBottom: 0 }}>
                  <Button htmlType="submit" type="primary" loading={loading}>
                    查询
                  </Button>
                </Form.Item>
              </Form>
            }
          >
            {capabilities.isQueryOnly && (
              <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
                当前为非管理员登录，仅可按档案号、SFID号、公钥查询绑定信息。
              </Typography.Paragraph>
            )}
            <Table<CitizenRow>
              rowKey={(r) => `${r.seq}-${r.account_pubkey}`}
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
        >
          <div
            style={{
              width: '84%',
              maxWidth: 320,
              aspectRatio: '1 / 1',
              background: '#111827',
              borderRadius: 8,
              overflow: 'hidden',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              position: 'relative',
              margin: '14px auto 12px'
            }}
          >
            <video
              ref={bindVideoRef}
              style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              muted
              playsInline
            />
            {!bindScannerReady && (
              <Typography.Text
                style={{
                  color: '#fff',
                  position: 'absolute',
                  cursor: bindScannerActive ? 'default' : 'pointer',
                  userSelect: 'none'
                }}
                onClick={() => {
                  if (!bindScannerActive) onToggleBindScanner();
                }}
              >
                {bindScannerActive ? '摄像头初始化中...' : '点击扫描二维码'}
              </Typography.Text>
            )}
          </div>

          {bindScanResult && (
            <Typography.Paragraph type="secondary">
              已验签通过：site={bindScanResult.site_sfid}，qr_id={bindScanResult.qr_id}
            </Typography.Paragraph>
          )}

          <Form layout="vertical" onFinish={onConfirmBind}>
            <Form.Item label="公钥">
              <Input value={bindTargetPubkey} disabled />
            </Form.Item>
            <Form.Item label="档案号">
              <Input value={bindScanResult?.archive_no ?? ''} disabled />
            </Form.Item>
            <Space>
              <Button onClick={() => setBindModalOpen(false)}>取消</Button>
              <Button htmlType="submit" type="primary" loading={binding} disabled={!bindScanResult}>
                确认绑定
              </Button>
            </Space>
          </Form>
        </Modal>
      )}

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
            background: '#111827',
            borderRadius: 8,
            overflow: 'hidden',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            position: 'relative'
          }}
        >
          <video ref={opVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
          {!opScannerReady && (
            <Typography.Text style={{ color: '#fff', position: 'absolute' }}>摄像头初始化中...</Typography.Text>
          )}
        </div>
      </Modal>

      <Modal
        title="生成身份识别码"
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
              options={sfidCities.map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
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
        </Form>
        {institutionSfidResult && (
          <Space direction="vertical" size={8} style={{ width: '100%' }}>
            <Typography.Text strong>身份识别码：{institutionSfidResult.site_sfid}</Typography.Text>
            <div ref={institutionQrRef} style={{ display: 'flex', justifyContent: 'center' }}>
              <QRCode value={institutionSfidResult.qr_payload} size={220} />
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
            <div ref={institutionQrPreviewRef} style={{ display: 'flex', justifyContent: 'center' }}>
              <QRCode value={institutionQrPreview.qr_payload} size={220} />
            </div>
          </Space>
        )}
      </Modal>

      <Modal
        title="SFID码生成工具"
        open={sfidToolOpen}
        onCancel={() => setSfidToolOpen(false)}
        onOk={() => sfidToolForm.submit()}
        confirmLoading={sfidToolLoading}
        okText="生成并应用"
        cancelText="取消"
        destroyOnClose
      >
        <Form form={sfidToolForm} layout="vertical" onFinish={onGenerateSfidCode}>
          {auth?.admin_province && (
            <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
              当前账号已限定为 {auth.admin_province}，只需选择本省下的市并填写机构信息。
            </Typography.Paragraph>
          )}
          <Form.Item label="用户公钥">
            <Input value={sfidToolTargetPubkey} disabled />
          </Form.Item>
          <Form.Item
            label="A3 主体属性"
            name="a3"
            rules={[{ required: true, message: '请选择A3主体属性' }]}
          >
            <Select
              options={(sfidMeta?.a3_options || []).map((o) => ({ label: `${o.label} (${o.value})`, value: o.value }))}
              placeholder="请选择A3主体属性"
              onChange={(nextA3) => {
                const nextDefault = defaultInstitutionByA3(nextA3);
                sfidToolForm.setFieldsValue({
                  institution: nextDefault,
                  p1: defaultP1ByA3(nextA3)
                });
              }}
            />
          </Form.Item>
          <Form.Item
            label="P1 盈利属性"
            name="p1"
            rules={[{ required: true, message: '请选择盈利属性' }]}
          >
            <Select
              options={[
                { value: '0', label: '非盈利 (0)' },
                { value: '1', label: '盈利 (1)' }
              ]}
              placeholder={p1LockedByA3(sfidToolA3 || '') ? 'P1已按A3自动固定' : '请选择盈利属性'}
              disabled={p1LockedByA3(sfidToolA3 || '')}
            />
          </Form.Item>
          <Form.Item label="省" name="province" rules={[{ required: true, message: '请选择省' }]}>
            <Select
              options={(sfidMeta?.provinces || []).map((p) => ({ label: `${p.name} (${p.code})`, value: p.name }))}
              placeholder="请选择省"
              disabled={Boolean(sfidMeta?.scoped_province)}
              onChange={(provinceName) => {
                sfidToolForm.setFieldsValue({ city: '' });
                void loadSfidCities(provinceName);
              }}
            />
          </Form.Item>
          <Form.Item label="市" name="city" rules={[{ required: true, message: '请选择市' }]}>
            <Select
              loading={sfidCitiesLoading}
              options={sfidCities.map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
              placeholder="请选择该省下的市"
            />
          </Form.Item>
          <Form.Item
            label="机构"
            name="institution"
            rules={[{ required: true, message: '请选择机构类型' }]}
          >
            <Select
              options={sfidInstitutionOptions}
              placeholder={institutionLocked ? '机构已按A3自动固定' : '请选择机构类型'}
              disabled={institutionLocked}
            />
          </Form.Item>
        </Form>
      </Modal>
    </Layout>
  );
}
