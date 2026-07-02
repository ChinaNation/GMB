// 院内表决(一人一票)。输入提案 ID + 赞成/反对 → api.castHouseVote → sign_request → 扫码上链弹窗。
// 只有能参与院内表决的机构(发起院/复议院)可见(canCastHouseVote 门控,后端 role 二次校验)。

import React, { useState } from 'react';
import { Button, InputNumber, Space, message } from 'antd';
import type { AdminAuth } from '../../../auth/types';
import { castHouseVote } from '../../api';
import { SignRequestModal } from './SignRequestModal';

interface Props {
  auth: AdminAuth;
}

export function HouseVotePanel({ auth }: Props) {
  const [proposalId, setProposalId] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [signRequest, setSignRequest] = useState<string | null>(null);

  const vote = async (approve: boolean) => {
    if (proposalId === null) {
      return;
    }
    setSubmitting(true);
    try {
      const request = await castHouseVote(auth, proposalId, approve);
      setSignRequest(request);
    } catch (e: unknown) {
      message.error(e instanceof Error ? e.message : '表决提交失败');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div>
      <Space wrap>
        <span>提案 ID:</span>
        <InputNumber
          min={0}
          value={proposalId ?? undefined}
          onChange={(v) => setProposalId(v ?? null)}
        />
        <Button
          type="primary"
          loading={submitting}
          disabled={proposalId === null}
          onClick={() => vote(true)}
        >
          赞成
        </Button>
        <Button
          danger
          loading={submitting}
          disabled={proposalId === null}
          onClick={() => vote(false)}
        >
          反对
        </Button>
      </Space>
      <SignRequestModal
        signRequest={signRequest}
        onClose={() => setSignRequest(null)}
        title="扫码签署表决并提交上链"
      />
    </div>
  );
}
