import { useEffect, useState } from 'react';
import { api } from '../../api';
import type { OtherTabsPayload } from '../../types';

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
        setError(e instanceof Error ? e.message : String(e));
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

  return (
    <section className="section whitepaper-section" key={tab.key}>
      <h2>{tab.title}</h2>
      {tab.contentType === 'iframe' && tab.url ? (
        <iframe
          className="whitepaper-iframe"
          src={tab.url}
          title={tab.title}
        />
      ) : (
        <p>{tab.text ?? '暂无内容'}</p>
      )}
    </section>
  );
}
