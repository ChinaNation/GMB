import { useState } from 'react';
import { GovernanceSection } from './governance/GovernanceSection';
import { HomeNodeSection } from './home/home-node';
import { TransactionPanel } from './transaction/TransactionPanel';
import { MiningDashboardSection } from './mining/mining-dashboard';
import { NetworkOverviewSection } from './network/network-overview';
import { OtherTabsSection } from './other/other-tabs';
import { SettingsSection } from './settings/settings-panel';

type TabKey =
  | 'home'
  | 'mining'
  | 'governance'
  | 'network'
  | 'whitepaper'
  | 'constitution'
  | 'settings';

export default function App() {
  const [tab, setTab] = useState<TabKey>('home');
  const [nodeActionBusy, setNodeActionBusy] = useState(false);

  return (
    <div className="page">
      <nav className="top-nav">
        <button className={tab === 'home' ? 'active' : ''} onClick={() => setTab('home')}>首页</button>
        <button className={tab === 'mining' ? 'active' : ''} onClick={() => setTab('mining')}>挖矿</button>
        <button className={tab === 'governance' ? 'active' : ''} onClick={() => setTab('governance')}>治理</button>
        <button className={tab === 'network' ? 'active' : ''} onClick={() => setTab('network')}>网络</button>
        <button className={tab === 'whitepaper' ? 'active' : ''} onClick={() => setTab('whitepaper')}>白皮书</button>
        <button className={tab === 'constitution' ? 'active' : ''} onClick={() => setTab('constitution')}>公民宪法</button>
        <button className={tab === 'settings' ? 'active' : ''} onClick={() => setTab('settings')}>设置</button>
      </nav>

      {tab === 'home' ? (
        <div className="home-dual-panel">
          <main className="app">
            <section className="content">
              <HomeNodeSection onNodeActionBusyChange={setNodeActionBusy} />
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
              <SettingsSection disabled={nodeActionBusy} />
            ) : null}

            {tab === 'mining' ? (
              <MiningDashboardSection />
            ) : null}

            {tab === 'governance' ? (
              <GovernanceSection />
            ) : null}

            {tab === 'network' ? (
              <NetworkOverviewSection />
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
