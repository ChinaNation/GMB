import { useCallback, useEffect, useRef, useState } from 'react';
import { sanitizeError } from '../../core/tauri';
import { adminsChangeApi as api } from './api';
import { AdminSetChangeSigningFlow } from './AdminSetChangeSigningFlow';
import { AdminSetDiff } from './AdminSetDiff';
import { AdminSetEditor } from './AdminSetEditor';
import { AdminWalletSelector } from './AdminWalletSelector';
import type { AdminAccountRef, AdminAccountState, VoteSignRequestResult } from './types';
import type { AdminWalletMatch } from '../../governance/types';
import './styles.css';

type Props = {
  accountRef: AdminAccountRef;
  cidFullName: string;
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

type Step = 'form' | 'sign';

export function AdminSetChangePage({
  accountRef,
  cidFullName,
  adminWallets,
  onBack,
  onSuccess,
}: Props) {
  const [account, setAccount] = useState<AdminAccountState | null>(null);
  const [admins, setAdmins] = useState<string[]>([]);
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null,
  );
  const [step, setStep] = useState<Step>('form');
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [signError, setSignError] = useState<string | null>(null);
  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [txHash, setTxHash] = useState<string | null>(null);
  const [signFlowVersion, setSignFlowVersion] = useState(0);
  const signRequestRef = useRef<VoteSignRequestResult | null>(null);
  const selectedWalletRef = useRef<AdminWalletMatch | null>(null);
  const adminsRef = useRef<string[]>([]);
  const accountRefRef = useRef<AdminAccountRef>(accountRef);

  signRequestRef.current = signRequest;
  selectedWalletRef.current = selectedWallet;
  adminsRef.current = admins;
  accountRefRef.current = accountRef;

  useEffect(() => {
    setLoading(true);
    api.getAdminAccountState(accountRef)
      .then((state) => {
        setAccount(state);
        setAdmins(state?.admins ?? []);
        setFormError(null);
      })
      .catch((e) => setFormError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [accountRef.cidNumber, accountRef.accountHex]);

  const buildRequest = async () => {
    if (!account || !selectedWallet) return;
    setSubmitting(true);
    setFormError(null);
    try {
      const result = await api.buildAdminSetChangeRequest(
        selectedWallet.pubkeyHex,
        accountRef,
        admins,
      );
      setSignRequest(result);
      setRequestJson(result.requestJson);
      setSignError(null);
      setTxHash(null);
      // 中文注释：每次重新生成签名请求都重置二维码有效期倒计时。
      setSignFlowVersion((value) => value + 1);
      setStep('sign');
    } catch (e) {
      setFormError(sanitizeError(e));
    } finally {
      setSubmitting(false);
    }
  };

  const submitSigned = useCallback(async (responseJson: string) => {
    const req = signRequestRef.current;
    const wallet = selectedWalletRef.current;
    if (!req || !wallet) {
      setSignError('签名请求数据丢失，请重新生成');
      return;
    }
    setSubmitting(true);
    setSignError(null);
    try {
      const result = await api.submitAdminSetChange(
        req.requestId,
        wallet.pubkeyHex,
        req.expectedPayloadHash,
        accountRefRef.current,
        adminsRef.current,
        req.signNonce,
        req.signBlockNumber,
        responseJson,
      );
      setTxHash(result.txHash);
    } catch (e) {
      setSignError(sanitizeError(e));
    } finally {
      setSubmitting(false);
    }
  }, []);

  if (loading) {
    return <div className="governance-section"><p>加载中…</p></div>;
  }

  if (!account) {
    return (
      <div className="governance-section">
        <button className="back-button" onClick={onBack}>← 返回</button>
        <div className="error">{formError || '未查询到管理员账户'}</div>
      </div>
    );
  }

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回机构详情</button>
      <h2>更换管理员</h2>
      <p className="proposal-institution-name">{cidFullName}</p>

      <div className="admin-set-change-summary">
        <div className="metric-card"><strong>{account.kindLabel}</strong><span>{account.statusLabel}</span></div>
        <div className="metric-card"><strong>{account.admins.length}</strong><span>当前管理员</span></div>
      </div>

      {step === 'form' && (
        <div className="create-proposal-form">
          {formError && <div className="error">{formError}</div>}
          <AdminWalletSelector
            wallets={adminWallets}
            value={selectedWallet}
            disabled={submitting}
            onChange={setSelectedWallet}
          />
          <AdminSetEditor admins={admins} disabled={submitting} onChange={setAdmins} />
          <AdminSetDiff currentAdmins={account.admins} admins={admins} />
          <button
            className="vote-signing-confirm"
            disabled={submitting || !selectedWallet || admins.length === 0}
            onClick={buildRequest}
          >
            {submitting ? '生成中…' : '生成签名请求'}
          </button>
        </div>
      )}

      {step === 'sign' && signRequest && (
        <AdminSetChangeSigningFlow
          key={signFlowVersion}
          request={signRequest}
          requestJson={requestJson}
          submitting={submitting}
          error={signError}
          txHash={txHash}
          onScan={submitSigned}
          onBackToForm={() => setStep('form')}
          onDone={onSuccess}
        />
      )}
    </div>
  );
}
