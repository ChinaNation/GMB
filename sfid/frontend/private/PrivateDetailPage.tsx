// 中文注释:私权机构详情页只调度私权布局,不得承载公安局/CPMS 业务。

import React, { useCallback, useEffect, useState } from 'react';
import { Typography } from 'antd';
import { getInstitution, type InstitutionDetail } from './common/api';
import { deleteAccount } from '../accounts/api';
import type { AdminAuth } from '../auth/types';
import { notice } from '../utils/notice';
import { PrivateDetailLayout } from './PrivateDetailLayout';
import {
  commitAdminAction,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
  type AdminSecurityGrantOutput,
} from '../admins/admin_security_api';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from '../core/CitizenSignatureModal';
import {
  institutionDetailCacheKey,
  readCachedInstitutionDetail,
  writeCachedInstitutionDetail,
} from '../china/metaCache';

interface Props {
  auth: AdminAuth;
  sfidNumber: string;
  canWrite: boolean;
  onBack: () => void;
}

type SecurityModalState = {
  actionId: string;
  signRequest: string;
  passkeyAssertion: unknown;
  resolve: (value: AdminSecurityGrantOutput) => void;
  reject: (reason?: unknown) => void;
};

export const PrivateDetailPage: React.FC<Props> = ({ auth, sfidNumber, canWrite, onBack }) => {
  const detailCacheKey = institutionDetailCacheKey(auth, sfidNumber);
  const [detail, setDetail] = useState<InstitutionDetail | null>(() =>
    readCachedInstitutionDetail(detailCacheKey),
  );
  const [loading, setLoading] = useState(false);
  const [securityCommitLoading, setSecurityCommitLoading] = useState(false);
  const [securityModal, setSecurityModal] = useState<SecurityModalState | null>(null);

  const load = useCallback(() => {
    const cached = readCachedInstitutionDetail(detailCacheKey);
    if (cached) {
      setDetail(cached);
      setLoading(false);
    } else {
      setDetail(null);
      setLoading(true);
    }
    getInstitution(auth, sfidNumber)
      .then((next) => {
        setDetail(next);
        writeCachedInstitutionDetail(detailCacheKey, next);
      })
      .catch(() => {
        // 中文注释:详情后台刷新失败时保留缓存,避免切页时闪断。
      })
      .finally(() => {
        if (!cached) setLoading(false);
      });
  }, [auth.access_token, detailCacheKey, sfidNumber]);

  useEffect(() => {
    load();
  }, [load]);

  const runPasskeyChallengeGrant = async (
    actionType: AdminActionType,
    payload: unknown,
  ): Promise<AdminSecurityGrantOutput> => {
    const prepared = await prepareAdminAction(auth, actionType, payload);
    if (prepared.auth_type !== 'PASSKEY_CHALLENGE' || !prepared.sign_request) {
      throw new Error('该操作缺少公民钱包签名请求');
    }
    const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
    return new Promise<AdminSecurityGrantOutput>((resolve, reject) => {
      setSecurityModal({
        actionId: prepared.action_id,
        signRequest: prepared.sign_request || '',
        passkeyAssertion,
        resolve,
        reject,
      });
    });
  };

  const handleSecuritySignedResponse = useCallback(async (raw: string) => {
    if (!securityModal) return;
    setSecurityCommitLoading(true);
    try {
      const signed = parseSignedReceiptPayload(raw, securityModal.actionId);
      if (signed.challenge_id !== securityModal.actionId) {
        throw new Error('签名回执与当前请求不匹配');
      }
      const grant = await commitAdminAction<AdminSecurityGrantOutput>(auth, {
        action_id: securityModal.actionId,
        passkey_assertion: securityModal.passkeyAssertion,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: signed.payload_hash,
      });
      securityModal.resolve(grant);
      setSecurityModal(null);
    } catch (err) {
      securityModal.reject(err);
      notice.error(err, '');
    } finally {
      setSecurityCommitLoading(false);
    }
  }, [auth, securityModal]);

  const inst = detail?.institution;

  const onDeleteAccount = async (accountName: string) => {
    try {
      const grant = await runPasskeyChallengeGrant('INSTITUTION_DELETE_ACCOUNT', {
        target: sfidNumber,
        sfid_number: sfidNumber,
        account_name: accountName,
      });
      await deleteAccount(auth, sfidNumber, accountName, grant);
      notice.success(`账户 "${accountName}" 已删除`);
      load();
    } catch (err) {
      notice.error(err, '');
    }
  };

  return (
    <div>
      {loading && !inst && <Typography.Text type="secondary">加载中...</Typography.Text>}

      {inst && detail && (
        <PrivateDetailLayout
          auth={auth}
          detail={detail}
          canWrite={canWrite}
          loading={loading}
          onReload={load}
          onDeleteAccount={onDeleteAccount}
          createPasskeyChallengeGrant={runPasskeyChallengeGrant}
          onBack={onBack}
        />
      )}

      <CitizenSignatureModal
        title="公民钱包签名确认"
        open={!!securityModal}
        onCancel={() => {
          securityModal?.reject(new Error('已取消签名确认'));
          setSecurityModal(null);
          setSecurityCommitLoading(false);
        }}
        qrTitle="签名二维码"
        qrValue={securityModal?.signRequest}
        qrHint="使用联邦管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名回执二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => notice.error(msg)}
      />
    </div>
  );
};
