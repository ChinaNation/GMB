// 中文注释:SFID 省管理员三槽枚举(ADR-008)。
// Main = 链上 trust anchor 首激活方,Backup1/Backup2 = main 在册期间通过 roster 加挂的备份。
// 三槽各自独立签名密钥,互不共享。

export type ShengSlot = 'Main' | 'Backup1' | 'Backup2';

export const ShengSlotLabel: Record<ShengSlot, string> = {
  Main: '主槽',
  Backup1: '备份槽 1',
  Backup2: '备份槽 2',
};

export const SHENG_SLOTS: readonly ShengSlot[] = ['Main', 'Backup1', 'Backup2'];
