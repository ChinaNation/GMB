// 清算行 tab 主入口。8 视图状态机驱动添加/管理流程。
//
// empty            列表 + 顶部"添加清算行"按钮
// add-input-sfid   输入 sfid_id 或搜索机构名,选中候选后进入 check-status
// check-status     综合判定 SFID 状态 + 链上多签 + 节点声明,自动跳转
// register-sfid    SFID 端未注册 → 提示去 SFID 系统注册(终止)
// propose-create   未创建多签账户 → 复用治理 propose_create 流程
// wait-vote        提案已发起 → 轮询投票结果
// declare-node     多签 Active 但未声明节点 → 填 RPC 信息 + 自测 + 签名提交
// detail           已声明节点 → 详情页(节点信息 + 管理员列表[解密] + 提案列表)
//
// 已添加的清算行(本机已用过的 sfid_id)缓存在 localStorage,empty 视图列出。

import { useEffect, useState, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import type {
  ClearingBankNodeOnChainInfo,
  ClearingBankView,
  EligibleClearingBankCandidate,
} from './clearing-bank-types';
import { ClearingBankAddPage } from './ClearingBankAddPage';
import { ClearingBankDeclareNodePage } from './ClearingBankDeclareNodePage';
import { ClearingBankDetailPage } from './ClearingBankDetailPage';

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

function saveKnownSfid(entry: KnownSfidEntry) {
  const list = loadKnownSfids().filter(e => e.sfidId !== entry.sfidId);
  list.unshift(entry);
  // 上限 50 条
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

  const goCheckStatus = useCallback((sfidId: string) => {
    setView({ kind: 'check-status', sfidId });
  }, []);

  const goDetail = useCallback((sfidId: string, institutionName?: string) => {
    if (institutionName) {
      saveKnownSfid({ sfidId, institutionName });
    }
    setKnownSfids(loadKnownSfids());
    setView({ kind: 'detail', sfidId });
  }, []);

  return (
    <div className="governance-section clearing-bank-section">
      {view.kind === 'empty' && (
        <EmptyView
          knownSfids={knownSfids}
          onAdd={goAdd}
          onOpen={(s) => goDetail(s.sfidId, s.institutionName)}
        />
      )}

      {view.kind === 'add-input-sfid' && (
        <ClearingBankAddPage
          onBack={goEmpty}
          onSelectCandidate={(c) => {
            // 缓存机构名,后续 detail 页面就有显示文案
            saveKnownSfid({ sfidId: c.sfidId, institutionName: c.institutionName });
            goCheckStatus(c.sfidId);
          }}
          onSelectKnownSfid={(s) => goCheckStatus(s)}
        />
      )}

      {view.kind === 'check-status' && (
        <CheckStatusView
          sfidId={view.sfidId}
          onBack={goEmpty}
          onNext={(next) => setView(next)}
        />
      )}

      {view.kind === 'register-sfid' && (
        <InfoView
          title="该机构尚未在 SFID 注册"
          message={`机构身份码 ${view.candidate.sfidId} 尚未在 SFID 系统创建。请先去 SFID 系统(crcfrcn.com 后台)创建机构后再回到此处声明清算行节点。`}
          onBack={goEmpty}
        />
      )}

      {view.kind === 'propose-create' && (
        <InfoView
          title="多签账户尚未创建"
          message={`机构 ${view.candidate.institutionName} (${view.candidate.sfidId}) 在链上尚未创建多签账户。请前往 SFID 后台执行"激活机构"流程,流程触发链上 register_sfid_institution(写入元数据)+ propose_create(投票创建多签)。投票通过后回到本页继续声明清算行节点。`}
          onBack={goEmpty}
        />
      )}

      {view.kind === 'wait-vote' && (
        <WaitVoteView
          sfidId={view.sfidId}
          institutionName={view.institutionName}
          onBack={goEmpty}
          onActivated={() => setView({ kind: 'declare-node', sfidId: view.sfidId, institutionName: view.institutionName })}
        />
      )}

      {view.kind === 'declare-node' && (
        <ClearingBankDeclareNodePage
          sfidId={view.sfidId}
          institutionName={view.institutionName}
          onBack={goEmpty}
          onSuccess={() => goDetail(view.sfidId, view.institutionName)}
        />
      )}

      {view.kind === 'detail' && (
        <ClearingBankDetailPage
          sfidId={view.sfidId}
          onBack={goEmpty}
          onUnregistered={goEmpty}
        />
      )}
    </div>
  );
}

// ─── 子组件:空视图 + 已添加列表 ───
function EmptyView({
  knownSfids,
  onAdd,
  onOpen,
}: {
  knownSfids: KnownSfidEntry[];
  onAdd: () => void;
  onOpen: (entry: KnownSfidEntry) => void;
}) {
  return (
    <>
      <div className="admin-list-header">
        <h2>清算行</h2>
        <button className="primary-button" onClick={onAdd}>+ 添加清算行</button>
      </div>
      <p className="muted">
        清算行 = 私法人股份公司(SFR-JOINT_STOCK)及其下属非法人(FFR-parent)。
        在 SFID 系统创建机构 + 链上激活后,在本页声明对外提供清算服务的全节点
        身份(PeerId + RPC 域名),即加入清算网络。
      </p>
      {knownSfids.length === 0 ? (
        <p className="no-data">本机暂未管理任何清算行。点"+ 添加清算行"开始。</p>
      ) : (
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

// ─── 子组件:综合判定状态 ───
function CheckStatusView({
  sfidId,
  onBack,
  onNext,
}: {
  sfidId: string;
  onBack: () => void;
  onNext: (next: ClearingBankView) => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<ClearingBankNodeOnChainInfo | null>(null);
  const [searchingCandidate, setSearchingCandidate] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setSearchingCandidate(true);
    Promise.all([
      api.queryClearingBankNodeInfo(sfidId).catch(() => null),
      api.searchEligibleClearingBanks(sfidId, 1).catch(() => [] as EligibleClearingBankCandidate[]),
    ])
      .then(([nodeInfo, candidates]) => {
        if (cancelled) return;
        setInfo(nodeInfo);
        setSearchingCandidate(false);

        if (nodeInfo) {
          // 已声明节点,直接进详情
          onNext({ kind: 'detail', sfidId });
          return;
        }
        const cand = candidates.find(c => c.sfidId === sfidId);
        if (!cand) {
          // SFID 未注册或非清算行候选
          onNext({
            kind: 'register-sfid',
            candidate: {
              sfidId,
              institutionName: sfidId,
              a3: '',
              province: '',
              city: '',
              mainChainStatus: 'Inactive',
            },
          });
          return;
        }
        if (cand.mainChainStatus === 'Inactive' || cand.mainChainStatus === 'Pending') {
          // 多签未创建/待激活
          onNext({ kind: 'propose-create', candidate: cand });
          return;
        }
        if (cand.mainChainStatus === 'Failed') {
          setError('链上多签账户激活失败,请回 SFID 后台重新激活');
          return;
        }
        // Registered = 多签 Active 但未声明节点
        onNext({ kind: 'declare-node', sfidId, institutionName: cand.institutionName });
      })
      .catch((e) => {
        if (!cancelled) setError(sanitizeError(e));
      });
    return () => { cancelled = true; };
  }, [sfidId, onNext]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      {error ? (
        <div className="error">{error}</div>
      ) : (
        <p>正在判定 {sfidId} 状态…{info ? '(链上已声明节点)' : (searchingCandidate ? '' : '')}</p>
      )}
    </>
  );
}

function InfoView({ title, message, onBack }: { title: string; message: string; onBack: () => void }) {
  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>{title}</h2>
      </div>
      <p>{message}</p>
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
        const cand = await api.searchEligibleClearingBanks(sfidId, 1);
        const found = cand.find(c => c.sfidId === sfidId);
        if (found && found.mainChainStatus === 'Registered') {
          onActivated();
        }
      } catch (_) { /* 容忍轮询失败 */ }
      setTick((t) => t + 1);
    }, 5000);
    return () => { cancelled = true; clearInterval(id); };
  }, [sfidId, onActivated]);

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>
      <div className="admin-list-header">
        <h2>等待管理员投票通过</h2>
      </div>
      <p>
        机构 {institutionName} ({sfidId}) 的多签账户创建提案已发起,正在等待 13/19
        国储会 ∨ 6/11 省储会等达成阈值。
      </p>
      <p className="muted">每 5 秒自动检查 1 次链上状态(已检查 {tick} 次)…</p>
    </>
  );
}
