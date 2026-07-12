import { useEffect, useState } from 'react';
import { CitizenSignaturePanel } from '../shared/qr/CitizenSignaturePanel';

type Props = {
  requestJson: string;
  submitting: boolean;
  error: string | null;
  txHash: string | null;
  onScan: (responseJson: string) => void;
  onBackToForm: () => void;
  onDone: () => void;
};

export function AdminSetChangeSigningFlow({
  requestJson,
  submitting,
  error,
  txHash,
  onScan,
  onBackToForm,
  onDone,
}: Props) {
  const [countdown, setCountdown] = useState(90);
  const [scanError, setScanError] = useState<string | null>(null);

  useEffect(() => {
    if (txHash || error || submitting) return;
    if (countdown <= 0) return;
    const timer = setTimeout(() => setCountdown((value) => value - 1), 1000);
    return () => clearTimeout(timer);
  }, [countdown, txHash, error, submitting]);

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

  return (
    <div className="vote-signing-body">
      <CitizenSignaturePanel
        qrValue={requestJson}
        countdownSeconds={countdown}
        status={scanError ? 'error' : 'ready'}
        error={scanError}
        onScan={onScan}
        onScanError={setScanError}
      />
    </div>
  );
}
