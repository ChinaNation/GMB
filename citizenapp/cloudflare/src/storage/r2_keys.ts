import { assertAccountId } from '../shared/ids';

export interface ObjectKeyPlan {
  manifest_object_key: string;
  object_keys: string[];
}

/// R2 对象键只使用 AccountId 的 64 位小写 hex 主体；入口先执行全格式严格校验。
export function accountIdPathSegment(accountId: string): string {
  return assertAccountId(accountId).slice(2);
}

/// 公开资料包 R2 object key：一个钱包账户一份 profile.json。
export function profileObjectKey(accountId: string): string {
  return `profile/${accountIdPathSegment(accountId)}/profile.json`;
}

/// 头像/背景 R2 object key 前缀；本人上传的头像与背景对象必须落在此前缀下。
export function profileAssetPrefix(accountId: string): string {
  return `profile/${accountIdPathSegment(accountId)}/`;
}

export function buildObjectKeyPlan(
  accountId: string,
  postId: string
): ObjectKeyPlan {
  const accountSegment = accountIdPathSegment(accountId);
  const basePath = `square/${accountSegment}/posts/${postId}`;
  const manifestObjectKey = `${basePath}/manifest.json`;

  return {
    manifest_object_key: manifestObjectKey,
    object_keys: [manifestObjectKey]
  };
}
