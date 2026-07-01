// 中文注释:法律详情(只读)。章>节>条>款 编辑体渲染;宪法双语可切换。
// **零发起入口**——阅读页绝不放发起/修改按钮(feedback_entry_placement_read_vs_action)。

import React, { useEffect, useState } from 'react';
import { Alert, Button, Spin, Switch } from 'antd';
import type { AdminAuth } from '../../../auth/types';
import { getLaw } from '../../api';
import type { LawChapter, LawView } from '../../types';
import { statusTag, tierLabel, voteTypeLabel } from './labels';

interface Props {
  auth: AdminAuth;
  lawId: number;
  onBack: () => void;
}

/** 双语取值:开启英文且有英文则显示英文,否则中文。 */
function text(zh: string, en: string | null | undefined, showEn: boolean): string {
  return showEn && en ? en : zh;
}

function formatTimestamp(ms: number): string {
  if (!ms) {
    return '立即生效';
  }
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(ms));
}

function versionText(law: LawView): string {
  const tags = [`展示 v${law.version}`];
  if (law.effectiveVersion) {
    tags.push(`生效 v${law.effectiveVersion}`);
  }
  if (law.pendingVersion) {
    tags.push(`待生效 v${law.pendingVersion}`);
  }
  return tags.join(' · ');
}

/** 章>节>条>款 编辑体渲染。 */
function renderChapter(chapter: LawChapter, showEn: boolean) {
  return (
    <section key={chapter.number} style={{ marginBottom: 24 }}>
      <h2 style={{ fontSize: 18, fontWeight: 700, margin: '16px 0 8px' }}>
        第{chapter.number}章 {text(chapter.title, chapter.titleEn, showEn)}
      </h2>
      {chapter.sections.map((section) => (
        <div key={section.number} style={{ marginLeft: 12 }}>
          <h3 style={{ fontSize: 15, fontWeight: 600, margin: '10px 0 6px' }}>
            第{section.number}节 {text(section.title, section.titleEn, showEn)}
          </h3>
          {section.articles.map((article) => (
            <div key={article.number} style={{ marginBottom: 12 }}>
              <div style={{ fontWeight: 600 }}>
                第{article.number}条 {text(article.title, article.titleEn, showEn)}
              </div>
              {article.body && (
                <p style={{ margin: '4px 0', lineHeight: 1.7 }}>
                  {text(article.body, article.bodyEn, showEn)}
                </p>
              )}
              {article.clauses.map((clause) => (
                <p
                  key={clause.number}
                  style={{ marginLeft: 16, lineHeight: 1.7, color: 'rgba(0,0,0,0.75)' }}
                >
                  第{clause.number}款 {text(clause.text, clause.textEn, showEn)}
                </p>
              ))}
            </div>
          ))}
        </div>
      ))}
    </section>
  );
}

/** 法律详情只读视图。 */
export function LawDetailView({ auth, lawId, onBack }: Props) {
  const [law, setLaw] = useState<LawView | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showEn, setShowEn] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    getLaw(auth, lawId)
      .then((data) => {
        if (!cancelled) {
          setLaw(data);
          setLoading(false);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : '加载法律详情失败');
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [auth.access_token, lawId]);

  if (loading) {
    return <Spin />;
  }
  if (error) {
    return (
      <div>
        <Button type="link" style={{ paddingLeft: 0 }} onClick={onBack}>
          ← 返回法律列表
        </Button>
        <Alert type="error" message={error} showIcon />
      </div>
    );
  }
  if (!law) {
    return null;
  }

  const bilingual = !!law.titleEn;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Button type="link" style={{ paddingLeft: 0 }} onClick={onBack}>
          ← 返回法律列表
        </Button>
        {bilingual && (
          <span style={{ fontSize: 13, color: 'rgba(0,0,0,0.55)' }}>
            中文 <Switch size="small" checked={showEn} onChange={setShowEn} /> English
          </span>
        )}
      </div>

      <header style={{ marginBottom: 16 }}>
        <h1 style={{ fontSize: 24, fontWeight: 700, margin: '4px 0' }}>{law.title}</h1>
        {law.titleEn && (
          <div style={{ color: 'rgba(0,0,0,0.55)', fontSize: 14 }}>{law.titleEn}</div>
        )}
        <div style={{ marginTop: 8, display: 'flex', gap: 12, alignItems: 'center', color: 'rgba(0,0,0,0.65)' }}>
          <span>{tierLabel(law.tier)}</span>
          <span>{voteTypeLabel(law.voteType)}</span>
          {statusTag(law.status)}
          <span>{versionText(law)}</span>
          <span>生效时间 {formatTimestamp(law.effectiveAt)}</span>
        </div>
      </header>

      <article>{law.chapters.map((chapter) => renderChapter(chapter, showEn))}</article>
    </div>
  );
}
