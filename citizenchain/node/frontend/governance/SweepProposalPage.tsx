// 手续费划转提案页面：金额输入 + QR 签名流程。
import { useState, useRef, useEffect, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { hexToSs58 } from '../format';
import { QrScanner } from './QrScanner';
import type { AdminWalletMatch, VoteSignRequestResult } from './governance-types';

type Props = {
  shenfenId: string;
  institutionName: string;
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

type Step = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function SweepProposalPage({
  shenfenId, institutionName, adminWallets, onBack, onSuccess,
}: Props) {
  const [step, setStep] = useState<Step>('form');
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null
  );
  const [amountYuan, setAmountYuan] = useState('');
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const formValuesRef = useRef({ amountYuan: 0 });
  const signRequestRef = useRef(signRequest);
  const selectedWalletRef = useRef(selectedWallet);
  signRequestRef.current = signRequest;
  selectedWalletRef.current = selectedWallet;

  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) { setError('签名请求已过期'); setStep('error'); return; }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  const handleSubmit = async () => {
    if (!selectedWallet) { setFormError('请选择管理员钱包'); return; }
    const amount = parseFloat(amountYuan);
    if (isNaN(amount) || amount <= 0) { setFormError('金额必须大于 0'); return; }
    setFormError(null);
    setSubmitting(true);

    try {
      formValuesRef.current = { amountYuan: amount };
      const result = await api.buildProposeSweepRequest(
        selectedWallet.pubkeyHex, shenfenId, amount,
      );
      setSignRequest(result);
      setRequestJson(result.requestJson);
      setCountdown(90);
      setStep('qr');
    } catch (e) {
      setFormError(sanitizeError(e));
    } finally {
      setSubmitting(false);
    }
  };

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const wallet = selectedWalletRef.current;
    if (!req || !wallet) { setError('数据丢失，请重试'); setStep('error'); return; }
    setStep('submit');
    try {
      const result = await api.submitProposeSweep(
        req.requestId, wallet.pubkeyHex, req.expectedPayloadHash,
        shenfenId, formValuesRef.current.amountYuan,
        req.signNonce, req.signBlockNumber, responseText,
      );
      setTxHash(result.txHash);
      setStep('done');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [shenfenId]);

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回</button>
      <h2>手续费划转提案</h2>
      <p className="proposal-institution-name">{institutionName}</p>

      {step === 'form' && (
        <div className="create-proposal-form">
          {formError && <div className="error">{formError}</div>}
          <div className="wallet-form-field">
            <label>发起管理员</label>
            <select
              value={selectedWallet?.pubkeyHex || ''}
              onChange={(e) => setSelectedWallet(adminWallets.find((w) => w.pubkeyHex === e.target.value) || null)}
              disabled={adminWallets.length <= 1}
            >
              {adminWallets.length === 0 && <option value="">无已激活管理员</option>}
              {adminWallets.length === 1 ? (
                <option value={adminWallets[0].pubkeyHex}>{hexToSs58(adminWallets[0].pubkeyHex)}</option>
              ) : (
                <>
                  <option value="">请选择…</option>
                  {adminWallets.map((w) => (
                    <option key={w.pubkeyHex} value={w.pubkeyHex}>{hexToSs58(w.pubkeyHex)}</option>
                  ))}
                </>
              )}
            </select>
          </div>
          <div className="wallet-form-field">
            <label>划转金额（元）</label>
            <input
              type="number" value={amountYuan}
              onChange={(e) => setAmountYuan(e.target.value)}
              placeholder="0.00" min="0.01" step="0.01"
              disabled={submitting}
            />
          </div>
          <p style={{ fontSize: 12, color: '#888', marginTop: 4 }}>
            划转后手续费账户至少保留 1111.11 元，单次不超过可用余额的 80%
          </p>
          <button
            className="vote-signing-confirm"
            onClick={handleSubmit}
            disabled={submitting || !selectedWallet || !amountYuan}
          >
            {submitting ? '生成中…' : '生成签名请求'}
          </button>
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

      {step === 'submit' && (
        <div className="vote-signing-body"><p className="qr-instruction">正在提交…</p></div>
      )}

      {step === 'done' && (
        <div className="vote-signing-body">
          <div className="vote-success">
            <p>手续费划转提案已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <button className="vote-signing-confirm" onClick={onSuccess}>完成</button>
        </div>
      )}

      {step === 'error' && (
        <div className="vote-signing-body">
          <div className="error">{error}</div>
          <button className="vote-signing-confirm" onClick={() => { setError(null); setStep('form'); }}>重试</button>
        </div>
      )}
    </div>
  );
}
