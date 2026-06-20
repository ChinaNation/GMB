// "添加清算行"页:输入 sfid_number / 关键字 → debounce 自动搜 SFID 候选 → 选中即进入下一步。
//
// 2026-05-01 重构:删除"查询"按钮(input debounce 已自动搜),回车也直接选第一个候选,
// 不再走 onSelectKnownSfid 走 sfidNumber 字符串透传(链上判定改由 check-multisig 视图统一处理)。

import { useEffect, useState } from 'react';
import { sanitizeError } from '../../core/tauri';
import { organizationManageApi } from './api';
import type { EligibleClearingBankCandidate } from './types';

type Props = {
  onBack: () => void;
  onSelectCandidate: (c: EligibleClearingBankCandidate) => void;
  /** 已知 sfid_number 直接进入下一步(empty 视图列表 → 直接 check-multisig)。 */
  onSelectKnownSfid: (sfidNumber: string) => void;
};

export function ClearingBankAddPage({ onBack, onSelectCandidate, onSelectKnownSfid: _onSelectKnownSfid }: Props) {
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
        const r = await organizationManageApi.searchEligibleClearingBanks(query.trim(), 20);
        setResults(r);
      } catch (e) {
        setError(sanitizeError(e));
      } finally {
        setLoading(false);
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>添加清算行</h2>
      </div>
      <div className="form-group">
        <input
          autoFocus
          type="text"
          placeholder="机构身份号码或名称关键字(自动搜索)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            // 回车选中第一个候选(无候选时静默)
            if (e.key === 'Enter' && results.length > 0) onSelectCandidate(results[0]);
          }}
        />
      </div>

      {loading && <p>搜索中…</p>}
      {error && <div className="error">{error}</div>}

      {!loading && results.length > 0 && (
        <div className="admin-grid">
          {results.map((r) => (
            <div
              key={r.sfidNumber}
              className="metric-card admin-card"
              role="button"
              tabIndex={0}
              onClick={() => onSelectCandidate(r)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') onSelectCandidate(r);
              }}
            >
              <strong>{r.sfidFullName}</strong>
              <code className="admin-card-address">{r.sfidNumber}</code>
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
