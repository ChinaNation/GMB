// "添加清算行"页:输入 sfid_id 直查 + 关键字搜索 SFID 端候选机构。
//
// 输入框做两件事:
// 1. 直接输 sfid_id(如 "SFR-12345-...") + 回车 → 直接进入综合判定
// 2. 输入机构名/部分 sfid_id → 实时调 SFID `/clearing-banks/eligible-search`
//    返回候选机构列表,点击进入综合判定

import { useEffect, useState } from 'react';
import { api, sanitizeError } from '../api';
import type { EligibleClearingBankCandidate } from './clearing-bank-types';

type Props = {
  onBack: () => void;
  onSelectCandidate: (c: EligibleClearingBankCandidate) => void;
  onSelectKnownSfid: (sfidId: string) => void;
};

export function ClearingBankAddPage({ onBack, onSelectCandidate, onSelectKnownSfid }: Props) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<EligibleClearingBankCandidate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 输入防抖 300ms 后调 SFID
  useEffect(() => {
    if (query.trim().length === 0) {
      setResults([]);
      setError(null);
      return;
    }
    setLoading(true);
    setError(null);
    const timer = setTimeout(async () => {
      try {
        const r = await api.searchEligibleClearingBanks(query.trim(), 20);
        setResults(r);
      } catch (e) {
        setError(sanitizeError(e));
      } finally {
        setLoading(false);
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  const directSubmit = () => {
    const q = query.trim();
    if (!q) return;
    onSelectKnownSfid(q);
  };

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>添加清算行</h2>
      </div>
      <p className="muted">
        输入机构身份码(SFR/FFR 开头)或机构名关键字。SFID 资格白名单 =
        私法人股份公司(SFR-JOINT_STOCK)及其下属非法人(FFR-parent)。
      </p>
      <div className="form-group">
        <input
          autoFocus
          type="text"
          placeholder="机构身份码或名称关键字"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') directSubmit();
          }}
        />
        <button className="secondary-button" onClick={directSubmit} disabled={!query.trim()}>
          直接进入(按 sfid_id 查)
        </button>
      </div>

      {loading && <p>搜索中…</p>}
      {error && <div className="error">{error}</div>}

      {!loading && results.length > 0 && (
        <div className="admin-grid">
          {results.map((r) => (
            <div
              key={r.sfidId}
              className="metric-card admin-card"
              role="button"
              tabIndex={0}
              onClick={() => onSelectCandidate(r)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') onSelectCandidate(r);
              }}
            >
              <div>
                <strong>{r.institutionName}</strong>
                <span className={`status-badge status-${r.mainChainStatus.toLowerCase()}`} style={{ marginLeft: 8 }}>
                  {r.mainChainStatus}
                </span>
              </div>
              <code className="admin-card-address">{r.sfidId}</code>
              <span className="muted">
                {r.a3}{r.subType ? `-${r.subType}` : ''} · {r.province} · {r.city}
              </span>
              {r.parentSfidId && (
                <span className="muted">
                  所属:{r.parentInstitutionName ?? r.parentSfidId}
                </span>
              )}
            </div>
          ))}
        </div>
      )}

      {!loading && query.trim() && results.length === 0 && !error && (
        <p className="no-data">没有匹配的清算行候选。可能机构未在 SFID 注册,或不属于资格白名单。</p>
      )}
    </>
  );
}
