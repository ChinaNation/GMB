import type { ChainStatus } from '../../../types';

type Props = {
  chain: ChainStatus;
  nodeRunning: boolean;
};

export function ChainSection({ chain, nodeRunning }: Props) {
  return (
    <section className="section">
      <h2>区块</h2>
      <p>当前高度: {chain.blockHeight ?? '-'}</p>
      <p>区块状态: {nodeRunning ? '同步中' : '暂停同步'}</p>
    </section>
  );
}
