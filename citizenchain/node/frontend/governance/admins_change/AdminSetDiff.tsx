import { hexToSs58 } from '../../shared/ss58';

type Props = {
  currentAdmins: string[];
  admins: string[];
};

export function AdminSetDiff({ currentAdmins, admins }: Props) {
  const current = new Set(currentAdmins.map((item) => item.toLowerCase()));
  const next = new Set(admins.map((item) => item.toLowerCase()));
  const added = admins.filter((item) => !current.has(item.toLowerCase()));
  const removed = currentAdmins.filter((item) => !next.has(item.toLowerCase()));

  return (
    <div className="admin-set-diff">
      <div>
        <strong>新增</strong>
        {added.length === 0 ? <p>无</p> : added.map((item) => <code key={item}>{hexToSs58(item)}</code>)}
      </div>
      <div>
        <strong>移除</strong>
        {removed.length === 0 ? <p>无</p> : removed.map((item) => <code key={item}>{hexToSs58(item)}</code>)}
      </div>
    </div>
  );
}

