// 节点信息长卡片:展示链上 ClearingBankNodes[sfid_id] 字段 + 端点更新 / 注销入口。

import { useEffect, useRef, useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../core/tauri';
import { QrScanner } from '../shared/qr/QrScanner';
import type { AdminWalletMatch, VoteSignRequestResult } from '../governance/types';
import { offchainApi } from './api';
import type { ClearingBankNodeOnChainInfo } from './types';

type Props = {
  info: ClearingBankNodeOnChainInfo;
  sfidId: string;
  /** 当前已激活管理员(用作签名者下拉)。空则不允许更新/注销。 */
  admins: AdminWalletMatch[];
  onChanged: () => void;
  onUnregistered: () => void;
};

type Mode = 'idle' | 'update' | 'unregister';
type Step = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function ClearingBankNodeInfoPanel({ info, sfidId, admins, onChanged, onUnregistered }: Props) {
  const [mode, setMode] = useState<Mode>('idle');
  const [step, setStep] = useState<Step>('form');
  const [error, setError] = useState<string | null>(null);
  const [selectedAdmin, setSelectedAdmin] = useState<AdminWalletMatch | null>(
    admins.length === 1 ? admins[0] : null,
  );
  const [newDomain, setNewDomain] = useState(info.rpcDomain);
  const [newPort, setNewPort] = useState<string>(String(info.rpcPort));
  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [countdown, setCountdown] = useState(90);
  const [txHash, setTxHash] = useState<string | null>(null);
  const signRequestRef = useRef(signRequest);
  signRequestRef.current = signRequest;

  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) {
      setError('签名请求已过期'); setStep('error');
      return;
    }
    const t = setTimeout(() => setCountdown(c => c - 1), 1000);
    return () => clearTimeout(t);
  }, [step, countdown]);

  const reset = () => {
    setMode('idle'); setStep('form'); setError(null);
    setSignRequest(null); setTxHash(null);
    setNewDomain(info.rpcDomain); setNewPort(String(info.rpcPort));
  };

  const startUpdate = async () => {
    if (!selectedAdmin) { setError('请先选择已激活管理员'); return; }
    const portNum = parseInt(newPort, 10);
    if (Number.isNaN(portNum) || portNum < 1024 || portNum > 65535) {
      setError('端口需在 1024-65535'); return;
    }
    setError(null); setStep('qr'); setCountdown(90);
    try {
      const r = await offchainApi.buildUpdateClearingBankEndpointRequest(
        selectedAdmin.pubkeyHex, sfidId, newDomain.trim(), portNum,
      );
      setSignRequest(r);
    } catch (e) { setError(sanitizeError(e)); setStep('error'); }
  };

  const startUnregister = async () => {
    if (!selectedAdmin) { setError('请先选择已激活管理员'); return; }
    setError(null); setStep('qr'); setCountdown(90);
    try {
      const r = await offchainApi.buildUnregisterClearingBankRequest(selectedAdmin.pubkeyHex, sfidId);
      setSignRequest(r);
    } catch (e) { setError(sanitizeError(e)); setStep('error'); }
  };

  const handleScan = async (responseJson: string) => {
    const sr = signRequestRef.current;
    if (!sr || !selectedAdmin) return;
    setStep('submit');
    try {
      let r;
      if (mode === 'update') {
        r = await offchainApi.submitUpdateClearingBankEndpoint(
          sr.requestId, selectedAdmin.pubkeyHex, sr.expectedPayloadHash,
          sfidId, newDomain.trim(), parseInt(newPort, 10),
          sr.signNonce, sr.signBlockNumber, responseJson,
        );
      } else {
        r = await offchainApi.submitUnregisterClearingBank(
          sr.requestId, selectedAdmin.pubkeyHex, sr.expectedPayloadHash,
          sfidId, sr.signNonce, sr.signBlockNumber, responseJson,
        );
      }
      setTxHash(r.txHash);
      setStep('done');
      setTimeout(() => {
        if (mode === 'unregister') onUnregistered();
        else onChanged();
        reset();
      }, 1500);
    } catch (e) { setError(sanitizeError(e)); setStep('error'); }
  };

  return (
    <div className="metric-card node-info-panel">
      <h3>清算行节点信息</h3>
      <dl>
        <dt>PeerId</dt>
        <dd><code>{info.peerId}</code></dd>
        <dt>RPC 端点</dt>
        <dd><code>{info.rpcDomain}:{info.rpcPort}</code></dd>
        <dt>注册区块</dt>
        <dd>#{info.registeredAt}</dd>
        <dt>注册管理员</dt>
        <dd><code>{info.registeredBySs58}</code></dd>
      </dl>

      {admins.length > 1 && mode !== 'idle' && (
        <div className="form-group">
          <label>签名管理员</label>
          <select
            value={selectedAdmin?.pubkeyHex ?? ''}
            onChange={(e) => {
              const a = admins.find(x => x.pubkeyHex === e.target.value);
              setSelectedAdmin(a ?? null);
            }}
          >
            <option value="">— 选择管理员 —</option>
            {admins.map(a => <option key={a.pubkeyHex} value={a.pubkeyHex}>{a.address}</option>)}
          </select>
        </div>
      )}

      {mode === 'idle' && (
        <div className="form-actions">
          <button className="secondary-button" onClick={() => setMode('update')} disabled={admins.length === 0}>
            更新 RPC 端点
          </button>
          <button className="secondary-button" onClick={() => setMode('unregister')} disabled={admins.length === 0}>
            注销节点
          </button>
        </div>
      )}

      {mode === 'update' && step === 'form' && (
        <>
          <div className="form-group">
            <label>新域名</label>
            <input type="text" value={newDomain} onChange={(e) => setNewDomain(e.target.value)} />
          </div>
          <div className="form-group">
            <label>新端口</label>
            <input type="number" value={newPort} onChange={(e) => setNewPort(e.target.value)} />
          </div>
          <div className="form-actions">
            <button className="primary-button" onClick={startUpdate}>扫码签名提交</button>
            <button className="secondary-button" onClick={reset}>取消</button>
          </div>
        </>
      )}

      {mode === 'unregister' && step === 'form' && (
        <>
          <p className="warning">
            注销后该机构将退出清算网络,wuminapp 不会再把它列为可绑定清算行。
            已绑定到该机构的用户需主动 switch_bank 切换。
          </p>
          <div className="form-actions">
            <button className="primary-button" onClick={startUnregister}>扫码签名注销</button>
            <button className="secondary-button" onClick={reset}>取消</button>
          </div>
        </>
      )}

      {step === 'qr' && signRequest && (
        <div className="modal-overlay" onClick={reset}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>{mode === 'update' ? '扫码签名更新端点' : '扫码签名注销节点'}</h3>
            <div className="qr-container">
              <QRCodeSVG value={signRequest.requestJson} size={280} level="L" />
            </div>
            <p className="countdown">有效时间:{countdown} 秒</p>
            <button className="primary-button" onClick={() => setStep('scan')}>已签名,扫描回执</button>
            <button className="secondary-button" onClick={reset}>取消</button>
          </div>
        </div>
      )}

      {step === 'scan' && (
        <div className="modal-overlay" onClick={() => setStep('qr')}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>扫描签名回执</h3>
            <QrScanner
              onScan={handleScan}
              onError={(e) => { setError(e); setStep('error'); }}
            />
            <button className="secondary-button" onClick={() => setStep('qr')}>返回二维码</button>
          </div>
        </div>
      )}

      {step === 'submit' && (
        <div className="modal-overlay"><div className="modal-content"><p>正在提交…</p></div></div>
      )}
      {step === 'done' && txHash && (
        <div className="modal-overlay">
          <div className="modal-content">
            <p>{mode === 'update' ? '端点已更新' : '节点已注销'}</p>
            <code>{txHash}</code>
          </div>
        </div>
      )}
      {step === 'error' && error && (
        <div className="modal-overlay" onClick={reset}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="error">{error}</div>
            <button className="secondary-button" onClick={reset}>关闭</button>
          </div>
        </div>
      )}
    </div>
  );
}
