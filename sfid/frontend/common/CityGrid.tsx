// 中文注释:某省的 N 市卡片网格 — 共享组件。
// 按省拉取城市清单(走现有 listSfidCities API),点击某市回调外部。
// 过滤掉 code="000" 的"本省统一"占位。

import React, { useEffect, useState } from 'react';
import type { AdminAuth } from '../auth/types';
import { listSfidCities, type SfidCityItem } from '../sfid/api';

interface Props {
  auth: AdminAuth;
  province: string;
  onPick: (city: string) => void;
}

const CARD_STYLE: React.CSSProperties = {
  padding: 14,
  borderRadius: 10,
  border: '1px solid rgba(15,23,42,0.22)',
  background: 'rgba(226,232,240,0.55)',
  boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
  cursor: 'pointer',
  transition: 'all 0.2s ease',
  textAlign: 'center' as const,
};

export const CityGrid: React.FC<Props> = ({ auth, province, onPick }) => {
  const [cities, setCities] = useState<SfidCityItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    listSfidCities(auth, province)
      .then((rows) => {
        if (cancelled) return;
        setCities(rows.filter((c) => c.code !== '000'));
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : '加载城市列表失败');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [auth.access_token, province]);

  return (
    <div>
      {loading && <div style={{ color: 'rgba(15,23,42,0.6)' }}>加载中...</div>}
      {error && <div style={{ color: '#b00020' }}>{error}</div>}
      {!loading && !error && cities.length === 0 && (
        <div style={{ color: 'rgba(15,23,42,0.6)' }}>该省暂无城市数据</div>
      )}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
          gap: 10,
        }}
      >
        {cities.map((c) => (
          <div
            key={c.code}
            onClick={() => onPick(c.name)}
            style={CARD_STYLE}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLDivElement).style.background = 'rgba(13,148,136,0.22)';
              (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(13,148,136,0.55)';
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLDivElement).style.background = 'rgba(226,232,240,0.55)';
              (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(15,23,42,0.22)';
            }}
          >
            <div style={{ fontSize: 15, fontWeight: 600, color: '#0f172a' }}>{c.name}</div>
          </div>
        ))}
      </div>
    </div>
  );
};
