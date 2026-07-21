// 投票签名流程：选钱包 → 显示 QR → 摄像头扫描响应 → 提交。
import { useState, useEffect, useRef, useCallback } from 'react';
import { sanitizeError } from '../tauri';
import { CitizenSignaturePanel } from '../shared/qr/CitizenSignaturePanel';
import { governanceApi as api } from './api';
import type { AdminWalletMatch, VoteSignRequestResult } from './types';

type Props = {
  proposalId: number;
  proposalKind: number;
  adminWallets: AdminWalletMatch[];
  cidNumber?: string;
  onClose: () => void;
  onSuccess: (txHash: string, pubkeyHex: string, voterRoleCode: string | null) => void;
};

type FlowStep = 'select' | 'qr' | 'submit' | 'done' | 'error';

export function VoteSigningFlow({
  proposalId, proposalKind, adminWallets, cidNumber, onClose, onSuccess,
}: Props) {
  const [step, setStep] = useState<FlowStep>('select');
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null
  );
  const [approve, setApprove] = useState<boolean | null>(null);
  const [selectedRoleCode, setSelectedRoleCode] = useState<string>(
    adminWallets.length === 1 && adminWallets[0].roleAssignments?.length === 1
      ? adminWallets[0].roleAssignments![0].roleCode
      : '',
  );
  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [callDataHex, setCallDataHex] = useState('');
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const signRequestRef = useRef(signRequest);
  const selectedWalletRef = useRef(selectedWallet);
  const callDataHexRef = useRef(callDataHex);
  signRequestRef.current = signRequest;
  selectedWalletRef.current = selectedWallet;
  callDataHexRef.current = callDataHex;

  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) { setError('签名请求已过期，请重新操作'); setStep('error'); return; }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  const generateRequest = useCallback(async () => {
    if (!selectedWallet || approve === null) return;
    const institutionVote = !!cidNumber;
    if (institutionVote && !selectedRoleCode) {
      setError('请选择本次投票使用的岗位');
      return;
    }
    try {
      let result: VoteSignRequestResult;
      let cdHex: string;
      // 内部投票(管理员一人一票)统一走 InternalVote::cast(20.0),
      // 联合投票走 JointVote::cast_admin(21.0),由 proposalKind===1 分支决定。
      if (proposalKind === 1 && cidNumber) {
        result = await api.buildJointVoteRequest(
          proposalId,
          selectedWallet.pubkeyHex,
          cidNumber,
          selectedRoleCode,
          approve,
        );
        cdHex = result.callDataHex;
      } else {
        result = await api.buildVoteRequest(
          proposalId,
          selectedWallet.pubkeyHex,
          institutionVote ? selectedRoleCode : null,
          approve,
        );
        cdHex = result.callDataHex;
      }
      setSignRequest(result);
      setCallDataHex(cdHex);
      setRequestJson(result.requestJson);
      setCountdown(90);
      setStep('qr');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [proposalId, proposalKind, selectedWallet, selectedRoleCode, approve, cidNumber]);

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const wallet = selectedWalletRef.current;
    const cdHex = callDataHexRef.current;
    if (!req || !wallet) { setError('签名请求数据丢失，请重试'); setStep('error'); return; }
    setStep('submit');
    try {
      const result = await api.submitVote(req.requestId, wallet.pubkeyHex, req.expectedPayloadHash, cdHex, req.signNonce, req.signBlockNumber, responseText);
      setTxHash(result.txHash);
      setStep('done');
      onSuccess(
        result.txHash,
        selectedWallet?.pubkeyHex ?? '',
        cidNumber ? selectedRoleCode : null,
      );
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [onSuccess, selectedWallet, selectedRoleCode, cidNumber]);

  return (
    <div className="vote-signing-overlay">
      <div className={`vote-signing-modal ${step === 'qr' ? 'signature-flow-modal' : ''}`}>
        <div className="vote-signing-header">
          <h3>{proposalKind === 1 ? '联合投票' : '投票'}</h3>
          <button className="vote-signing-close" onClick={onClose}>✕</button>
        </div>

        {step === 'select' && (
          <div className="vote-signing-body">
            {adminWallets.length > 1 && (
              <div className="vote-signing-field">
                <label>选择管理员钱包</label>
                <select value={selectedWallet?.pubkeyHex || ''} onChange={(e) => {
                  const wallet = adminWallets.find((w) => w.pubkeyHex === e.target.value) || null;
                  setSelectedWallet(wallet);
                  setSelectedRoleCode(wallet?.roleAssignments?.length === 1 ? wallet.roleAssignments[0].roleCode : '');
                }}>
                  <option value="">请选择…</option>
                  {adminWallets.map((w) => <option key={w.pubkeyHex} value={w.pubkeyHex}>{w.walletLabel || w.address}</option>)}
                </select>
              </div>
            )}
            {cidNumber && selectedWallet && (
              <div className="vote-signing-field">
                <label>使用岗位</label>
                <select value={selectedRoleCode} onChange={(e) => setSelectedRoleCode(e.target.value)}>
                  <option value="">请选择…</option>
                  {(selectedWallet.roleAssignments ?? []).map((assignment) => (
                    <option key={assignment.roleCode} value={assignment.roleCode}>
                      {assignment.roleName || assignment.roleCode}（{assignment.roleCode}）
                    </option>
                  ))}
                </select>
              </div>
            )}
            <div className="vote-signing-field">
              <label>投票方向</label>
              <div className="vote-direction-buttons">
                <button className={`vote-dir-btn ${approve === true ? 'selected-yes' : ''}`} onClick={() => setApprove(true)}>赞成</button>
                <button className={`vote-dir-btn ${approve === false ? 'selected-no' : ''}`} onClick={() => setApprove(false)}>反对</button>
              </div>
            </div>
            <button className="vote-signing-confirm" disabled={!selectedWallet || approve === null || (!!cidNumber && !selectedRoleCode)} onClick={generateRequest}>生成签名请求</button>
          </div>
        )}

        {step === 'qr' && (
          <div className="vote-signing-body">
            <CitizenSignaturePanel
              qrValue={requestJson}
              countdownSeconds={countdown}
              onScan={handleScanResult}
              onScanError={(e) => { setError(e); setStep('error'); }}
            />
          </div>
        )}

        {step === 'submit' && <div className="vote-signing-body"><p className="qr-instruction">正在验证签名并提交到链…</p></div>}

        {step === 'done' && (
          <div className="vote-signing-body">
            <div className="vote-success"><p>投票已提交</p>{txHash && <code className="tx-hash">交易哈希: {txHash}</code>}</div>
            <button className="vote-signing-confirm" onClick={onClose}>完成</button>
          </div>
        )}

        {step === 'error' && (
          <div className="vote-signing-body">
            <div className="error">{error}</div>
            <button className="vote-signing-confirm" onClick={() => { setError(null); setStep('select'); }}>重试</button>
          </div>
        )}
      </div>
    </div>
  );
}
