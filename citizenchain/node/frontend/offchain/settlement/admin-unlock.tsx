// 清算行机构管理员列表子页(机构详情下的折叠卡片入口)。
//
// 简洁版展示:N 位管理员 SS58 + 内部投票阈值。
// 复杂的"已激活/未激活"区分需要 wumin 冷钱包"解密"流程,本任务暂不集成,
// 沿用 governance/AdminListPage 模式即可。

type Props = {
  sfidId: string;
  admins: string[];
  threshold: number;
  adminCount: number;
  onBack: () => void;
};

export function ClearingBankAdminListPage({
  sfidId,
  admins,
  threshold,
  adminCount,
  onBack,
}: Props) {
  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>管理员列表（{admins.length} 人,阈值 {threshold}/{adminCount}）</h2>
        <code className="admin-card-address">{sfidId}</code>
      </div>

      {admins.length === 0 ? (
        <p className="no-data">机构尚未配置管理员</p>
      ) : (
        <div className="admin-grid">
          {admins.map((ss58, idx) => (
            <div key={ss58} className="metric-card admin-card">
              <span className="admin-card-index">{idx + 1}</span>
              <code className="admin-card-address">{ss58}</code>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
