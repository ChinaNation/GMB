// 清算行 tab 主入口。8 视图状态机驱动添加/管理流程。
//
// 重构(2026-05-01)后状态机:
//   empty                        列表 + ＋添加清算行
//   add-input-cid               输入 cid_number 自动 debounce 搜索 CID 候选(不再带"查询"按钮)
//   check-multisig               链上查 Institutions[cid_number]
//                                  ├─ 已存在 → institution-detail
//                                  └─ 不存在 → create-multisig-institution
//   institution-detail           机构详情 = 卡片栅格 + 折叠卡片(其他账户 / 管理员) + 节点信息
//                                顶部按钮:未声明节点 → declare-node;已声明 → 内联展示节点信息
//   other-accounts-list          子页:其他账户列表
//   admin-list                   子页:管理员列表
//   admin-set-change             子页:复用 governance/admins_change 更换管理员流程
//   create-multisig-institution  创建机构多签 propose_create_institution(冷钱包签 + 提交)
//   wait-vote                    等待管理员投票通过(轮询 Institutions[cid_number].status === 'Active')
//   declare-node                 多签 Active 但本机未声明节点 → 填 RPC + 自测 + 签名声明
//
// 已下架(2026-05-01):
//   - register-cid info 终态(CID 端没找到候选时改为内联红条)
//   - propose-create info 终态("去 CID 后台"长说明,改为直接进 create-multisig-institution)
//   - 老 detail 视图(只展示节点信息),并入 institution-detail 内联节点信息卡

import { useEffect, useState, useCallback } from 'react';
import { sanitizeError } from '../../core/tauri';
import { AdminSetChangePage } from '../../governance/admins_change';
import { adminsChangeApi } from '../../governance/admins_change/api';
import { organizationManageApi } from '../../governance/organization-manage/api';
import { ClearingBankAddPage } from '../../governance/organization-manage/add-candidate';
import { ClearingBankInstitutionDetailPage } from '../../governance/organization-manage/institution-detail';
import { CreateMultisigInstitutionPage } from '../../governance/organization-manage/create-multisig';
import { OtherAccountsListPage } from '../../governance/organization-manage/other-accounts';
import type {
  AccountWithBalance,
  EligibleClearingBankCandidate,
  InstitutionDetail,
} from '../../governance/organization-manage/types';
import { hexToSs58 } from '../../shared/ss58';
import { offchainApi } from './api';
import type { ClearingBankView } from './types';
import { ClearingBankDeclareNodePage } from './node-register';
import { ClearingBankAdminListPage } from './settlement/admin-unlock';
import './styles.css';

const STORAGE_KEY = 'gmb-clearing-bank-known-cids';

type KnownCidEntry = { cidNumber: string; cidFullName: string };

function loadKnownCids(): KnownCidEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (e): e is KnownCidEntry =>
        typeof e === 'object' && e !== null
        && typeof (e as { cidNumber?: unknown }).cidNumber === 'string'
        && typeof (e as { cidFullName?: unknown }).cidFullName === 'string',
    );
  } catch {
    return [];
  }
}

/// 把已确认存在(链上有 Institutions[cid_number] 或刚提交 propose_create_institution
/// 成功)的机构入条目加入本机已添加列表。**不要**在用户只是选了候选时调本函数,
/// 否则若链上没有 + 创建流程失败,EmptyView 会显示孤儿卡。
export function saveKnownCid(entry: KnownCidEntry) {
  const list = loadKnownCids().filter((e) => e.cidNumber !== entry.cidNumber);
  list.unshift(entry);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list.slice(0, 50)));
}

/// 从本机已添加列表中删除指定 cid_number 条目(链上判定为 None 或用户主动移除时调)。
function removeKnownCid(cidNumber: string) {
  const list = loadKnownCids().filter((e) => e.cidNumber !== cidNumber);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list.slice(0, 50)));
}

export function ClearingBankSection() {
  const [view, setView] = useState<ClearingBankView>({ kind: 'empty' });
  const [knownCids, setKnownCids] = useState<KnownCidEntry[]>(() => loadKnownCids());

  const goEmpty = useCallback(() => {
    setKnownCids(loadKnownCids());
    setView({ kind: 'empty' });
  }, []);

  const goAdd = useCallback(() => setView({ kind: 'add-input-cid' }), []);

  const goCheckMultisig = useCallback((cidNumber: string, cidFullName: string) => {
    // 中文注释:**不在此处** saveKnownCid。用户只是选了候选,链上未必存在,
    // 创建流程也可能立即失败。只有 goInstitutionDetail(链上确认存在)与
    // create_multisig.tsx 提案上链成功 callback 才能调 saveKnownCid。
    setView({ kind: 'check-multisig', cidNumber, cidFullName });
  }, []);

  const goInstitutionDetail = useCallback((cidNumber: string, cidFullName?: string) => {
    if (cidFullName) {
      saveKnownCid({ cidNumber, cidFullName });
      setKnownCids(loadKnownCids());
    }
    setView({ kind: 'institution-detail', cidNumber });
  }, []);

  const goAdminSetChange = useCallback(async (detail: InstitutionDetail) => {
    const accountRef = {
      cidNumber: detail.cidNumber,
      accountHex: detail.adminAccountHex,
      org: detail.org,
    };
    try {
      const activatedAdmins = await adminsChangeApi.getActivatedAdmins(detail.cidNumber, accountRef);
      setView({
        kind: 'admin-set-change',
        cidNumber: detail.cidNumber,
        cidFullName: detail.cidFullName,
        adminAccountHex: detail.adminAccountHex,
        org: detail.org,
        adminWallets: activatedAdmins.map((admin) => ({
          address: hexToSs58(admin.pubkeyHex),
          pubkeyHex: admin.pubkeyHex,
          name: '',
        })),
      });
    } catch (e) {
      window.alert(sanitizeError(e));
    }
  }, []);

  return (
    <div className="governance-section clearing-bank-section">
      {view.kind === 'empty' && (
        <EmptyView
          knownCids={knownCids}
          onAdd={goAdd}
          onOpen={(s) => goInstitutionDetail(s.cidNumber, s.cidFullName)}
        />
      )}

      {view.kind === 'add-input-cid' && (
        <ClearingBankAddPage
          onBack={goEmpty}
          onSelectCandidate={(c) => goCheckMultisig(c.cidNumber, c.cidFullName)}
          onSelectKnownCid={(s) => goCheckMultisig(s, '')}
        />
      )}

      {view.kind === 'check-multisig' && (
        <CheckMultisigView
          cidNumber={view.cidNumber}
          cidFullName={view.cidFullName}
          onBack={goEmpty}
          onExists={() => goInstitutionDetail(view.cidNumber, view.cidFullName)}
          onMissing={() => setView({ kind: 'create-multisig-institution', cidNumber: view.cidNumber })}
        />
      )}

      {view.kind === 'institution-detail' && (
        <ClearingBankInstitutionDetailPage
          cidNumber={view.cidNumber}
          onBack={goEmpty}
          onOpenOtherAccounts={(detail) =>
            setView({
              kind: 'other-accounts-list',
              cidNumber: view.cidNumber,
              otherAccounts: detail.otherAccounts,
            })
          }
          onOpenAdminList={(detail) =>
            setView({
              kind: 'admin-list',
              cidNumber: view.cidNumber,
              admins: detail.adminsSs58,
              threshold: detail.threshold,
              adminsLen: detail.adminsLen,
            })
          }
          onDeclareNode={(cidNumber, cidFullName) =>
            setView({ kind: 'declare-node', cidNumber, cidFullName })
          }
          onCreateAdminSetChange={goAdminSetChange}
        />
      )}

      {view.kind === 'admin-set-change' && (
        <AdminSetChangePage
          accountRef={{
            cidNumber: view.cidNumber,
            accountHex: view.adminAccountHex,
            org: view.org,
          }}
          cidFullName={view.cidFullName}
          adminWallets={view.adminWallets}
          onBack={() => setView({ kind: 'institution-detail', cidNumber: view.cidNumber })}
          onSuccess={() => setView({ kind: 'institution-detail', cidNumber: view.cidNumber })}
        />
      )}

      {view.kind === 'other-accounts-list' && (
        <OtherAccountsListPage
          cidNumber={view.cidNumber}
          otherAccounts={view.otherAccounts}
          onBack={() => setView({ kind: 'institution-detail', cidNumber: view.cidNumber })}
        />
      )}

      {view.kind === 'admin-list' && (
        <ClearingBankAdminListPage
          cidNumber={view.cidNumber}
          admins={view.admins}
          threshold={view.threshold}
          adminsLen={view.adminsLen}
          onBack={() => setView({ kind: 'institution-detail', cidNumber: view.cidNumber })}
        />
      )}

      {view.kind === 'create-multisig-institution' && (
        <CreateMultisigInstitutionPage
          cidNumber={view.cidNumber}
          coldWallets={[]}
          onBack={goEmpty}
          onSubmitted={(cidNumber, cidFullName) =>
            setView({ kind: 'wait-vote', cidNumber, cidFullName })
          }
        />
      )}

      {view.kind === 'wait-vote' && (
        <WaitVoteView
          cidNumber={view.cidNumber}
          cidFullName={view.cidFullName}
          onBack={goEmpty}
          onActivated={() =>
            setView({ kind: 'declare-node', cidNumber: view.cidNumber, cidFullName: view.cidFullName })
          }
        />
      )}

      {view.kind === 'declare-node' && (
        <ClearingBankDeclareNodePage
          cidNumber={view.cidNumber}
          cidFullName={view.cidFullName}
          onBack={goEmpty}
          onSuccess={() => goInstitutionDetail(view.cidNumber, view.cidFullName)}
        />
      )}
    </div>
  );
}

// ─── 子组件:空视图 + 已添加列表 ───
//
// 中文注释:挂载时自愈本地脏数据 ——— 对每个 knownCid 条目调
// fetchInstitutionDetail(cidNumber);链上 None 即从 localStorage 移除。
// 这样旧版本(2026-05-01 之前)误存的"选候选即落条目"的孤儿卡能自动消失。
//
// 链查失败(网络/节点未运行)时**保留**条目(避免误删合法记录)。
function EmptyView({
  knownCids: initialKnown,
  onAdd,
  onOpen,
}: {
  knownCids: KnownCidEntry[];
  onAdd: () => void;
  onOpen: (entry: KnownCidEntry) => void;
}) {
  const [knownCids, setKnownCids] = useState<KnownCidEntry[]>(initialKnown);

  useEffect(() => {
    let cancelled = false;
    const cleanup = async () => {
      const next: KnownCidEntry[] = [];
      for (const entry of initialKnown) {
        try {
          const detail = await organizationManageApi.fetchInstitutionDetail(entry.cidNumber);
          if (detail !== null) {
            next.push(entry); // 链上有(Pending / Active 都保留)
          } else {
            removeKnownCid(entry.cidNumber); // 链上明确不存在 → 移除孤儿
          }
        } catch {
          next.push(entry); // 链查失败保守保留
        }
      }
      if (!cancelled) setKnownCids(next);
    };
    cleanup();
    return () => {
      cancelled = true;
    };
  }, [initialKnown]);

  return (
    <>
      <div className="admin-list-header">
        <h2>清算行</h2>
        <button className="primary-button" onClick={onAdd}>+ 添加清算行</button>
      </div>
      {knownCids.length > 0 && (
        <div className="admin-grid">
          {knownCids.map((s) => (
            <div
              key={s.cidNumber}
              className="metric-card admin-card"
              role="button"
              tabIndex={0}
              onClick={() => onOpen(s)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') onOpen(s);
              }}
            >
              <span className="admin-card-index">→</span>
              <div>
                <strong>{s.cidFullName}</strong>
                <code className="admin-card-address" style={{ marginLeft: 8 }}>{s.cidNumber}</code>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}

// ─── 子组件:链上查机构是否已存在,已存在跳详情,不存在跳创建 ───
function CheckMultisigView({
  cidNumber,
  cidFullName,
  onBack,
  onExists,
  onMissing,
}: {
  cidNumber: string;
  cidFullName: string;
  onBack: () => void;
  onExists: () => void;
  onMissing: () => void;
}) {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    organizationManageApi
      .fetchInstitutionDetail(cidNumber)
      .then((detail) => {
        if (cancelled) return;
        if (detail) onExists();
        else onMissing();
      })
      .catch((e) => {
        if (!cancelled) setError(sanitizeError(e));
      });
    return () => {
      cancelled = true;
    };
  }, [cidNumber, onExists, onMissing]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      {error ? (
        <div className="error">{error}</div>
      ) : (
        <p>正在判定 {cidFullName || cidNumber} 链上多签状态…</p>
      )}
    </>
  );
}

// ─── 子组件:等其他管理员投票通过 ───
function WaitVoteView({
  cidNumber,
  cidFullName,
  onBack,
  onActivated,
}: {
  cidNumber: string;
  cidFullName: string;
  onBack: () => void;
  onActivated: () => void;
}) {
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let cancelled = false;
    const id = setInterval(async () => {
      if (cancelled) return;
      try {
        const detail = await organizationManageApi.fetchInstitutionDetail(cidNumber);
        if (detail && detail.status === 'Active') {
          onActivated();
        }
      } catch {
        /* 容忍轮询失败 */
      }
      setTick((t) => t + 1);
    }, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [cidNumber, onActivated]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>等待管理员投票通过</h2>
      </div>
      <p>
        机构 {cidFullName} ({cidNumber}) 的创建提案已发起,正在等待其他管理员
        通过冷钱包投赞成达阈值。投票通过后链上 Institutions[cid_number].status 由
        Pending 变 Active,本页自动跳转到"声明本机为清算行节点"。
      </p>
      <p className="muted">每 5 秒自动检查 1 次链上状态(已检查 {tick} 次)…</p>
    </>
  );
}

// AccountWithBalance 来自 types(被 view kind 中的字段引用,确保 import 有效)。
export type _Reexport = AccountWithBalance | EligibleClearingBankCandidate | InstitutionDetail;
