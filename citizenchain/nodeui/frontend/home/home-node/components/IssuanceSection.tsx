import type { TotalIssuance, TotalStake } from '../../../types';

type Props = {
  issuance: TotalIssuance;
  stake: TotalStake;
};

export function IssuanceSection({ issuance, stake }: Props) {
  return (
    <section className="section mining-section">
      <h2>发行</h2>
      <div className="mining-income-grid">
        <div className="metric-card">
          <div className="metric-label">全链发行总额</div>
          <div className="metric-value">
            {issuance.totalIssuance ? (
              <>
                {issuance.totalIssuance}元
                <span className="metric-value-currency">（公民币）</span>
              </>
            ) : (
              '-'
            )}
          </div>
        </div>
        <div className="metric-card">
          <div className="metric-label">永久质押金额</div>
          <div className="metric-value">
            {stake.totalStake ? (
              <>
                {stake.totalStake}元
                <span className="metric-value-currency">（公民币）</span>
              </>
            ) : (
              '-'
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
