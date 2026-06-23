// 中文注释:CID 前端唯一的用户提示入口。
// 所有业务页面都必须通过本文件显示提示,避免英文原始错误、重复提示和连续弹窗。

import { message, Modal } from 'antd';
import type { ArgsProps } from 'antd/es/message';
import type { ModalFuncProps } from 'antd';
import { ApiError, AuthExpiredError } from './http';

const NOTICE_KEY = 'cid-global-notice';
const DUPLICATE_WINDOW_MS = 1200;

let lastNotice: { type: ArgsProps['type']; content: string; at: number } | null = null;

message.config({
  maxCount: 1,
  duration: 2.4,
});

export const notice = {
  success(content: unknown, fallback = '操作成功') {
    show('success', normalizeNoticeText(content, fallback));
  },
  info(content: unknown, fallback = '操作已取消') {
    show('info', normalizeNoticeText(content, fallback));
  },
  warning(content: unknown, fallback = '请检查输入内容') {
    show('warning', normalizeNoticeText(content, fallback));
  },
  error(error: unknown, fallback = '操作失败') {
    const text = normalizeNoticeText(error, fallback);
    if (isCancelText(text)) {
      show('info', text);
      return;
    }
    show('error', text);
  },
  confirm(config: ModalFuncProps) {
    return Modal.confirm({
      okText: '确定',
      cancelText: '取消',
      ...config,
    });
  },
  warningModal(config: ModalFuncProps) {
    return Modal.warning({
      okText: '知道了',
      ...config,
    });
  },
  destroy() {
    message.destroy();
  },
};

export function normalizeNoticeText(input: unknown, fallback = '操作失败'): string {
  const raw = extractMessage(input, fallback).trim();
  if (!raw) return fallback;
  return translateKnownMessage(raw, fallback);
}

function show(type: ArgsProps['type'], content: string) {
  const now = Date.now();
  if (
    lastNotice &&
    lastNotice.type === type &&
    lastNotice.content === content &&
    now - lastNotice.at < DUPLICATE_WINDOW_MS
  ) {
    return;
  }
  lastNotice = { type, content, at: now };
  message.destroy();
  message.open({ key: NOTICE_KEY, type, content });
}

function extractMessage(input: unknown, fallback: string): string {
  if (typeof input === 'string') return input;
  if (input instanceof AuthExpiredError) return '登录已过期，请重新登录';
  if (input instanceof ApiError) return apiErrorText(input, fallback);
  if (input instanceof DOMException) return domExceptionText(input, fallback);
  if (input instanceof Error) return input.message || fallback;
  if (input && typeof input === 'object' && 'message' in input) {
    const messageValue = (input as { message?: unknown }).message;
    if (typeof messageValue === 'string') return messageValue;
  }
  return fallback;
}

function apiErrorText(error: ApiError, fallback: string): string {
  if (error.status === 401) return '登录已过期，请重新登录';
  // 中文注释:403 既可能是通用权限不足,也可能是精确的跨省/跨市业务原因。
  // 后端已返回中文业务原因时优先展示,避免统一映射盖住真实问题。
  if (error.status === 403 && error.message && !isGenericForbiddenText(error.message)) {
    return translateKnownMessage(error.message, fallback);
  }
  if (error.errorCode) {
    const mapped = translateErrorCode(error.errorCode);
    if (mapped) return mapped;
  }
  return error.message || fallback;
}

function isGenericForbiddenText(message: string): boolean {
  const lower = message.trim().toLowerCase();
  return !lower || lower === 'forbidden' || lower === 'permission denied';
}

function domExceptionText(error: DOMException, fallback: string): string {
  if (error.name === 'NotAllowedError') return '已取消通行密钥验证';
  if (error.name === 'AbortError') return '操作已取消';
  if (error.name === 'SecurityError') return '当前页面不允许使用通行密钥';
  if (error.name === 'NotSupportedError') return '当前浏览器不支持通行密钥';
  return error.message || fallback;
}

function translateErrorCode(code: string): string | null {
  const map: Record<string, string> = {
    CID_ADMIN_ACCOUNT_EXISTS_AS_FEDERAL_REGISTRY: '该账户已是联邦注册局管理员，不能重复新增',
    CID_ADMIN_ACCOUNT_EXISTS_AS_CITY_REGISTRY: '该账户已是市注册局管理员，不能重复新增',
    CID_ADMIN_FEDERAL_REGISTRY_PROVINCE_LIMIT_REACHED: '联邦注册局管理员已满 5 人，不能继续新增',
    CID_ADMIN_CITY_REGISTRY_CITY_LIMIT_REACHED: '本市市注册局管理员已满 30 人，不能继续新增',
    CID_ADMIN_PASSKEY_REQUIRED: '请先完成通行密钥验证',
    CID_ADMIN_SECURITY_GRANT_REQUIRED: '请先完成安全确认',
    CID_AUTH_FORBIDDEN: '权限不足，无法执行该操作',
    CID_AUTH_UNAUTHORIZED: '请先登录管理员账户',
    CID_REQUEST_INVALID: '请求内容不正确，请检查后重试',
    CID_RESOURCE_NOT_FOUND: '数据不存在',
    CID_RESOURCE_CONFLICT: '数据状态冲突，请刷新后重试',
    CID_RESOURCE_EXPIRED: '请求已过期，请重新操作',
    CID_RATE_LIMITED: '请求过于频繁，请稍后重试',
    CID_SERVICE_UNAVAILABLE: '服务暂不可用，请稍后重试',
  };
  return map[code] ?? null;
}

function translateKnownMessage(raw: string, fallback: string): string {
  const text = raw.trim();
  const lower = text.toLowerCase();
  const exact = KNOWN_ENGLISH_MESSAGES[lower];
  if (exact) return exact;

  if (lower.includes('the operation either timed out or was not allowed')) {
    return '已取消通行密钥验证';
  }
  if (lower.includes('notallowederror')) return '已取消通行密钥验证';
  if (lower.includes('aborterror')) return '操作已取消';
  if (lower.includes('notsupportederror')) return '当前浏览器不支持通行密钥';
  if (lower.includes('securityerror')) return '当前页面不允许使用通行密钥';
  if (lower.includes('networkerror') || lower.includes('failed to fetch')) {
    return '无法连接服务器，请检查网络或服务状态';
  }
  if (lower.startsWith('request failed')) return fallback;
  if (text === 'admin_display_name is required') return '请输入管理员姓名';
  if (lower.endsWith(' is required')) return requiredFieldText(lower);
  if (lower.startsWith('invalid ')) return invalidFieldText(lower);
  if (lower.startsWith('unknown ')) return unknownFieldText(lower);
  if (lower.includes(' query failed')) return '查询失败，请稍后重试';
  if (lower.includes(' create failed')) return '创建失败，请稍后重试';
  if (lower.includes(' update failed')) return '更新失败，请稍后重试';
  if (lower.includes(' delete failed')) return '删除失败，请稍后重试';
  if (lower.includes('write file failed')) return '文件写入失败';
  if (lower.includes('create dir failed')) return '目录创建失败';
  if (lower.includes('file not found')) return '文件不存在';

  // 中文注释:最后兜底。用户可见提示不允许裸露后端英文原文。
  if (isPlainEnglishLike(text)) return fallback || '操作失败';

  return text.replace(/Passkey/g, '通行密钥') || fallback;
}

function isCancelText(text: string): boolean {
  return text.includes('已取消') || text.includes('操作已取消');
}

const KNOWN_ENGLISH_MESSAGES: Record<string, string> = {
  'passkey required': '请先完成通行密钥验证',
  'security grant required': '请先完成安全确认',
  'citizen wallet confirmation required first': '请先完成公民钱包确认',
  'admin auth required': '请先登录管理员账户',
  'federal admin required': '需要联邦注册局管理员权限',
  'initial federal admin required': '需要初始联邦注册局管理员权限',
  'province scope required': '缺少省级权限范围',
  'admin province scope missing': '缺少管理员省级权限范围',
  'admin city scope missing': '缺少管理员市级权限范围',
  'city out of scope': '城市超出当前管理员权限范围',
  'out of admin scope': '超出当前管理员权限范围',
  'rate limit exceeded': '请求过于频繁，请稍后重试',
  'binding not vote eligible': '该绑定不具备投票资格',
  'binding not found': '未找到绑定记录',
  'citizen record not found': '未找到公民记录',
  'citizen record is not bound': '公民记录尚未绑定',
  'citizen status is stale': '公民状态已过期，请重新同步',
  'archive_no already bound': '该档案号已绑定',
  'wallet_pubkey already bound': '该钱包公钥已绑定',
  'challenge expired': '签名请求已过期，请重新操作',
  'invalid signature hex': '签名格式无效',
  'institution not found': '机构不存在',
  'account not found': '账户不存在',
  'document not found': '资料不存在',
  'cpms site not found': 'CPMS 授权不存在',
  'invalid target status': '目标状态无效',
  'field too long': '字段长度超出限制',
  'reason too long': '原因长度超出限制',
  'file is empty': '文件为空',
  'invalid doc_type': '资料类型无效',
  'invalid page cursor': '分页游标无效',
};

const FIELD_LABELS: Record<string, string> = {
  address: '地址',
  account_pubkey: '账户公钥',
  wallet_address: '钱包地址',
  wallet_pubkey: '钱包公钥',
  archive_no: '档案号',
  identity_code: '身份ID',
  name: '名称',
  province: '省份',
  city: '城市',
  institution: '机构',
  cid_full_name: '机构全称',
  cid_number: '身份ID号',
  file: '文件',
  domain: '域名',
  origin: '来源',
  session_id: '会话',
  identity_qr: '身份二维码',
  admin_account: '管理员账户',
  signer_pubkey: '签名账户',
  signature: '签名',
  payload_hash: '载荷哈希',
  bind_nonce: '绑定随机数',
  binding_seed: '绑定种子',
  vote_nonce: '投票随机数',
  snapshot_nonce: '快照随机数',
};

function requiredFieldText(lower: string): string {
  const field = lower.replace(/ is required$/, '').trim();
  const label = FIELD_LABELS[field];
  return label ? `请填写${label}` : '请填写必填项';
}

function invalidFieldText(lower: string): string {
  const field = lower.replace(/^invalid /, '').trim();
  const label = FIELD_LABELS[field];
  return label ? `${label}无效` : '输入内容无效';
}

function unknownFieldText(lower: string): string {
  const field = lower.replace(/^unknown /, '').trim();
  const label = FIELD_LABELS[field];
  return label ? `${label}不存在` : '数据不存在';
}

function isPlainEnglishLike(text: string): boolean {
  return /[A-Za-z]/.test(text) && !/[\u4e00-\u9fff]/.test(text);
}
