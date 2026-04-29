import { useState, useEffect, useRef, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../../core/tauri';
import { QrScanner } from '../../shared/qr/QrScanner';
import { transactionApi as api } from './api';
import type { ColdWallet, TransferSignRequestResult } from './types';

type Props = {
  wallet: ColdWallet;
  toAddress: string;
  amountYuan: number;
  onClose: () => void;
  onSuccess: (txHash: string) => void;
};

type FlowStep = 'confirm' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

function truncateAddress(addr: string): string {
  if (addr.length <= 14) return addr;
  return addr.slice(0, 8) + '...' + addr.slice(-6);
}

function fmtYuan(v: number): string {
  const fixed = v.toFixed(2);
  const [int, dec] = fixed.split('.');
  return `${int.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}.${dec}`;
}

export function TransferSigningFlow({ wallet, toAddress, amountYuan, onClose, onSuccess }: Props) {
  const [step, setStep] = useState<FlowStep>('confirm');
  const [signRequest, setSignRequest] = useState<TransferSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const signRequestRef = useRef(signRequest);
  signRequestRef.current = signRequest;

  const fee = Math.max(amountYuan * 0.001, 0.10);

  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) {
      setError('签名请求已过期，请重新操作');
      setStep('error');
      return;
    }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  const handleConfirm = useCallback(async () => {
    try {
      const result = await api.buildTransferRequest(wallet.pubkeyHex, toAddress, amountYuan);
      setSignRequest(result);
      setRequestJson(result.requestJson);
      setCountdown(90);
      setStep('qr');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [wallet.pubkeyHex, toAddress, amountYuan]);

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    if (!req) {
      setError('签名请求数据丢失，请重试');
      setStep('error');
      return;
    }
    setStep('submit');
    try {
      const result = await api.submitTransfer(
        req.requestId,
        wallet.pubkeyHex,
        req.expectedPayloadHash,
        req.callDataHex,
        req.signNonce,
        req.signBlockNumber,
        responseText,
      );
      setTxHash(result.txHash);
      setStep('done');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [wallet.pubkeyHex]);

  return (
    <div className="transfer-signing-overlay">
      <div className="transfer-signing-modal">
        {/* 统一右上角关闭叉 */}
        <div className="transfer-signing-header">
          <h3>转账签名</h3>
          <span className="transfer-signing-close" onClick={onClose}>&times;</span>
        </div>

        {step === 'confirm' && (
          <div className="transfer-signing-body">
            <div className="transfer-signing-summary">
              <div className="transfer-signing-row">
                <span className="transfer-signing-label">付款地址</span>
                <span className="transfer-signing-value">{wallet.address}</span>
              </div>
              <div className="transfer-signing-row">
                <span className="transfer-signing-label">收款地址</span>
                <span className="transfer-signing-value">{toAddress}</span>
              </div>
              <div className="transfer-signing-row">
                <span className="transfer-signing-label">转账金额</span>
                <span className="transfer-signing-value">{fmtYuan(amountYuan)} 元</span>
              </div>
              <div className="transfer-signing-row">
                <span className="transfer-signing-label">预估手续费</span>
                <span className="transfer-signing-value">{fmtYuan(fee)} 元</span>
              </div>
            </div>
            <div className="transfer-signing-actions">
              <button className="transfer-signing-confirm" onClick={handleConfirm}>确认转账</button>
              <button className="cancel-button" onClick={onClose}>取消</button>
            </div>
          </div>
        )}

        {step === 'qr' && (
          <div className="transfer-signing-body qr-step">
            <p className="qr-instruction">用离线设备扫描此二维码完成签名</p>
            <div className="transfer-qr-box">
              <QRCodeSVG value={requestJson} size={240} level="L" />
            </div>
            <p className="qr-countdown">剩余 <strong>{countdown}</strong> 秒</p>
            <button className="transfer-signing-confirm" onClick={() => setStep('scan')}>
              已签名，扫描回执
            </button>
          </div>
        )}

        {step === 'scan' && (
          <div className="transfer-signing-body">
            <p className="qr-instruction">将签名回执二维码对准摄像头</p>
            <QrScanner
              onScan={handleScanResult}
              onError={(e) => { setError(e); setStep('error'); }}
            />
            <button className="cancel-button" onClick={() => setStep('qr')}>返回</button>
          </div>
        )}

        {step === 'submit' && (
          <div className="transfer-signing-body">
            <p className="qr-instruction">提交中...</p>
          </div>
        )}

        {step === 'done' && (
          <div className="transfer-signing-body">
            <div className="transfer-success">
              <p>转账已提交</p>
              {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
            </div>
            <button
              className="transfer-signing-confirm"
              onClick={() => { if (txHash) onSuccess(txHash); onClose(); }}
            >
              完成
            </button>
          </div>
        )}

        {step === 'error' && (
          <div className="transfer-signing-body">
            <div className="error">{error}</div>
            <button
              className="transfer-signing-confirm"
              onClick={() => { setError(null); setStep('confirm'); }}
            >
              重试
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
