export interface ObjectKeyPlan {
  manifest_object_key: string;
  object_keys: string[];
}

export function sanitizeOwnerAccount(ownerAccount: string): string {
  return ownerAccount.replace(/[^a-zA-Z0-9._-]/g, '_');
}

/// 公开资料包 R2 object key：一个钱包账户一份 profile.json。
export function profileObjectKey(ownerAccount: string): string {
  return `profile/${sanitizeOwnerAccount(ownerAccount)}/profile.json`;
}

/// 头像/背景 R2 object key 前缀；本人上传的头像与背景对象必须落在此前缀下。
export function profileAssetPrefix(ownerAccount: string): string {
  return `profile/${sanitizeOwnerAccount(ownerAccount)}/`;
}

export function buildObjectKeyPlan(
  ownerAccount: string,
  postId: string
): ObjectKeyPlan {
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  const basePath = `square/${safeOwner}/posts/${postId}`;
  const manifestObjectKey = `${basePath}/manifest.json`;

  return {
    manifest_object_key: manifestObjectKey,
    object_keys: [manifestObjectKey]
  };
}
