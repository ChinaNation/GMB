// 身份信息展示：节点角色 + P2P 地址。
import type { NodeIdentity } from '../../../types';

type Props = {
  identity: NodeIdentity;
};

export function IdentitySection({ identity }: Props) {
  return (
    <section className="section">
      <h2>身份</h2>
      <p>节点角色: {identity.role ?? '全节点'}</p>
      <p>P2P地址: {identity.peerId ? `/p2p/${identity.peerId}` : '-'}</p>
    </section>
  );
}
