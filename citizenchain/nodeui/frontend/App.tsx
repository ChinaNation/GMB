import { useState } from 'react';
import { HomeNodeSection } from './home/home-node';
import { MiningDashboardSection } from './mining/mining-dashboard';
import { NetworkOverviewSection } from './network/network-overview';
import { OtherTabsSection } from './other/other-tabs';
import { SettingsSection } from './settings/settings-panel';

type TabKey =
  | 'home'
  | 'mining'
  | 'network'
  | 'whitepaper'
  | 'party'
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
        <button className={tab === 'network' ? 'active' : ''} onClick={() => setTab('network')}>网络</button>
        <button className={tab === 'whitepaper' ? 'active' : ''} onClick={() => setTab('whitepaper')}>白皮书</button>
        <button className={tab === 'party' ? 'active' : ''} onClick={() => setTab('party')}>公民党</button>
        <button className={tab === 'constitution' ? 'active' : ''} onClick={() => setTab('constitution')}>公民宪法</button>
        <button className={tab === 'settings' ? 'active' : ''} onClick={() => setTab('settings')}>设置</button>
      </nav>

      <main className="app">
        <section className="content">
          {tab === 'home' ? (
            <HomeNodeSection onNodeActionBusyChange={setNodeActionBusy} />
          ) : null}

          {tab === 'settings' ? (
            <SettingsSection disabled={nodeActionBusy} />
          ) : null}

          {tab === 'mining' ? (
            <MiningDashboardSection />
          ) : null}

          {tab === 'network' ? (
            <NetworkOverviewSection />
          ) : null}

          {tab === 'whitepaper' ? (
            <OtherTabsSection activeKey="whitepaper" />
          ) : null}

          {tab === 'party' ? (
            <OtherTabsSection activeKey="party" />
          ) : null}

          {tab === 'constitution' ? (
            <OtherTabsSection activeKey="constitution" />
          ) : null}
        </section>
      </main>
    </div>
  );
}
