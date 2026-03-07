import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api';
import { WalletSection } from '../fee-address/WalletSection';
import { NodeKeySection } from '../node-key/NodeKeySection';
import type { BootnodeKey, RewardWallet } from '../../types';

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

  const loadSettings = useCallback(async () => {
    const [w, k] = await Promise.allSettled([
      api.getRewardWallet(),
      api.getBootnodeKey(),
    ]);
    if (w.status === 'fulfilled') setWallet(w.value);
    if (k.status === 'fulfilled') setNodeKey(k.value);
  }, []);

  useEffect(() => {
    void loadSettings().catch(() => undefined);
  }, [loadSettings]);

  return (
    <>
      <WalletSection wallet={wallet} onUpdated={setWallet} disabled={disabled} />
      <NodeKeySection
        nodeKey={nodeKey}
        onUpdated={setNodeKey}
        onApplied={() => {
          void loadSettings();
        }}
        disabled={disabled}
      />
    </>
  );
}
