// 链交易冷签 hook(ADR-031 D6/D7)。
// 占号 / 吊销 / 身份上链的 prepare 都返回 { request_id, sign_request }:
// 本 hook 弹出管理员冷钱包二维码,扫描签名响应,解析出 { signer_pubkey, signature },
// 再交给调用方 POST 到 chain/submit 由 onchina 组装提交。QR 只签不提交,冷钱包边界不变。
//
// 用法:
//   const { signChain, chainSignModal } = useChainSign();
//   const { signer_pubkey, signature } = await signChain(request_id, sign_request);
//   ...在 JSX 末尾渲染 {chainSignModal}

import { useCallback, useState, type ReactNode } from 'react';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from './CitizenSignatureModal';
import { notice } from '../utils/notice';

export type ChainSignResult = { signer_pubkey: string; signature: string };

type PendingChainSign = {
  requestId: string;
  signRequest: string;
  resolve: (value: ChainSignResult) => void;
  reject: (reason?: unknown) => void;
};

export interface UseChainSignResult {
  /** 展示 sign_request 二维码,扫描冷钱包响应,解析出 signer_pubkey / signature。 */
  signChain: (requestId: string, signRequest: string) => Promise<ChainSignResult>;
  /** 需在组件 JSX 中渲染的扫码签名弹窗。 */
  chainSignModal: ReactNode;
}

export function useChainSign(title = '管理员冷钱包签名'): UseChainSignResult {
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
      qrHint="使用管理员冷钱包扫码签名"
      scannerHint="扫描冷钱包生成的签名响应二维码"
      scannerDisabled={scanning}
      scannerLoading={scanning}
      onDetected={onDetected}
      onScannerError={(msg) => notice.error(msg)}
    />
  );

  return { signChain, chainSignModal };
}
