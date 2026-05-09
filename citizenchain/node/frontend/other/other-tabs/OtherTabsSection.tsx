import { useEffect, useState } from 'react';
import { sanitizeError } from '../../core/tauri';
import { LOCAL_DOCS } from '../../generated/local-docs.generated';
import { otherTabsApi as api } from './api';
import { LocalDocViewer } from './LocalDocViewer';
import type { OtherTabsPayload } from './types';

type Props = {
  activeKey: 'whitepaper' | 'party' | 'constitution';
};

export function OtherTabsSection({ activeKey }: Props) {
  const [payload, setPayload] = useState<OtherTabsPayload | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void api
      .getOtherTabsContent()
      .then((data) => {
        if (cancelled) return;
        setPayload(data);
      })
      .catch((e) => {
        if (cancelled) return;
        setError(sanitizeError(e));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  if (error) {
    return <pre className="error">{error}</pre>;
  }

  if (!payload) {
    return (
      <section className="section">
        <p>加载中...</p>
      </section>
    );
  }

  const tab = payload.tabs.find((item) => item.key === activeKey);
  if (!tab) {
    return (
      <section className="section">
        <p>暂无内容</p>
      </section>
    );
  }

  // 中文注释：本地文档以当前 tab 为绑定源，避免字段缺失时误回退到白皮书。
  const localDoc =
    tab.contentType === 'document'
      ? LOCAL_DOCS.find((doc) => doc.key === activeKey)
      : null;

  return (
    <section className="section other-tab-section" key={tab.key}>
      {tab.contentType === 'document' ? (
        localDoc ? (
          <LocalDocViewer doc={localDoc} />
        ) : (
          <pre className="error">文档配置错误：{activeKey}</pre>
        )
      ) : (
        <p>{tab.text}</p>
      )}
    </section>
  );
}
