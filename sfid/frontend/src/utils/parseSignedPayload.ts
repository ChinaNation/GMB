// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 4)
// 统一的签名二维码 payload 解析工具,供 citizens (绑定/解绑) 和 keyring (密钥轮换) 共享。
// 原 App.tsx / KeyringView.tsx 各有一份本地副本,本步骤抽到此处并统一 import。

// 中文注释:登录签名 payload(任务卡 20260408-sfid-frontend-app-tsx-split 步 5 迁入)
export type SignedLoginPayload = {
  challenge_id: string;
  session_id?: string;
  admin_pubkey: string;
  signer_pubkey?: string;
  signature: string;
};

export function parseSignedLoginPayload(
  raw: string,
  fallbackChallengeId: string,
): SignedLoginPayload {
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

export type KeyringSignedPayload = {
  challenge_id: string;
  signature: string;
};

export function parseKeyringSignedPayload(
  raw: string,
  fallbackChallengeId: string,
): KeyringSignedPayload {
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
    signature: trimmed,
  };
}
