// 清算行 tab 主入口。8 视图状态机驱动添加/管理流程。
//
// 重构(2026-05-01)后状态机:
//   empty                        列表 + ＋添加清算行
//   add-input-sfid               输入 sfid_id 自动 debounce 搜索 SFID 候选(不再带"查询"按钮)
//   check-multisig               链上查 Institutions[sfid_id]
//                                  ├─ 已存在 → institution-detail
//                                  └─ 不存在 → create-multisig-institution
//   institution-detail           机构详情 = 卡片栅格 + 折叠卡片(其他账户 / 管理员) + 节点信息
//                                顶部按钮:未声明节点 → declare-node;已声明 → 内联展示节点信息
//   other-accounts-list          子页:其他账户列表
//   admin-list                   子页:管理员列表
//   create-multisig-institution  创建机构多签 propose_create_institution(冷钱包签 + 提交)
//   wait-vote                    等待管理员投票通过(轮询 Institutions[sfid_id].status === 'Active')
//   declare-node                 多签 Active 但本机未声明节点 → 填 RPC + 自测 + 签名声明
//
// 已下架(2026-05-01):
//   - register-sfid info 终态(SFID 端没找到候选时改为内联红条)
//   - propose-create info 终态("去 SFID 后台"长说明,改为直接进 create-multisig-institution)
//   - 老 detail 视图(只展示节点信息),并入 institution-detail 内联节点信息卡

import { useEffect, useState, useCallback } from 'react';
import { sanitizeError } from '../core/tauri';
import { offchainApi } from './api';
import type {
  AccountWithBalance,
  ClearingBankView,
  EligibleClearingBankCandidate,
  InstitutionDetail,
} from './types';
import { ClearingBankAddPage } from './duoqian-manage/add-candidate';
import { ClearingBankDeclareNodePage } from './offchain-transaction/node-register';
import { ClearingBankInstitutionDetailPage } from './duoqian-manage/institution-detail';
import { CreateMultisigInstitutionPage } from './duoqian-manage/create-multisig';
import { OtherAccountsListPage } from './duoqian-manage/other-accounts';
import { ClearingBankAdminListPage } from './settlement/admin-unlock';
import './styles.css';

const STORAGE_KEY = 'gmb-clearing-bank-known-sfids';

type KnownSfidEntry = { sfidId: string; institutionName: string };

function loadKnownSfids(): KnownSfidEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (e): e is KnownSfidEntry =>
        typeof e === 'object' && e !== null
        && typeof (e as { sfidId?: unknown }).sfidId === 'string'
        && typeof (e as { institutionName?: unknown }).institutionName === 'string',
    );
  } catch {
    return [];
  }
}

/// 把已确认存在(链上有 Institutions[sfid_id] 或刚提交 propose_create_institution
/// 成功)的机构入条目加入本机已添加列表。**不要**在用户只是选了候选时调本函数,
/// 否则若链上没有 + 创建流程失败,EmptyView 会显示孤儿卡。
export function saveKnownSfid(entry: KnownSfidEntry) {
  const list = loadKnownSfids().filter((e) => e.sfidId !== entry.sfidId);
  list.unshift(entry);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list.slice(0, 50)));
}

/// 从本机已添加列表中删除指定 sfid_id 条目(链上判定为 None 或用户主动移除时调)。
function removeKnownSfid(sfidId: string) {
  const list = loadKnownSfids().filter((e) => e.sfidId !== sfidId);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list.slice(0, 50)));
}

export function ClearingBankSection() {
  const [view, setView] = useState<ClearingBankView>({ kind: 'empty' });
  const [knownSfids, setKnownSfids] = useState<KnownSfidEntry[]>(() => loadKnownSfids());

  const goEmpty = useCallback(() => {
    setKnownSfids(loadKnownSfids());
    setView({ kind: 'empty' });
  }, []);

  const goAdd = useCallback(() => setView({ kind: 'add-input-sfid' }), []);

  const goCheckMultisig = useCallback((sfidId: string, institutionName: string) => {
    // 中文注释:**不在此处** saveKnownSfid。用户只是选了候选,链上未必存在,
    // 创建流程也可能立即失败。只有 goInstitutionDetail(链上确认存在)与
    // create_multisig.tsx 提案上链成功 callback 才能调 saveKnownSfid。
    setView({ kind: 'check-multisig', sfidId, institutionName });
  }, []);

  const goInstitutionDetail = useCallback((sfidId: string, institutionName?: string) => {
    if (institutionName) {
      saveKnownSfid({ sfidId, institutionName });
      setKnownSfids(loadKnownSfids());
    }
    setView({ kind: 'institution-detail', sfidId });
  }, []);

  return (
    <div className="governance-section clearing-bank-section">
      {view.kind === 'empty' && (
        <EmptyView
          knownSfids={knownSfids}
          onAdd={goAdd}
          onOpen={(s) => goInstitutionDetail(s.sfidId, s.institutionName)}
        />
      )}

      {view.kind === 'add-input-sfid' && (
        <ClearingBankAddPage
          onBack={goEmpty}
          onSelectCandidate={(c) => goCheckMultisig(c.sfidId, c.institutionName)}
          onSelectKnownSfid={(s) => goCheckMultisig(s, '')}
        />
      )}

      {view.kind === 'check-multisig' && (
        <CheckMultisigView
          sfidId={view.sfidId}
          institutionName={view.institutionName}
          onBack={goEmpty}
          onExists={() => goInstitutionDetail(view.sfidId, view.institutionName)}
          onMissing={() => setView({ kind: 'create-multisig-institution', sfidId: view.sfidId })}
        />
      )}

      {view.kind === 'institution-detail' && (
        <ClearingBankInstitutionDetailPage
          sfidId={view.sfidId}
          onBack={goEmpty}
          onOpenOtherAccounts={(detail) =>
            setView({
              kind: 'other-accounts-list',
              sfidId: view.sfidId,
              otherAccounts: detail.otherAccounts,
            })
          }
          onOpenAdminList={(detail) =>
            setView({
              kind: 'admin-list',
              sfidId: view.sfidId,
              admins: detail.duoqianAdminsSs58,
              threshold: detail.threshold,
              adminCount: detail.adminCount,
            })
          }
          onDeclareNode={(sfidId, institutionName) =>
            setView({ kind: 'declare-node', sfidId, institutionName })
          }
        />
      )}

      {view.kind === 'other-accounts-list' && (
        <OtherAccountsListPage
          sfidId={view.sfidId}
          otherAccounts={view.otherAccounts}
          onBack={() => setView({ kind: 'institution-detail', sfidId: view.sfidId })}
        />
      )}

      {view.kind === 'admin-list' && (
        <ClearingBankAdminListPage
          sfidId={view.sfidId}
          admins={view.admins}
          threshold={view.threshold}
          adminCount={view.adminCount}
          onBack={() => setView({ kind: 'institution-detail', sfidId: view.sfidId })}
        />
      )}

      {view.kind === 'create-multisig-institution' && (
        <CreateMultisigInstitutionPage
          sfidId={view.sfidId}
          coldWallets={[]}
          onBack={goEmpty}
          onSubmitted={(sfidId, institutionName) =>
            setView({ kind: 'wait-vote', sfidId, institutionName })
          }
        />
      )}

      {view.kind === 'wait-vote' && (
        <WaitVoteView
          sfidId={view.sfidId}
          institutionName={view.institutionName}
          onBack={goEmpty}
          onActivated={() =>
            setView({ kind: 'declare-node', sfidId: view.sfidId, institutionName: view.institutionName })
          }
        />
      )}

      {view.kind === 'declare-node' && (
        <ClearingBankDeclareNodePage
          sfidId={view.sfidId}
          institutionName={view.institutionName}
          onBack={goEmpty}
          onSuccess={() => goInstitutionDetail(view.sfidId, view.institutionName)}
        />
      )}
    </div>
  );
}

// ─── 子组件:空视图 + 已添加列表 ───
//
// 中文注释:挂载时自愈本地脏数据 ——— 对每个 knownSfid 条目调
// fetchInstitutionDetail(sfidId);链上 None 即从 localStorage 移除。
// 这样旧版本(2026-05-01 之前)误存的"选候选即落条目"的孤儿卡能自动消失。
//
// 链查失败(网络/节点未运行)时**保留**条目(避免误删合法记录)。
function EmptyView({
  knownSfids: initialKnown,
  onAdd,
  onOpen,
}: {
  knownSfids: KnownSfidEntry[];
  onAdd: () => void;
  onOpen: (entry: KnownSfidEntry) => void;
}) {
  const [knownSfids, setKnownSfids] = useState<KnownSfidEntry[]>(initialKnown);

  useEffect(() => {
    let cancelled = false;
    const cleanup = async () => {
      const next: KnownSfidEntry[] = [];
      for (const entry of initialKnown) {
        try {
          const detail = await offchainApi.fetchInstitutionDetail(entry.sfidId);
          if (detail !== null) {
            next.push(entry); // 链上有(Pending / Active 都保留)
          } else {
            removeKnownSfid(entry.sfidId); // 链上明确不存在 → 移除孤儿
          }
        } catch {
          next.push(entry); // 链查失败保守保留
        }
      }
      if (!cancelled) setKnownSfids(next);
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
      {knownSfids.length > 0 && (
        <div className="admin-grid">
          {knownSfids.map((s) => (
            <div
              key={s.sfidId}
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
                <strong>{s.institutionName}</strong>
                <code className="admin-card-address" style={{ marginLeft: 8 }}>{s.sfidId}</code>
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
  sfidId,
  institutionName,
  onBack,
  onExists,
  onMissing,
}: {
  sfidId: string;
  institutionName: string;
  onBack: () => void;
  onExists: () => void;
  onMissing: () => void;
}) {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    offchainApi
      .fetchInstitutionDetail(sfidId)
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
  }, [sfidId, onExists, onMissing]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      {error ? (
        <div className="error">{error}</div>
      ) : (
        <p>正在判定 {institutionName || sfidId} 链上多签状态…</p>
      )}
    </>
  );
}

// ─── 子组件:等其他管理员投票通过 ───
function WaitVoteView({
  sfidId,
  institutionName,
  onBack,
  onActivated,
}: {
  sfidId: string;
  institutionName: string;
  onBack: () => void;
  onActivated: () => void;
}) {
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let cancelled = false;
    const id = setInterval(async () => {
      if (cancelled) return;
      try {
        const detail = await offchainApi.fetchInstitutionDetail(sfidId);
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
  }, [sfidId, onActivated]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>等待管理员投票通过</h2>
      </div>
      <p>
        机构 {institutionName} ({sfidId}) 的创建提案已发起,正在等待其他管理员
        通过冷钱包投赞成达阈值。投票通过后链上 Institutions[sfid_id].status 由
        Pending 变 Active,本页自动跳转到"声明本机为清算行节点"。
      </p>
      <p className="muted">每 5 秒自动检查 1 次链上状态(已检查 {tick} 次)…</p>
    </>
  );
}

// AccountWithBalance 来自 types(被 view kind 中的字段引用,确保 import 有效)。
export type _Reexport = AccountWithBalance | EligibleClearingBankCandidate | InstitutionDetail;
