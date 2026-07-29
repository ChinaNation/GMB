import { assertAccountId, assertCidNumber } from '../shared/ids';
import { HttpError } from '../shared/http';

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

/// 从 D1 上传事实读取该内容唯一允许删除的 R2 对象清单。
///
/// 当前广场 R2 只保存规范 manifest；Images/Stream 使用各自 provider id。删除路径必须
/// 同时校验 JSON 形状和由发布账户、post_id 生成的精确对象键，禁止接受任意 R2 键，也
/// 禁止对象清单损坏时返回空数组后继续删除 D1 索引。
export function uploadObjectKeys(row: {
  account_id: string;
  post_id: string;
  object_keys_json: string;
}): string[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(row.object_keys_json);
  } catch {
    throw new HttpError(409, 'upload_object_keys_invalid', '上传对象清单不是合法 JSON');
  }
  let expected: string[];
  try {
    expected = buildObjectKeyPlan(row.account_id, row.post_id).object_keys;
  } catch {
    throw new HttpError(409, 'upload_object_keys_invalid', '上传对象清单的发布事实不合法');
  }
  if (
    !Array.isArray(parsed) ||
    parsed.length !== expected.length ||
    parsed.some((value, index) => value !== expected[index])
  ) {
    throw new HttpError(409, 'upload_object_keys_invalid', '上传对象清单与发布事实不一致');
  }
  return [...expected];
}
