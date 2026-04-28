// 节点 UI 顶级 tab 栏：首页 / 挖矿 / 国储会 / 省储会 / 省储行 / 清算行 / 白皮书 / 公民宪法 / 设置。
// 2026-04-24 重构：
//   - 删除"治理"顶级 tab（其子页 国储会/省储会/省储行 提升为顶级 tab；提案子 tab 下线；开发升级迁至设置页）。
//   - 删除"网络"顶级 tab（内容 3×2 卡片布局下沉到挖矿页"资源监控"与"出块记录"之间）。
// 2026-04-27(ADR-007 Step 2 阶段 C):新增"清算行"顶级 tab,介于"省储行"与"白皮书"之间。
import { useState } from 'react';
import { NrcSection } from './governance/NrcSection';
import { PrcSection } from './governance/PrcSection';
import { PrbSection } from './governance/PrbSection';
import { ClearingBankSection } from './clearing-bank/ClearingBankSection';
import { HomeNodeSection } from './home/home-node';
import { TransactionPanel } from './transaction/TransactionPanel';
import { MiningDashboardSection } from './mining/mining-dashboard';
import { OtherTabsSection } from './other/other-tabs';
import { SettingsSection } from './settings/settings-panel';

type TabKey =
  | 'home'
  | 'mining'
  | 'nrc'
  | 'prc'
  | 'prb'
  | 'clearing-bank'
  | 'whitepaper'
  | 'constitution'
  | 'settings';

export default function App() {
  const [tab, setTab] = useState<TabKey>('home');

  return (
    <div className="page">
      <nav className="top-nav">
        <button className={tab === 'home' ? 'active' : ''} onClick={() => setTab('home')}>首页</button>
        <button className={tab === 'mining' ? 'active' : ''} onClick={() => setTab('mining')}>挖矿</button>
        <button className={tab === 'nrc' ? 'active' : ''} onClick={() => setTab('nrc')}>国储会</button>
        <button className={tab === 'prc' ? 'active' : ''} onClick={() => setTab('prc')}>省储会</button>
        <button className={tab === 'prb' ? 'active' : ''} onClick={() => setTab('prb')}>省储行</button>
        <button className={tab === 'clearing-bank' ? 'active' : ''} onClick={() => setTab('clearing-bank')}>清算行</button>
        <button className={tab === 'whitepaper' ? 'active' : ''} onClick={() => setTab('whitepaper')}>白皮书</button>
        <button className={tab === 'constitution' ? 'active' : ''} onClick={() => setTab('constitution')}>公民宪法</button>
        <button className={tab === 'settings' ? 'active' : ''} onClick={() => setTab('settings')}>设置</button>
      </nav>

      {tab === 'home' ? (
        <div className="home-dual-panel">
          <main className="app">
            <section className="content">
              <HomeNodeSection />
            </section>
          </main>
          <aside className="app">
            <section className="content">
              <TransactionPanel />
            </section>
          </aside>
        </div>
      ) : (
        <main className="app">
          <section className="content">
            {tab === 'settings' ? (
              <SettingsSection />
            ) : null}

            {tab === 'mining' ? (
              <MiningDashboardSection />
            ) : null}

            {tab === 'nrc' ? (
              <NrcSection />
            ) : null}

            {tab === 'prc' ? (
              <PrcSection />
            ) : null}

            {tab === 'prb' ? (
              <PrbSection />
            ) : null}

            {tab === 'clearing-bank' ? (
              <ClearingBankSection />
            ) : null}

            {tab === 'whitepaper' ? (
              <OtherTabsSection activeKey="whitepaper" />
            ) : null}

            {tab === 'constitution' ? (
              <OtherTabsSection activeKey="constitution" />
            ) : null}
          </section>
        </main>
      )}
    </div>
  );
}
