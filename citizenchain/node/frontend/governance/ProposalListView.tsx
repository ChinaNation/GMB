// 全部提案分页列表：按 ID 倒序，展示提案类型、状态和简要描述。
import { useEffect, useState, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import type { ProposalPageResult, ProposalListItem } from './governance-types';

type Props = {
  onSelect: (proposalId: number) => void;
};

const PAGE_SIZE = 10;

export function ProposalListView({ onSelect }: Props) {
  const [items, setItems] = useState<ProposalListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const [nextStartId, setNextStartId] = useState<number | null>(null);

  // 首次加载
  useEffect(() => {
    setLoading(true);
    api.getNextProposalId()
      .then((nextId) => {
        if (nextId === 0) {
          setItems([]);
          setHasMore(false);
          setLoading(false);
          return;
        }
        return api.getProposalPage(nextId - 1, PAGE_SIZE).then((result) => {
          setItems(result.items);
          setHasMore(result.hasMore);
          setWarning(result.warning);
          if (result.items.length > 0) {
            const lastId = result.items[result.items.length - 1].proposalId;
            setNextStartId(lastId > 0 ? lastId - 1 : null);
          }
        });
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, []);

  // 加载更多
  const loadMore = useCallback(() => {
    if (loadingMore || nextStartId == null || !hasMore) return;
    setLoadingMore(true);
    api.getProposalPage(nextStartId, PAGE_SIZE)
      .then((result) => {
        setItems((prev) => [...prev, ...result.items]);
        setHasMore(result.hasMore);
        if (result.items.length > 0) {
          const lastId = result.items[result.items.length - 1].proposalId;
          setNextStartId(lastId > 0 ? lastId - 1 : null);
        } else {
          setHasMore(false);
        }
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoadingMore(false));
  }, [loadingMore, nextStartId, hasMore]);

  if (loading) {
    return <div className="governance-section"><p>加载提案列表…</p></div>;
  }

  if (error) {
    return (
      <div className="governance-section">
        <div className="error">{error}</div>
      </div>
    );
  }

  return (
    <div className="governance-section">
      <h2>全部提案</h2>
      {warning && <div className="warning">{warning}</div>}

      {items.length === 0 ? (
        <p className="no-data">暂无提案（需节点运行并同步完成后查询）</p>
      ) : (
        <div className="proposal-list">
          {items.map((item) => (
            <div
              key={item.proposalId}
              className="proposal-card"
              onClick={() => onSelect(item.proposalId)}
            >
              <div className="proposal-card-header">
                <span className="proposal-id">{item.displayId}</span>
                <span className={`proposal-status status-${item.status}`}>
                  {item.statusLabel}
                </span>
              </div>
              <div className="proposal-card-body">
                <div className="proposal-card-tags">
                  <span className="proposal-tag">{item.kindLabel}</span>
                  {item.kind === 1 && (
                    <span className="proposal-tag">{item.stageLabel}</span>
                  )}
                  {item.institutionName && (
                    <span className="proposal-tag institution-tag">
                      {item.institutionName}
                    </span>
                  )}
                </div>
                <div className="proposal-summary">{item.summary}</div>
              </div>
            </div>
          ))}
        </div>
      )}

      {hasMore && (
        <button
          className="load-more-button"
          onClick={loadMore}
          disabled={loadingMore}
        >
          {loadingMore ? '加载中…' : '加载更多'}
        </button>
      )}
    </div>
  );
}
