import type { Env } from '../types';
import { HttpError } from '../shared/http';

const systemEventsStorageKey =
  '0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7';

/// 单次链 RPC 请求超时；Worker 不在请求内自动重试，避免重复广播已签名交易。
const CHAIN_RPC_TIMEOUT_MS = 3000;
/// System.Events 可能较大，但必须给 Worker 内存设置硬边界。
const CHAIN_RPC_MAX_RESPONSE_BYTES = 4 * 1024 * 1024;

type ChainRpcMethod = 'state_getStorage' | 'author_submitExtrinsic';
type JsonRpcId = number | string;

interface ChainRpcConfig {
  url: string;
  accessClientId: string;
  accessClientSecret: string;
}

export async function fetchSystemEventsAtBlock(
  env: Env,
  blockHashHex: string
): Promise<string> {
  const result = await fetchChainStorage(env, systemEventsStorageKey, blockHashHex);
  if (!result) {
    throw new HttpError(404, 'chain_events_not_found', '指定区块没有 System.Events');
  }
  return result;
}

export async function fetchChainStorage(
  env: Env,
  storageKeyHex: string,
  blockHashHex?: string
): Promise<string | null> {
  const params = blockHashHex ? [storageKeyHex, blockHashHex] : [storageKeyHex];
  const result = await callChainRpc(env, 'state_getStorage', params, 1);
  if (result !== null && typeof result !== 'string') {
    throw new HttpError(502, 'chain_rpc_invalid_response', '链服务节点返回了无效存储数据');
  }
  return result;
}

/// 内部链 RPC 只允许代码声明的固定方法，不接受 App 传入 method 或 RPC URL。
export async function callChainRpc(
  env: Env,
  method: ChainRpcMethod,
  params: string[],
  requestId: JsonRpcId
): Promise<unknown> {
  const config = requireChainRpcConfig(env);
  const timeoutSignal = AbortSignal.timeout(CHAIN_RPC_TIMEOUT_MS);
  let response: Response;
  try {
    response = await fetch(config.url, {
      method: 'POST',
      headers: {
        accept: 'application/json',
        'content-type': 'application/json',
        'CF-Access-Client-Id': config.accessClientId,
        'CF-Access-Client-Secret': config.accessClientSecret
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: requestId,
        method,
        params
      }),
      // workerd 只支持 manual/follow；manual 可阻止 Access 服务令牌被带到其他主机。
      redirect: 'manual',
      signal: timeoutSignal
    });
  } catch (error) {
    if (timeoutSignal.aborted || isTimeoutError(error)) {
      throw new HttpError(504, 'chain_rpc_timeout', '链服务节点请求超时');
    }
    throw new HttpError(502, 'chain_rpc_transport_failed', '无法连接链服务节点');
  }

  if (!response.ok) {
    throw new HttpError(502, 'chain_rpc_http_failed', '链服务节点 HTTP 请求失败');
  }

  let payload: unknown;
  try {
    payload = await readBoundedJson(response);
  } catch (error) {
    if (error instanceof HttpError) {
      throw error;
    }
    if (timeoutSignal.aborted || isTimeoutError(error)) {
      throw new HttpError(504, 'chain_rpc_timeout', '链服务节点请求超时');
    }
    throw new HttpError(502, 'chain_rpc_transport_failed', '读取链服务节点响应失败');
  }
  if (!isRecord(payload) || payload.jsonrpc !== '2.0' || payload.id !== requestId) {
    throw new HttpError(502, 'chain_rpc_invalid_response', '链服务节点返回了无效 JSON-RPC 响应');
  }
  if (payload.error !== undefined && payload.error !== null) {
    const message = rpcErrorMessage(payload.error);
    throw new HttpError(502, 'chain_rpc_rejected', message);
  }
  if (!Object.hasOwn(payload, 'result')) {
    throw new HttpError(502, 'chain_rpc_invalid_response', '链服务节点响应缺少 result');
  }
  return payload.result;
}

/// bootstrap 只据此判断 relay 是否可用，不暴露具体缺失项或 Secret 内容。
export function isChainRpcConfigured(env: Env): boolean {
  try {
    requireChainRpcConfig(env);
    return true;
  } catch {
    return false;
  }
}

function requireChainRpcConfig(env: Env): ChainRpcConfig {
  const rawUrl = env.CHAIN_URL?.trim();
  if (!rawUrl) {
    throw new HttpError(503, 'chain_rpc_not_configured', '链服务节点 RPC 未配置');
  }

  let parsedUrl: URL;
  try {
    parsedUrl = new URL(rawUrl);
  } catch {
    throw new HttpError(503, 'chain_rpc_invalid_config', '链服务节点 RPC 配置无效');
  }
  if (
    parsedUrl.protocol !== 'https:' ||
    parsedUrl.username !== '' ||
    parsedUrl.password !== '' ||
    parsedUrl.hash !== ''
  ) {
    throw new HttpError(503, 'chain_rpc_invalid_config', '链服务节点 RPC 必须使用受保护的 HTTPS 地址');
  }

  const accessClientId = env.CHAIN_ID?.trim();
  const accessClientSecret = env.CHAIN_SECRET?.trim();
  if (!accessClientId || !accessClientSecret) {
    throw new HttpError(503, 'chain_rpc_access_not_configured', '链服务节点 Access 服务令牌未配置');
  }

  return {
    url: parsedUrl.toString(),
    accessClientId,
    accessClientSecret
  };
}

async function readBoundedJson(response: Response): Promise<unknown> {
  const declaredLength = response.headers.get('content-length');
  if (declaredLength) {
    const parsedLength = Number.parseInt(declaredLength, 10);
    if (Number.isFinite(parsedLength) && parsedLength > CHAIN_RPC_MAX_RESPONSE_BYTES) {
      await response.body?.cancel();
      throw new HttpError(502, 'chain_rpc_response_too_large', '链服务节点响应超过大小限制');
    }
  }

  if (!response.body) {
    throw new HttpError(502, 'chain_rpc_invalid_response', '链服务节点返回了空响应');
  }

  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let totalBytes = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }
      totalBytes += value.byteLength;
      if (totalBytes > CHAIN_RPC_MAX_RESPONSE_BYTES) {
        await reader.cancel();
        throw new HttpError(502, 'chain_rpc_response_too_large', '链服务节点响应超过大小限制');
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }

  const bytes = new Uint8Array(totalBytes);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }

  try {
    return JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes)) as unknown;
  } catch {
    throw new HttpError(502, 'chain_rpc_invalid_response', '链服务节点返回了无效 JSON');
  }
}

function rpcErrorMessage(value: unknown): string {
  if (isRecord(value) && typeof value.message === 'string') {
    const message = value.message.trim().slice(0, 512);
    if (message) {
      return message;
    }
  }
  return '链服务节点拒绝了 JSON-RPC 请求';
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isTimeoutError(error: unknown): boolean {
  return error instanceof DOMException && error.name === 'TimeoutError';
}
