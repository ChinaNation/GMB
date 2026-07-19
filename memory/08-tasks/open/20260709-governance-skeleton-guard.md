# 任务卡：固定治理骨架与 Node Guard

## 当前状态

- 状态：已完成，成果并入 `20260628-institution-admin-field-model-onchain.md`
- 当前模型：固定机构岗位定义和法定席位归 entity；admins 独立保存 `admin_account + family_name + given_name` 人员集合。固定治理骨架可约束任职集合与 admins 一致，但不得由岗位反向派生管理员。

## 完成结果

- Node Guard 与创世共用 `runtime/primitives/src/governance_skeleton.rs` 的五类固定机构、岗位代码和席位协议。
- NRC、PRC、PRB、NJD、FRG 的机构码、活动状态、岗位目录、岗位名称、法定席位数和任职一致性均纳入守卫。
- NJD 固定为 7 名护宪大法官、1 名首席大法官、2 名次席大法官、5 名大法官；护宪终审成员从“护宪大法官”有效岗位任职取得。
- FRG 固定为 43 个省专员岗位、每岗 5 席；不存在旧虚拟省组 storage。
- 守卫允许依法轮换任职账户、任职来源和合法任期，拒绝岗位缺失、改名、停用、额外岗位、席位变化、重复占席、畸形 key 和 SCALE 尾随字节。
- Node Guard 不读取法定代表人，也不要求创世写入法定代表人。
- 旧机构管理员资料内嵌模型及 public/private admins 直接变更模型已经废弃，不得恢复。

## 验收归属

详细测试、真实节点哈希和后续跨端验收统一记录在主任务卡。重新创世按当前用户指令暂缓。
