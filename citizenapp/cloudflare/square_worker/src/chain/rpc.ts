import type { Env } from '../types';
import { HttpError } from '../shared/http';

const systemEventsStorageKey =
  '0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7';

interface JsonRpcResponse<T> {
  result?: T;
  error?: {
    code: number;
    message: string;
  };
}

export async function fetchSystemEventsAtBlock(
  env: Env,
  blockHashHex: string
): Promise<string> {
  const rpcUrl = env.SQUARE_CHAIN_RPC_URL;
  if (!rpcUrl) {
    throw new HttpError(503, 'chain_rpc_not_configured', '广场链上确认 RPC 未配置');
  }

  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'state_getStorage',
      params: [systemEventsStorageKey, blockHashHex]
    })
  });

  if (!response.ok) {
    throw new HttpError(502, 'chain_rpc_http_failed', '读取链上事件失败');
  }

  const data = (await response.json()) as JsonRpcResponse<string | null>;
  if (data.error) {
    throw new HttpError(502, 'chain_rpc_error', data.error.message);
  }
  if (!data.result) {
    throw new HttpError(404, 'chain_events_not_found', '指定区块没有 System.Events');
  }
  return data.result;
}
