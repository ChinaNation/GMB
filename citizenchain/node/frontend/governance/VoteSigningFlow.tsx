// 投票签名流程：选钱包 → 显示 QR → 摄像头扫描响应 → 提交。
import { useState, useEffect, useRef, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../core/tauri';
import { QrScanner } from '../shared/qr/QrScanner';
import { governanceApi as api } from './api';
import type { AdminWalletMatch, VoteSignRequestResult } from './types';

type Props = {
  proposalId: number;
  proposalKind: number;
  adminWallets: AdminWalletMatch[];
  sfidNumber?: string;
  onClose: () => void;
  onSuccess: (txHash: string) => void;
};

type FlowStep = 'select' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function VoteSigningFlow({
  proposalId, proposalKind, adminWallets, sfidNumber, onClose, onSuccess,
}: Props) {
  const [step, setStep] = useState<FlowStep>('select');
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null
  );
  const [approve, setApprove] = useState<boolean | null>(null);
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
    try {
      let result: VoteSignRequestResult;
      let cdHex: string;
      // 内部投票(管理员一人一票)统一走 InternalVote::cast(22.0),
      // 联合投票走 JointVote::cast_admin(23.0),由 proposalKind===1 分支决定。
      if (proposalKind === 1 && sfidNumber) {
        result = await api.buildJointVoteRequest(proposalId, selectedWallet.pubkeyHex, sfidNumber, approve);
        cdHex = result.callDataHex;
      } else {
        result = await api.buildVoteRequest(proposalId, selectedWallet.pubkeyHex, approve);
        cdHex = result.callDataHex || buildInternalVoteCallDataHex(proposalId, approve);
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
  }, [proposalId, proposalKind, selectedWallet, approve, sfidNumber]);

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
      onSuccess(result.txHash);
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [onSuccess]);

  return (
    <div className="vote-signing-overlay">
      <div className="vote-signing-modal">
        <div className="vote-signing-header">
          <h3>{proposalKind === 1 ? '联合投票签名' : '投票签名'}</h3>
          <button className="vote-signing-close" onClick={onClose}>✕</button>
        </div>

        {step === 'select' && (
          <div className="vote-signing-body">
            {adminWallets.length > 1 && (
              <div className="vote-signing-field">
                <label>选择管理员钱包</label>
                <select value={selectedWallet?.pubkeyHex || ''} onChange={(e) => setSelectedWallet(adminWallets.find((w) => w.pubkeyHex === e.target.value) || null)}>
                  <option value="">请选择…</option>
                  {adminWallets.map((w) => <option key={w.pubkeyHex} value={w.pubkeyHex}>{w.name}</option>)}
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
            <button className="vote-signing-confirm" disabled={!selectedWallet || approve === null} onClick={generateRequest}>生成签名请求</button>
          </div>
        )}

        {step === 'qr' && (
          <div className="vote-signing-body qr-step">
            <p className="qr-instruction">用 wumin 离线设备扫描此二维码完成签名</p>
            <div className="qr-container"><QRCodeSVG value={requestJson} size={280} level="L" /></div>
            <p className="qr-countdown">剩余 <strong>{countdown}</strong> 秒</p>
            <button className="vote-signing-confirm" onClick={() => setStep('scan')}>已签名，扫描回执</button>
          </div>
        )}

        {step === 'scan' && (
          <div className="vote-signing-body">
            <p className="qr-instruction">将签名回执二维码对准摄像头</p>
            <QrScanner onScan={handleScanResult} onError={(e) => { setError(e); setStep('error'); }} />
            <button className="cancel-button" onClick={() => setStep('qr')}>返回</button>
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

/**
 * 统一投票入口 call 编码:`[0x16][0x00][proposal_id:u64_le][approve:bool]` = 11 bytes。
 *
 * 所有业务 pallet 的 vote_X / finalize_X 已物理删除,管理员一人一票
 * 一律走 InternalVote::cast(pallet=22, call=0)(2026-05-05 sub-pallet 拆分,
 * 原 VotingEngine.internal_vote 迁出)。
 */
function buildInternalVoteCallDataHex(proposalId: number, approve: boolean): string {
  const buf = new ArrayBuffer(11);
  const view = new DataView(buf);
  const arr = new Uint8Array(buf);
  arr[0] = 22; arr[1] = 0; // InternalVote.cast (2026-05-05 sub-pallet 拆分,原 VotingEngine.internal_vote)
  view.setUint32(2, proposalId & 0xFFFFFFFF, true);
  view.setUint32(6, Math.floor(proposalId / 0x100000000), true);
  arr[10] = approve ? 1 : 0;
  return Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
}
