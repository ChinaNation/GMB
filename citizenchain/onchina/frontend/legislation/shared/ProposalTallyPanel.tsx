// 中文注释:提案进度纯展示件——六阶段步骤条 + 状态/表决类型/当前院 Tag + 院内/公投计票条。
// 只读投影(接收已取好的 LegProposalState,不含取数/鉴权),操作端进度页与大屏看板共用。

import React from 'react';
import { Progress, Space, Steps, Tag } from 'antd';
import type { LegProposalState } from '../types';
import { voteTypeLabel } from './labels';
import { STAGES, approvalPercent, statusTag } from './proposalStageUtils';

interface Props {
  state: LegProposalState;
}

export function ProposalTallyPanel({ state }: Props) {
  const stageIndex = STAGES.findIndex((s) => s.value === state.stage);
  const status = statusTag(state.status);

  return (
    <div>
      <Space wrap style={{ marginBottom: 12 }}>
        <Tag color={status.color}>{status.text}</Tag>
        <span>表决类型:{voteTypeLabel(state.voteType)}</span>
        <span>当前院:第 {state.currentHouse + 1} 院</span>
        {state.needsGuard && <Tag color="volcano">需护宪终审</Tag>}
        {state.referendumRequired && <Tag color="gold">需公民投票</Tag>}
      </Space>

      <Steps
        size="small"
        current={stageIndex < 0 ? 0 : stageIndex}
        items={STAGES.map((s) => ({ title: s.label }))}
      />

      <div style={{ marginTop: 16 }}>
        <div>
          院内计票(赞成 {state.houseTally.yes} / 反对 {state.houseTally.no}):
        </div>
        <Progress percent={approvalPercent(state.houseTally.yes, state.houseTally.no)} />
        {state.referendumRequired && (
          <>
            <div style={{ marginTop: 8 }}>
              公投计票(赞成 {state.referendumTally.yes} / 反对 {state.referendumTally.no}):
            </div>
            <Progress
              percent={approvalPercent(state.referendumTally.yes, state.referendumTally.no)}
            />
          </>
        )}
      </div>
    </div>
  );
}
