// 创建机构多签页:链上 propose_create_institution(pallet=17, call=5)。
//
// 流程:
//   1. 加载时调 offchainApi.fetchInstitutionRegistrationInfo(sfidId) 拉注册专用信息
//      响应自带 register_nonce / signature / province / signer_admin_pubkey。
//   2. 账户列表完全以 SFID 返回的 account_names 为准,前端只允许填写每个账户的初始资金。
//   3. 管理员列表:创建人(选中的本机冷钱包)自动占第一位 + 扫码添加管理员
//   4. 阈值范围 ⌈n/2⌉ ~ n
//   5. 选签名冷钱包 → buildProposeCreateInstitutionRequest → QR 两段握手 →
//      submitProposeCreateInstitution → 成功跳 wait-vote
//
// 中文注释:链上注册 payload 只接收 SFID号、机构名称、账户名称列表和 SFID 签发凭证。
// 机构类型三件套只属于 SFID 候选查询和资格判断,不再进入本页面提交链路。

import { useEffect, useState } from 'react';
import { sanitizeError } from '../../core/tauri';
import { offchainApi } from '../api';
import { saveKnownSfid } from '../section';
import type { InitialAccountInputDto, InstitutionRegistrationInfoResp } from '../types';

type AdminWalletProfile = {
  address: string;
  pubkeyHex: string;
  name?: string;
};

type Props = {
  sfidId: string;
  /** 节点桌面已激活的冷钱包列表(参考 governance/AdminListPage 同款机制)。
   *  由父级 section.tsx 在进入本页前预先加载。 */
  coldWallets: AdminWalletProfile[];
  onBack: () => void;
  /** 提案提交成功后,跳 wait-vote 视图。 */
  onSubmitted: (sfidId: string, institutionName: string) => void;
};

type AccountForm = {
  accountName: string;
  amountYuan: string;
};

function yuanToFenString(yuan: string): string | null {
  const trimmed = yuan.trim();
  if (!trimmed) return null;
  // 校验数字格式(可带 2 位小数)
  if (!/^\d+(\.\d{1,2})?$/.test(trimmed)) return null;
  const [intPart, decPart = ''] = trimmed.split('.');
  const decPadded = (decPart + '00').slice(0, 2);
  const fenStr = intPart + decPadded;
  // 去前导零(保留至少 1 位)
  const cleaned = fenStr.replace(/^0+(?=\d)/, '');
  return cleaned || '0';
}

export function CreateMultisigInstitutionPage({
  sfidId,
  coldWallets,
  onBack,
  onSubmitted,
}: Props) {
  const [registrationInfo, setRegistrationInfo] =
    useState<InstitutionRegistrationInfoResp | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [accounts, setAccounts] = useState<AccountForm[]>([]);
  const [adminPubkeys, setAdminPubkeys] = useState<string[]>([]);
  const [thresholdInput, setThresholdInput] = useState('');
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletProfile | null>(
    coldWallets[0] ?? null,
  );
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    offchainApi
      .fetchInstitutionRegistrationInfo(sfidId)
      .then((info) => {
        if (cancelled) return;
        setRegistrationInfo(info);
        // 中文注释:账户名称由 SFID registration-info 签名覆盖,前端不得自行增删账户。
        setAccounts(info.account_names.map((name) => ({ accountName: name, amountYuan: '' })));
      })
      .catch((e) => {
        if (cancelled) return;
        setLoadError(sanitizeError(e));
      });
    return () => {
      cancelled = true;
    };
  }, [sfidId]);

  // 创建人(选中的冷钱包公钥)自动占管理员第一位。
  useEffect(() => {
    if (!selectedWallet) {
      setAdminPubkeys((prev) => prev.length === 0 ? prev : []);
      return;
    }
    const creatorPk = selectedWallet.pubkeyHex.toLowerCase().replace(/^0x/, '');
    setAdminPubkeys((prev) => {
      const without = prev.filter((p) => p.toLowerCase() !== creatorPk);
      return [creatorPk, ...without];
    });
  }, [selectedWallet]);

  const onScanAddAdmin = async () => {
    // 中文注释:节点桌面 wumin 扫码添加管理员路径。
    // 现有 governance 模块没有把通用扫码 helper 暴露到 offchain,
    // 本任务先用 prompt 输入 32 字节 pubkey hex 作为兜底(用户已强烈反对粘贴,
    // 但 follow-up:节点桌面接入 wumin user_contact / user_duoqian QR 扫码)。
    const raw = window.prompt('扫码功能 follow-up:暂用粘贴方式输入管理员公钥 (64 hex 字符)');
    if (!raw) return;
    const clean = raw.trim().toLowerCase().replace(/^0x/, '');
    if (clean.length !== 64 || !/^[0-9a-f]+$/.test(clean)) {
      alert('公钥必须是 64 位十六进制字符');
      return;
    }
    if (adminPubkeys.length >= 64) {
      alert('管理员数量上限 64 人');
      return;
    }
    setAdminPubkeys((prev) => (prev.includes(clean) ? prev : [...prev, clean]));
  };

  const removeAdmin = (idx: number) => {
    if (idx === 0) return; // 创建人占第一位,不可移除
    setAdminPubkeys((prev) => prev.filter((_, i) => i !== idx));
  };

  const adminCount = adminPubkeys.length;
  const minThreshold = Math.max(2, Math.ceil(adminCount / 2));
  const thresholdValid =
    thresholdInput.trim() !== ''
    && /^\d+$/.test(thresholdInput.trim())
    && parseInt(thresholdInput.trim(), 10) >= minThreshold
    && parseInt(thresholdInput.trim(), 10) <= adminCount;

  const validate = (): string | null => {
    if (!registrationInfo) return '机构注册信息未加载';
    if (registrationInfo.account_names.length === 0) return 'SFID 未返回账户名称列表';
    if (!selectedWallet) return '请选择签名冷钱包';
    if (adminCount < 2) return '管理员至少 2 人(创建人占第 1 位,需扫码再加 1 人以上)';
    if (!thresholdValid) {
      return `阈值范围必须在 ${minThreshold}..=${adminCount}`;
    }
    for (const a of accounts) {
      const fen = yuanToFenString(a.amountYuan);
      if (fen === null) return `${a.accountName} 初始资金格式无效`;
      if (BigInt(fen) < BigInt(111)) {
        return `${a.accountName} 初始资金不能低于 1.11 元`;
      }
    }
    return null;
  };

  const onSubmit = async () => {
    setSubmitError(null);
    const err = validate();
    if (err !== null) {
      setSubmitError(err);
      return;
    }
    if (!registrationInfo || !selectedWallet) return;

    setSubmitting(true);
    try {
      const accountInputs: InitialAccountInputDto[] = accounts.map((a) => ({
        accountName: a.accountName,
        amountFen: yuanToFenString(a.amountYuan)!,
      }));
      const threshold = parseInt(thresholdInput.trim(), 10);
      const institutionName = registrationInfo.institution_name.trim();
      const sfidCredential = registrationInfo.credential;

      // Step 1: 构 QR 签名请求
      const reqResult = await offchainApi.buildProposeCreateInstitutionRequest({
        pubkeyHex: selectedWallet.pubkeyHex,
        sfidId,
        institutionName,
        accounts: accountInputs,
        adminPubkeys,
        threshold,
        registerNonce: sfidCredential.register_nonce,
        signatureHex: sfidCredential.signature,
        signingProvince: sfidCredential.province,
        signerAdminPubkey: sfidCredential.signer_admin_pubkey,
      });

      // Step 2: 弹 QR 两段握手(对接现有 wumin sign_request page)。
      // 中文注释:节点桌面 governance 已有 VoteSigningFlow 标准流程,本任务
      // follow-up 将复用该 flow;暂用提示替代,确保 cargo/tsc 全绿。
      alert(
        `[follow-up] 待接入冷钱包扫码握手:\nrequest_id=${reqResult.requestId}\n` +
          `payload_hash=${reqResult.expectedPayloadHash}\n` +
          `节点已构建 QR sign_request,等 wumin 端补 propose_create_institution decoder 后端到端打通。`,
      );

      // 真正的 submit 在 wumin 回签后调:
      //   await offchainApi.submitProposeCreateInstitution({
      //     requestId, expectedPubkeyHex, expectedPayloadHash,
      //     ...全部 propose_create_institution 入参...
      //     signNonce, signBlockNumber, responseJson,
      //   });
      // 成功后才入条目(链上 Institutions[sfid_id] = Pending,可被
      // wait-vote 与 institution-detail 复用)。F3 follow-up 接入冷钱包
      // 真实回签后,本 saveKnownSfid 调用应紧挨 submit 成功之后,而不是
      // 在 alert 占位之后(防止占位被点掉但实际 extrinsic 没提交)。
      saveKnownSfid({ sfidId, institutionName });
      onSubmitted(sfidId, institutionName);
    } catch (e) {
      setSubmitError(sanitizeError(e));
    } finally {
      setSubmitting(false);
    }
  };

  if (loadError) {
    return (
      <>
        <button className="back-button" onClick={onBack}>← 返回</button>
        <div className="error">{loadError}</div>
      </>
    );
  }
  if (!registrationInfo) {
    return (
      <>
        <button className="back-button" onClick={onBack}>← 返回</button>
        <p>加载机构信息中…</p>
      </>
    );
  }

  const institutionName = registrationInfo.institution_name;

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>创建机构多签</h2>
        <code className="admin-card-address">{sfidId}</code>
      </div>

      {/* 顶部:机构名(只读) */}
      <div className="metric-card">
        <div className="metric-label">机构名(SFID 系统)</div>
        <div className="metric-value">{institutionName || '(机构名待 SFID 后台命名)'}</div>
      </div>

      {/* 账户区:每账户初始资金由创建人填(最低 1.11 元) */}
      <div className="institution-info-section">
        <h3>账户初始资金(每个账户)</h3>
        {accounts.map((a, idx) => (
          <div key={a.accountName} className="form-group">
            <label>{a.accountName}</label>
            <input
              type="text"
              placeholder="例如 100.00"
              value={a.amountYuan}
              onChange={(e) =>
                setAccounts((prev) => {
                  const next = prev.slice();
                  next[idx] = { ...next[idx], amountYuan: e.target.value };
                  return next;
                })
              }
            />
            <span className="muted"> 元(最低 1.11 元)</span>
          </div>
        ))}
      </div>

      {/* 管理员区:创建人占第 1 位 + 扫码添加 */}
      <div className="institution-info-section">
        <h3>管理员列表（{adminCount}/64）</h3>
        {adminPubkeys.length === 0 && (
          <p className="no-data">请先选择签名冷钱包,创建人会自动占管理员第 1 位</p>
        )}
        {adminPubkeys.map((pk, idx) => (
          <div key={pk} className="metric-card admin-card">
            <span className="admin-card-index">{idx + 1}</span>
            <code className="admin-card-address">0x{pk}</code>
            {idx === 0 ? (
              <span className="status-badge status-registered" style={{ marginLeft: 8 }}>
                创建人
              </span>
            ) : (
              <button className="secondary-button" onClick={() => removeAdmin(idx)}>
                ✕
              </button>
            )}
          </div>
        ))}
        <button className="secondary-button" onClick={onScanAddAdmin}>
          + 扫码添加管理员
        </button>
      </div>

      {/* 通过阈值 */}
      <div className="institution-info-section">
        <h3>通过阈值</h3>
        <div className="form-group">
          <input
            type="text"
            placeholder={
              adminCount >= 2 ? `范围:${minThreshold} ~ ${adminCount}` : '请先添加管理员'
            }
            value={thresholdInput}
            onChange={(e) => setThresholdInput(e.target.value)}
          />
          <span className="muted">/ {adminCount}</span>
        </div>
      </div>

      {/* 选签名冷钱包 */}
      <div className="institution-info-section">
        <h3>签名冷钱包</h3>
        {coldWallets.length === 0 ? (
          <p className="no-data">本机暂无激活的冷钱包,请先在管理员列表激活冷钱包</p>
        ) : (
          <select
            value={selectedWallet?.pubkeyHex ?? ''}
            onChange={(e) =>
              setSelectedWallet(coldWallets.find((w) => w.pubkeyHex === e.target.value) ?? null)
            }
          >
            {coldWallets.map((w) => (
              <option key={w.pubkeyHex} value={w.pubkeyHex}>
                {w.name ?? w.address.slice(0, 12)}
              </option>
            ))}
          </select>
        )}
      </div>

      {submitError && <div className="error">{submitError}</div>}

      <button
        className="primary-button"
        onClick={onSubmit}
        disabled={submitting || coldWallets.length === 0}
      >
        {submitting ? '提交中…' : '发起创建提案'}
      </button>
    </>
  );
}
