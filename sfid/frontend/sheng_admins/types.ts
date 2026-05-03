// 中文注释:SFID 省管理员三槽枚举(ADR-008)。
// Main = 当前主管理员,Backup1/Backup2 = 备用管理员槽位。
// 三槽各自独立签名密钥,互不共享;主备交换的链上写入后续集中到更换省管理员功能。

export type ShengSlot = 'Main' | 'Backup1' | 'Backup2';

export const ShengSlotLabel: Record<ShengSlot, string> = {
  Main: '主槽',
  Backup1: '备份槽 1',
  Backup2: '备份槽 2',
};

export const SHENG_SLOTS: readonly ShengSlot[] = ['Main', 'Backup1', 'Backup2'];
