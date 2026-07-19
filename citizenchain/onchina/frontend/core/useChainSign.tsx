// 链交易冷签 hook(ADR-031 D6/D7)。
// 占号 / 吊销 / 身份上链的 prepare 都返回 { request_id, sign_request }:
// 本 hook 弹出管理员 CitizenWallet 请求二维码，扫描签名响应并解析
// { signer_pubkey, signature }，再由 OnChina 统一组装提交。CitizenWallet 只签名一次并展示响应二维码。
//
// 用法:
//   const { signChain, chainSignModal } = useChainSign();
//   const { signer_pubkey, signature } = await signChain(request_id, sign_request);
//   ...在 JSX 末尾渲染 {chainSignModal}

import { useCallback, useState, type ReactNode } from 'react';
import type { AdminAuth } from '../auth/types';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from './CitizenSignatureModal';
import { notice } from '../utils/notice';
import { adminHeaders, request } from '../utils/http';

export type ChainSignResult = { signer_pubkey: string; signature: string };

/** 所有 OnChina 链交易 prepare 接口共用的最小返回结构。 */
export type ChainSignPrepare = { request_id: string; sign_request: string };

/**
 * 所有 OnChina 链交易共用的提交结果。
 * `citizen` 只在公民占号用途返回，其他业务不得为相同签名响应另建提交实现。
 */
export type ChainSubmitResult<TCitizen = unknown> = {
  purpose: string;
  cid_number: string;
  tx_hash: string;
  block_number?: number | null;
  citizen?: TCitizen | null;
};

/**
 * 统一链交易提交入口：OnChina 回扫 CitizenWallet 的一次签名响应后，
 * 由后端统一验签、dry-run、提交并等待进块。
 */
export async function submitChainSign<TCitizen = unknown>(
  auth: AdminAuth,
  requestId: string,
  signerPubkey: string,
  signature: string,
): Promise<ChainSubmitResult<TCitizen>> {
  return request<ChainSubmitResult<TCitizen>>('/api/v1/admin/chain/submit', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify({
      request_id: requestId,
      signer_pubkey: signerPubkey,
      signature,
    }),
  });
}

type PendingChainSign = {
  requestId: string;
  signRequest: string;
  resolve: (value: ChainSignResult) => void;
  reject: (reason?: unknown) => void;
};

export interface UseChainSignResult {
  /** 展示 sign_request 二维码，扫描 CitizenWallet 响应并解析 signer_pubkey / signature。 */
  signChain: (requestId: string, signRequest: string) => Promise<ChainSignResult>;
  /** 需在组件 JSX 中渲染的扫码签名弹窗。 */
  chainSignModal: ReactNode;
}

export function useChainSign(title = '管理员公民钱包签名'): UseChainSignResult {
  const [pending, setPending] = useState<PendingChainSign | null>(null);
  const [scanning, setScanning] = useState(false);

  const signChain = useCallback(
    (requestId: string, signRequest: string) =>
      new Promise<ChainSignResult>((resolve, reject) => {
        setPending({ requestId, signRequest, resolve, reject });
      }),
    [],
  );

  const onDetected = useCallback(
    (raw: string) => {
      if (!pending) return;
      setScanning(true);
      try {
        const signed = parseSignedReceiptPayload(raw, pending.requestId);
        if (signed.challenge_id !== pending.requestId) {
          throw new Error('签名响应与当前请求不匹配');
        }
        if (!signed.signer_pubkey) {
          throw new Error('签名响应缺少 signer_pubkey');
        }
        pending.resolve({ signer_pubkey: signed.signer_pubkey, signature: signed.signature });
        setPending(null);
      } catch (err) {
        pending.reject(err);
        setPending(null);
        notice.error(err, '');
      } finally {
        setScanning(false);
      }
    },
    [pending],
  );

  const onCancel = useCallback(() => {
    pending?.reject(new Error('已取消签名'));
    setPending(null);
    setScanning(false);
  }, [pending]);

  const chainSignModal = (
    <CitizenSignatureModal
      title={title}
      open={!!pending}
      onCancel={onCancel}
      qrTitle="链交易签名二维码"
      qrValue={pending?.signRequest ?? undefined}
      qrHint="使用管理员公民钱包扫码签名"
      scannerHint="扫描公民钱包生成的签名响应二维码"
      scannerDisabled={scanning}
      scannerLoading={scanning}
      onDetected={onDetected}
      onScannerError={(msg) => notice.error(msg)}
    />
  );

  return { signChain, chainSignModal };
}
