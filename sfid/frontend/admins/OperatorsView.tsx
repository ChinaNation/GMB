// 中文注释:市级管理员入口复用联邦管理员/市级管理员统一治理视图。
// 写操作由 ShengAdminsView 内部统一走 Passkey + 冷钱包签名。

import { ShengAdminsView } from './ShengAdminsView';

export function OperatorsView() {
  return <ShengAdminsView mode="system-settings" />;
}
