import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api';
import { WalletSection } from '../fee-address/WalletSection';
import { NodeKeySection } from '../node-key/NodeKeySection';
import { DeveloperUpgradePage } from '../developer-upgrade';
import type { BootnodeKey, ChainStatus, RewardWallet } from '../../types';

type Props = {
  disabled: boolean;
};

export function SettingsSection({ disabled }: Props) {
  const [wallet, setWallet] = useState<RewardWallet>({ address: null });
  const [nodeKey, setNodeKey] = useState<BootnodeKey>({
    nodeKey: null,
    peerId: null,
    institutionName: null,
  });
  const [chainStatus, setChainStatus] = useState<ChainStatus | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);

  const loadSettings = useCallback(async () => {
    const [w, k, c, a] = await Promise.allSettled([
      api.getRewardWallet(),
      api.getBootnodeKey(),
      api.getChainStatus(),
      api.hasAnyActivatedAdmin(),
    ]);
    if (w.status === 'fulfilled') setWallet(w.value);
    if (k.status === 'fulfilled') setNodeKey(k.value);
    if (c.status === 'fulfilled') setChainStatus(c.value);
    if (a.status === 'fulfilled') setIsAdmin(a.value);
  }, []);

  useEffect(() => {
    void loadSettings().catch(() => undefined);
  }, [loadSettings]);

  return (
    <>
      <WalletSection wallet={wallet} onUpdated={setWallet} disabled={disabled} />
      {isAdmin && (
        <NodeKeySection
          nodeKey={nodeKey}
          onUpdated={setNodeKey}
          onApplied={() => {
            void loadSettings();
          }}
          disabled={disabled}
        />
      )}
      {isAdmin && <DeveloperUpgradePage />}
      {chainStatus && (
        <div className="settings-version-section">
          <div className="settings-version-row">
            <span className="settings-version-label">节点程序版本</span>
            <span className="settings-version-value">{chainStatus.nodeVersion}</span>
          </div>
          <div className="settings-version-row">
            <span className="settings-version-label">Runtime 版本</span>
            <span className="settings-version-value">
              {chainStatus.specVersion != null ? `spec ${chainStatus.specVersion}` : '节点未运行'}
            </span>
          </div>
        </div>
      )}
    </>
  );
}
