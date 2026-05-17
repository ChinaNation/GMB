import { useCallback, useEffect, useState } from 'react';
import { adminsChangeApi } from '../../governance/admins_change/api';
import { homeNodeApi } from '../../home/api';
import { settingsApi } from '../api';
import { WalletSection } from '../fee-address/WalletSection';
import { NodeKeySection } from '../node-key/NodeKeySection';
import type { ChainStatus } from '../../home/types';
import type { BootnodeKey, DesktopUpdateInfo, RewardWallet } from '../types';

type SettingsSectionProps = {
  desktopUpdateInfo: DesktopUpdateInfo;
  onInstallDesktopUpdate: () => Promise<void>;
};

export function SettingsSection({
  desktopUpdateInfo,
  onInstallDesktopUpdate,
}: SettingsSectionProps) {
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
      settingsApi.getRewardWallet(),
      settingsApi.getBootnodeKey(),
      homeNodeApi.getChainStatus(),
      adminsChangeApi.hasAnyActivatedAdmin(),
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
      <WalletSection wallet={wallet} onUpdated={setWallet} />
      {isAdmin && (
        <NodeKeySection
          nodeKey={nodeKey}
          onUpdated={setNodeKey}
          onApplied={() => {
            void loadSettings();
          }}
        />
      )}
      {chainStatus && (
        <div className="settings-version-section">
          <div className="settings-version-row">
            <span className="settings-version-label">节点程序版本</span>
            <span className="settings-version-value-wrap">
              {desktopUpdateInfo.status === 'available' || desktopUpdateInfo.status === 'installing' ? (
                <button
                  type="button"
                  className="settings-update-button"
                  disabled={desktopUpdateInfo.status === 'installing'}
                  title={
                    desktopUpdateInfo.latestVersion
                      ? `可更新到 ${desktopUpdateInfo.latestVersion}`
                      : undefined
                  }
                  onClick={() => {
                    void onInstallDesktopUpdate();
                  }}
                >
                  {desktopUpdateInfo.status === 'installing' ? '更新中' : '更新'}
                </button>
              ) : null}
              <span className="settings-version-value">{chainStatus.nodeVersion}</span>
            </span>
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
