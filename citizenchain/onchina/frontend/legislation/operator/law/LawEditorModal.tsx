// 中文注释:法律案结构化编辑器(章>节>条>款 + 元字段)。立法/修法带正文,废法只填 law_id。
// 2D-1:组装 ProposeLawInput 并预览(校验编辑器产出正确);冷签提交(sign_request→QR)在 2D-2 接入。
// 嵌套不可变更新用 structuredClone(草稿改后整体替换),保持 React 引用变更即渲染。

import React, { useState } from 'react';
import { Button, Divider, Input, InputNumber, Modal, Space, Typography, message } from 'antd';
import type { AdminAuth } from '../../../auth/types';
import { proposeLegislation } from '../../api';
import { SignRequestModal } from './SignRequestModal';
import type {
  LawActionInput,
  LawArticle,
  LawChapter,
  LawClause,
  LawSection,
  ProposeLawInput,
} from '../../types';
import { voteTypeLabel } from './labels';

interface Props {
  open: boolean;
  auth: AdminAuth;
  lawAction: LawActionInput;
  tier: number;
  voteType: number;
  onClose: () => void;
}

const ACTION_LABEL: Record<LawActionInput, string> = {
  enact: '立法',
  amend: '修法',
  repeal: '废法',
};

const emptyClause = (n: number): LawClause => ({ number: n, text: '', textEn: null });
const emptyArticle = (n: number): LawArticle => ({
  number: n,
  title: '',
  titleEn: null,
  body: '',
  bodyEn: null,
  clauses: [],
});
const emptySection = (n: number): LawSection => ({ number: n, title: '', titleEn: null, articles: [] });
const emptyChapter = (n: number): LawChapter => ({ number: n, title: '', titleEn: null, sections: [] });

const toLocalDateTimeValue = (ms: number): string => {
  const date = new Date(ms);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
};

const fromLocalDateTimeValue = (value: string): number => {
  const ms = new Date(value).getTime();
  return Number.isFinite(ms) ? ms : 0;
};

const blockStyle = (indent: number): React.CSSProperties => ({
  marginLeft: indent,
  marginTop: 8,
  paddingLeft: 12,
  borderLeft: '2px solid #e5e7eb',
});

export function LawEditorModal({ open, auth, lawAction, tier, voteType, onClose }: Props) {
  const [title, setTitle] = useState('');
  const [titleEn, setTitleEn] = useState('');
  const [effectiveAt, setEffectiveAt] = useState<number>(() => Date.now());
  const [lawId, setLawId] = useState<number | null>(null);
  const [chapters, setChapters] = useState<LawChapter[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [signRequest, setSignRequest] = useState<string | null>(null);

  const needsLawId = lawAction === 'amend' || lawAction === 'repeal';
  const needsChapters = lawAction === 'enact' || lawAction === 'amend';

  /** 嵌套章节的不可变更新:克隆草稿 → 就地改 → 整体替换。 */
  const mutate = (fn: (draft: LawChapter[]) => void) =>
    setChapters((prev) => {
      const draft = structuredClone(prev) as LawChapter[];
      fn(draft);
      return draft;
    });

  const buildInput = (): ProposeLawInput => ({
    lawAction,
    tier,
    scopeCode: 0, // 后端会话派生本节点 scope(2D-2 起后端覆盖),前端占位
    voteType,
    title,
    titleEn: titleEn || null,
    chapters: needsChapters ? chapters : [],
    effectiveAt,
    lawId: needsLawId ? lawId : null,
  });

  const submit = async () => {
    setSubmitting(true);
    try {
      const request = await proposeLegislation(auth, buildInput());
      setSignRequest(request);
    } catch (e: unknown) {
      message.error(e instanceof Error ? e.message : '发起提案失败');
    } finally {
      setSubmitting(false);
    }
  };

  const closeSign = () => {
    setSignRequest(null);
    onClose();
  };

  return (
    <>
    <Modal
      open={open}
      title={`${ACTION_LABEL[lawAction]}(${voteTypeLabel(voteType)})`}
      onCancel={onClose}
      onOk={submit}
      okText="发起并生成签名码"
      confirmLoading={submitting}
      width={860}
      destroyOnClose
    >
      <Space direction="vertical" style={{ width: '100%' }} size="small">
        <Input addonBefore="法律标题" value={title} onChange={(e) => setTitle(e.target.value)} />
        <Input
          addonBefore="英文标题"
          value={titleEn}
          onChange={(e) => setTitleEn(e.target.value)}
          placeholder="宪法必填;普通法可空"
        />
        {needsLawId && (
          <Space>
            <span>目标法律 ID:</span>
            <InputNumber min={0} value={lawId ?? undefined} onChange={(v) => setLawId(v ?? null)} />
          </Space>
        )}
        {needsChapters && (
          <Space>
            <span>生效时间:</span>
            <Input
              type="datetime-local"
              value={toLocalDateTimeValue(effectiveAt)}
              onChange={(e) => setEffectiveAt(fromLocalDateTimeValue(e.target.value))}
              style={{ width: 220 }}
            />
          </Space>
        )}
      </Space>

      {needsChapters && (
        <>
          <Divider orientation="left" style={{ margin: '12px 0' }}>
            正文(章 &gt; 节 &gt; 条 &gt; 款)
          </Divider>
          {chapters.map((chapter, ci) => (
            <div key={ci} style={blockStyle(0)}>
              <Space wrap style={{ width: '100%' }}>
                <Typography.Text strong>第{chapter.number}章</Typography.Text>
                <Input
                  style={{ width: 200 }}
                  placeholder="章名"
                  value={chapter.title}
                  onChange={(e) => mutate((d) => { d[ci].title = e.target.value; })}
                />
                <Button size="small" onClick={() => mutate((d) => d[ci].sections.push(emptySection(d[ci].sections.length + 1)))}>
                  + 节
                </Button>
                <Button size="small" danger onClick={() => mutate((d) => { d.splice(ci, 1); })}>
                  删章
                </Button>
              </Space>

              {chapter.sections.map((section, si) => (
                <div key={si} style={blockStyle(12)}>
                  <Space wrap>
                    <Typography.Text>第{section.number}节</Typography.Text>
                    <Input
                      style={{ width: 180 }}
                      placeholder="节名"
                      value={section.title}
                      onChange={(e) => mutate((d) => { d[ci].sections[si].title = e.target.value; })}
                    />
                    <Button size="small" onClick={() => mutate((d) => d[ci].sections[si].articles.push(emptyArticle(d[ci].sections[si].articles.length + 1)))}>
                      + 条
                    </Button>
                    <Button size="small" danger onClick={() => mutate((d) => { d[ci].sections.splice(si, 1); })}>
                      删节
                    </Button>
                  </Space>

                  {section.articles.map((article, ai) => (
                    <div key={ai} style={blockStyle(12)}>
                      <Space wrap>
                        <Typography.Text>第{article.number}条</Typography.Text>
                        <Input
                          style={{ width: 160 }}
                          placeholder="条标题"
                          value={article.title}
                          onChange={(e) => mutate((d) => { d[ci].sections[si].articles[ai].title = e.target.value; })}
                        />
                        <Button size="small" onClick={() => mutate((d) => d[ci].sections[si].articles[ai].clauses.push(emptyClause(d[ci].sections[si].articles[ai].clauses.length + 1)))}>
                          + 款
                        </Button>
                        <Button size="small" danger onClick={() => mutate((d) => { d[ci].sections[si].articles.splice(ai, 1); })}>
                          删条
                        </Button>
                      </Space>
                      <Input.TextArea
                        style={{ marginTop: 4 }}
                        rows={2}
                        placeholder="条正文"
                        value={article.body}
                        onChange={(e) => mutate((d) => { d[ci].sections[si].articles[ai].body = e.target.value; })}
                      />
                      {article.clauses.map((clause, li) => (
                        <div key={li} style={blockStyle(12)}>
                          <Space style={{ width: '100%' }}>
                            <Typography.Text type="secondary">第{clause.number}款</Typography.Text>
                            <Input
                              style={{ width: 360 }}
                              placeholder="款正文"
                              value={clause.text}
                              onChange={(e) => mutate((d) => { d[ci].sections[si].articles[ai].clauses[li].text = e.target.value; })}
                            />
                            <Button size="small" danger onClick={() => mutate((d) => { d[ci].sections[si].articles[ai].clauses.splice(li, 1); })}>
                              删
                            </Button>
                          </Space>
                        </div>
                      ))}
                    </div>
                  ))}
                </div>
              ))}
            </div>
          ))}
          <Button style={{ marginTop: 8 }} onClick={() => mutate((d) => d.push(emptyChapter(d.length + 1)))}>
            + 添加章
          </Button>
        </>
      )}
    </Modal>
      <SignRequestModal signRequest={signRequest} onClose={closeSign} />
    </>
  );
}
