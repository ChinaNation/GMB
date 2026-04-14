import { useState, useEffect, useRef, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import type { ColdWallet, WalletStore } from './transaction-types';
import { WalletManagerModal } from './WalletManagerModal';
import { TransferForm } from './TransferForm';
import { TransferSigningFlow } from './TransferSigningFlow';

export function TransactionPanel() {
  const [wallets, setWallets] = useState<ColdWallet[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [balance, setBalance] = useState<string | null>(null);
  const [showWalletModal, setShowWalletModal] = useState(false);
  const [signingFlow, setSigningFlow] = useState<{ toAddress: string; amountYuan: number } | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const successTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const activeWallet = wallets.find((w) => w.id === activeId) ?? null;

  // 加载钱包列表
  useEffect(() => {
    api.getWallets().then((store) => {
      setWallets(store.wallets);
      setActiveId(store.activeId);
    }).catch((e) => setError(sanitizeError(e)));
  }, []);

  // 轮询当前钱包余额（5s）
  useEffect(() => {
    if (!activeWallet) { setBalance(null); return; }
    let cancelled = false;
    const fetch = () => {
      api.getWalletBalance(activeWallet.pubkeyHex)
        .then((b) => { if (!cancelled) setBalance(b); })
        .catch(() => { if (!cancelled) setBalance(null); });
    };
    fetch();
    const interval = setInterval(fetch, 5000);
    return () => { cancelled = true; clearInterval(interval); };
  }, [activeWallet?.pubkeyHex]);

  const handleWalletUpdate = useCallback((store: WalletStore) => {
    setWallets(store.wallets);
    setActiveId(store.activeId);
  }, []);

  // 弹窗中选中钱包后，立即更新余额
  const handleWalletSelect = useCallback((_wallet: ColdWallet, balanceFen: string | null) => {
    setBalance(balanceFen);
  }, []);

  const handleTransferSubmit = useCallback((toAddress: string, amountYuan: number) => {
    setSigningFlow({ toAddress, amountYuan });
  }, []);

  const handleTransferSuccess = useCallback((txHash: string) => {
    setSigningFlow(null);
    setSuccessMsg(`转账成功，哈希: ${txHash.slice(0, 16)}...`);
    if (successTimer.current) clearTimeout(successTimer.current);
    successTimer.current = setTimeout(() => setSuccessMsg(null), 5000);
  }, []);

  return (
    <div className="transaction-panel">
      <div className="transaction-panel-header">
        <h2>交易</h2>
        <button className="wallet-manage-btn" onClick={() => setShowWalletModal(true)}>
          钱包管理
        </button>
      </div>

      {error && <div className="error">{error}</div>}
      {successMsg && <div className="transfer-success-msg">{successMsg}</div>}

      <TransferForm
        activeWallet={activeWallet}
        balance={balance}
        onSubmit={handleTransferSubmit}
        disabled={signingFlow != null}
      />

      {signingFlow && activeWallet && (
        <TransferSigningFlow
          wallet={activeWallet}
          toAddress={signingFlow.toAddress}
          amountYuan={signingFlow.amountYuan}
          onClose={() => setSigningFlow(null)}
          onSuccess={handleTransferSuccess}
        />
      )}

      {showWalletModal && (
        <WalletManagerModal
          wallets={wallets}
          activeId={activeId}
          onClose={() => setShowWalletModal(false)}
          onUpdate={handleWalletUpdate}
          onSelect={handleWalletSelect}
        />
      )}
    </div>
  );
}
