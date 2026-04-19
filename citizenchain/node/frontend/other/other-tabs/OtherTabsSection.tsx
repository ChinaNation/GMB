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

  // 白皮书/公民宪法页不再在 UI 内重复显示标题文字(顶部 tab 栏已经表明当前页),
  // 让 iframe/文本内容直接铺满可用区域。tab.title 仍传给 iframe 的 title 属性用于无障碍。
  return (
    <section className="section other-tab-section" key={tab.key}>
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
