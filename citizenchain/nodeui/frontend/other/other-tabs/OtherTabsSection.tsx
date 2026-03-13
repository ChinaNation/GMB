import { useEffect, useState } from 'react';
import { api, sanitizeError } from '../../api';
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

  return (
    <section className="section other-tab-section" key={tab.key}>
      <h2>{tab.title}</h2>
      {tab.contentType === 'iframe' ? (
        <iframe
          className="other-tab-iframe"
          src={tab.url}
          title={tab.title}
          sandbox="allow-scripts allow-same-origin"
          referrerPolicy="no-referrer"
        />
      ) : (
        <p>{tab.text}</p>
      )}
    </section>
  );
}
