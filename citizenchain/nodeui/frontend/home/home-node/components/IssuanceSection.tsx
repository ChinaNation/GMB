import type { TotalIssuance } from '../../../types';

type Props = {
  issuance: TotalIssuance;
};

export function IssuanceSection({ issuance }: Props) {
  return (
    <section className="section">
      <h2>发行</h2>
      <p>全链发行总额: {issuance.totalIssuance ? `${issuance.totalIssuance} 元` : '-'}</p>
    </section>
  );
}
