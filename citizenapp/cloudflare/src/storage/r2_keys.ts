import { assertAccountId, assertCidNumber } from '../shared/ids';

export interface ObjectKeyPlan {
  manifest_object_key: string;
  object_keys: string[];
}

/// 帖子 manifest 等发布内容的 R2 对象键使用 AccountId 的 64 位小写 hex 主体
/// (发布签名者路径,与 post 行保留的 account_id 列一致);入口先执行全格式严格校验。
export function accountIdPathSegment(accountId: string): string {
  return assertAccountId(accountId).slice(2);
}

/// 公开资料包 R2 object key：一份 profile.json 挂身份主键 `cid_number`(换绑不丢)。
export function profileObjectKey(cidNumber: string): string {
  return `profile/${assertCidNumber(cidNumber)}/profile.json`;
}

/// 头像/背景 R2 object key 前缀(按身份主键 cid_number);本人上传的头像与背景对象必须落在此前缀下。
export function profileAssetPrefix(cidNumber: string): string {
  return `profile/${assertCidNumber(cidNumber)}/`;
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
