import { useCallback, useEffect, useState } from 'react';
import { adminsChangeApi } from '../../admins/admin-management/api';
import { homeNodeApi } from '../../home/api';
import { settingsApi } from '../api';
import { WalletSection } from '../fee-address/WalletSection';
import { NodeModeSection } from '../NodeModeSection';
import { NodeKeySection } from '../NodeKeySection';
import { OnChinaPlatformSection } from '../OnChinaPlatformSection';
import type { ChainStatus } from '../../home/types';
import type {
  BootnodeKey,
  DesktopUpdateInfo,
  NodeModeState,
  OnChinaPlatformState,
  RewardWallet,
} from '../types';

type SettingsSectionProps = {
  desktopUpdateInfo: DesktopUpdateInfo;
  onInstallDesktopUpdate: () => Promise<void>;
};

export function SettingsSection({
  desktopUpdateInfo,
  onInstallDesktopUpdate,
}: SettingsSectionProps) {
  const [nodeMode, setNodeMode] = useState<NodeModeState | null>(null);
  const [onChinaPlatform, setOnChinaPlatform] =
    useState<OnChinaPlatformState | null>(null);
  const [wallet, setWallet] = useState<RewardWallet>({ address: null });
  const [nodeKey, setNodeKey] = useState<BootnodeKey>({
    nodeKey: null,
    peerId: null,
    authorityNodeLabel: null,
  });
  const [chainStatus, setChainStatus] = useState<ChainStatus | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);

  const loadSettings = useCallback(async () => {
    const [m, p, w, k, c, a] = await Promise.allSettled([
      settingsApi.getNodeMode(),
      settingsApi.getOnChinaPlatform(),
      settingsApi.getRewardWallet(),
      settingsApi.getBootnodeKey(),
      homeNodeApi.getChainStatus(),
      adminsChangeApi.hasAnyActivatedAdmin(),
    ]);
    if (m.status === 'fulfilled') setNodeMode(m.value);
    if (p.status === 'fulfilled') setOnChinaPlatform(p.value);
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
      <NodeModeSection nodeMode={nodeMode} onUpdated={setNodeMode} />
      <OnChinaPlatformSection
        platform={onChinaPlatform}
        onUpdated={setOnChinaPlatform}
      />
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
