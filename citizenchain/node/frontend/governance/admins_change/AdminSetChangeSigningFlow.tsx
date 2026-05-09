import { useEffect, useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { QrScanner } from '../../shared/qr/QrScanner';
import type { VoteSignRequestResult } from './types';

type Props = {
  request: VoteSignRequestResult;
  requestJson: string;
  submitting: boolean;
  error: string | null;
  txHash: string | null;
  onScan: (responseJson: string) => void;
  onBackToForm: () => void;
  onDone: () => void;
};

type Step = 'qr' | 'scan';

export function AdminSetChangeSigningFlow({
  request,
  requestJson,
  submitting,
  error,
  txHash,
  onScan,
  onBackToForm,
  onDone,
}: Props) {
  const [step, setStep] = useState<Step>('qr');
  const [countdown, setCountdown] = useState(90);

  useEffect(() => {
    if (step !== 'qr' || txHash || error) return;
    if (countdown <= 0) return;
    const timer = setTimeout(() => setCountdown((value) => value - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown, txHash, error]);

  if (txHash) {
    return (
      <div className="vote-signing-body">
        <div className="vote-success">
          <p>管理员更换提案已提交</p>
          <code className="tx-hash">交易哈希: {txHash}</code>
        </div>
        <button className="vote-signing-confirm" onClick={onDone}>完成</button>
      </div>
    );
  }

  if (submitting) {
    return <div className="vote-signing-body"><p className="qr-instruction">正在提交管理员更换提案…</p></div>;
  }

  if (error) {
    return (
      <div className="vote-signing-body">
        <div className="error">{error}</div>
        <button className="vote-signing-confirm" onClick={onBackToForm}>返回修改</button>
      </div>
    );
  }

  if (step === 'scan') {
    return (
      <div className="vote-signing-body">
        <p className="qr-instruction">将签名回执二维码对准摄像头</p>
        <QrScanner onScan={onScan} onError={(_error) => setStep('qr')} />
        <button className="cancel-button" onClick={() => setStep('qr')}>返回</button>
      </div>
    );
  }

  return (
    <div className="vote-signing-body qr-step">
      <p className="qr-instruction">用 wumin 离线设备扫描此二维码完成签名</p>
      <div className="qr-container"><QRCodeSVG value={requestJson} size={280} level="L" /></div>
      <p className="qr-countdown">剩余 <strong>{countdown}</strong> 秒</p>
      <code className="tx-hash">请求 ID: {request.requestId}</code>
      <button className="vote-signing-confirm" onClick={() => setStep('scan')}>已签名，扫描回执</button>
      <button className="cancel-button" onClick={onBackToForm}>返回修改</button>
    </div>
  );
}
