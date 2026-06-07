// 中文注释:sfid 元信息(subject_property / 机构类型 / 省份 / scoped_province)加载 hook。
// 步 0 目标:提供一个统一的加载点,机构详情页、注册弹窗、三机构视图等都用这一份。
// 设计:
//   - auth 为 null 时不发请求,直接返回 { meta: null, loading: false }
//   - 按需调用 reload() 手动刷新;确定性元数据优先命中本地缓存
//   - 失败时把错误写到 error,不抛到组件外

import { useCallback, useEffect, useState } from 'react';
import type { AdminAuth } from '../auth/types';
import type { SfidMetaResult } from '../china/api';
import { loadCachedSfidMeta } from '../china/metaCache';

export interface UseSfidMetaResult {
  meta: SfidMetaResult | null;
  loading: boolean;
  error: string | null;
  reload: () => Promise<void>;
}

export function useSfidMeta(auth: AdminAuth | null): UseSfidMetaResult {
  const [meta, setMeta] = useState<SfidMetaResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (!auth) {
      setMeta(null);
      setError(null);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const next = await loadCachedSfidMeta(auth);
      setMeta(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [auth]);

  useEffect(() => {
    let cancelled = false;
    if (!auth) {
      setMeta(null);
      setError(null);
      return () => {
        cancelled = true;
      };
    }
    setLoading(true);
    setError(null);
    loadCachedSfidMeta(auth)
      .then((next) => {
        if (!cancelled) setMeta(next);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [auth]);

  return { meta, loading, error, reload };
}
