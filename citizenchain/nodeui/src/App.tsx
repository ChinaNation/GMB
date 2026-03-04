import { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Alert, Button, Card, Input, Layout, Modal, Space, Tag, Typography, message } from 'antd';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady, decodeAddress } from '@polkadot/util-crypto';
import { connectNode } from './services/rpc/polkadot';
import { DEFAULT_NODE_ENDPOINT } from './constants/node';

const { Content } = Layout;

type NodeStatus = {
  running: boolean;
  state: 'stopped' | 'running' | string;
};

type NodeHealth = {
  running: boolean;
  pid: number | null;
  uptimeSec: number | null;
  rpcReachable: boolean;
};

function statusTag(status: NodeStatus | null): { text: string; color: string } {
  if (!status) {
    return { text: '加载中', color: 'default' };
  }
  if (status.state === 'running') {
    return { text: '运行中', color: 'green' };
  }
  if (status.state === 'stopped') {
    return { text: '已停止', color: 'gold' };
  }
  return { text: status.state, color: 'default' };
}

function assertValidWalletAddress(input: string): string {
  const value = input.trim();
  if (!value) {
    throw new Error('钱包地址不能为空');
  }

  try {
    const bytes = decodeAddress(value);
    if (bytes.length !== 32) {
      throw new Error('钱包地址长度无效');
    }
    return value;
  } catch {
    if (/^0x[0-9a-fA-F]{64}$/.test(value)) {
      return value.toLowerCase();
    }
    throw new Error('钱包地址格式不合法');
  }
}

function assertValidNodeKey(input: string): string {
  const value = input.trim();
  if (!value) {
    throw new Error('node-key 不能为空');
  }
  const raw = value.startsWith('0x') ? value.slice(2) : value;
  if (!/^[0-9a-fA-F]{64}$/.test(raw)) {
    throw new Error('node-key 必须是 64 位十六进制字符串');
  }
  return raw.toLowerCase();
}

async function signAndSendBindTx(walletAddress: string): Promise<string> {
  const api = await connectNode(DEFAULT_NODE_ENDPOINT);
  await cryptoWaitReady();

  const minerSuri = await invoke<string>('get_miner_suri');
  const keyring = new Keyring({ type: 'sr25519' });
  const minerPair = keyring.addFromUri(minerSuri);
  const minerAddress = minerPair.address;

  const existing = await api.query.fullnodePowReward.rewardWalletByMiner(minerAddress);
  const hasExisting = Boolean((existing as { isSome?: boolean }).isSome);
  const tx = hasExisting
    ? api.tx.fullnodePowReward.rebindRewardWallet(walletAddress)
    : api.tx.fullnodePowReward.bindRewardWallet(walletAddress);

  await new Promise<void>((resolve, reject) => {
    let unsub: (() => void) | undefined;

    tx.signAndSend(minerPair, (result: any) => {
      if (result.dispatchError) {
        let detail = result.dispatchError.toString();
        if (result.dispatchError.isModule) {
          const decoded = api.registry.findMetaError(result.dispatchError.asModule);
          detail = `${decoded.section}.${decoded.name}`;
        }
        if (unsub) unsub();
        reject(new Error(`链上绑定失败：${detail}`));
        return;
      }

      if (result.status?.isInBlock || result.status?.isFinalized) {
        if (unsub) unsub();
        resolve();
      }
    })
      .then((u: any) => {
        unsub = u;
      })
      .catch((err: unknown) => reject(err instanceof Error ? err : new Error(String(err))));
  });

  await invoke<string>('set_reward_wallet_address', { address: walletAddress });
  return minerAddress;
}

export default function App() {
  const [status, setStatus] = useState<NodeStatus | null>(null);
  const [health, setHealth] = useState<NodeHealth | null>(null);
  const [blockHeight, setBlockHeight] = useState<number | null>(null);
  const [walletAddress, setWalletAddress] = useState('');
  const [bootnodeNodeKey, setBootnodeNodeKey] = useState('');
  const [walletModalOpen, setWalletModalOpen] = useState(false);
  const [nodeKeyModalOpen, setNodeKeyModalOpen] = useState(false);
  const [walletInput, setWalletInput] = useState('');
  const [nodeKeyInput, setNodeKeyInput] = useState('');
  const [logs, setLogs] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [walletSaving, setWalletSaving] = useState(false);
  const [nodeKeySaving, setNodeKeySaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const tag = useMemo(() => statusTag(status), [status]);

  const loadStatus = useCallback(async () => {
    const next = await invoke<NodeStatus>('get_installer_status');
    setStatus(next);
  }, []);

  const loadLogs = useCallback(async () => {
    const next = await invoke<string[]>('get_logs', { tail: 80 });
    setLogs(next);
  }, []);

  const loadHealth = useCallback(async () => {
    const next = await invoke<NodeHealth>('get_node_health');
    setHealth(next);
  }, []);

  const loadBlockHeight = useCallback(async () => {
    try {
      const api = await connectNode(DEFAULT_NODE_ENDPOINT);
      const header = await api.rpc.chain.getHeader();
      setBlockHeight(header.number.toNumber());
    } catch {
      setBlockHeight(null);
    }
  }, []);

  const loadWalletAddress = useCallback(async () => {
    const next = await invoke<string | null>('get_reward_wallet_address');
    const value = next ?? '';
    setWalletAddress(value);
    setWalletInput(value);
  }, []);

  const loadNodeKey = useCallback(async () => {
    const next = await invoke<string | null>('get_bootnode_node_key');
    const value = next ?? '';
    setBootnodeNodeKey(value);
    setNodeKeyInput(value);
  }, []);

  const runAction = useCallback(
    async (command: 'start_node' | 'stop_node') => {
      setBusy(true);
      setError(null);
      try {
        const next = await invoke<NodeStatus>(command);
        setStatus(next);
        await Promise.all([loadLogs(), loadHealth(), loadBlockHeight()]);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [loadBlockHeight, loadHealth, loadLogs]
  );

  const bindWallet = useCallback(async () => {
    setWalletSaving(true);
    setError(null);
    try {
      const normalized = assertValidWalletAddress(walletInput);
      const minerAddress = await signAndSendBindTx(normalized);
      setWalletAddress(normalized);
      setWalletInput(normalized);
      setWalletModalOpen(false);
      message.success(`绑定成功，矿工地址：${minerAddress}`);
      await Promise.all([loadLogs(), loadHealth()]);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      message.error(msg);
    } finally {
      setWalletSaving(false);
    }
  }, [loadHealth, loadLogs, walletInput]);

  const saveNodeKey = useCallback(async () => {
    setNodeKeySaving(true);
    setError(null);
    try {
      const normalized = assertValidNodeKey(nodeKeyInput);
      const saved = await invoke<string>('set_bootnode_node_key', { nodeKey: normalized });
      setBootnodeNodeKey(saved);
      setNodeKeyInput(saved);
      setNodeKeyModalOpen(false);
      message.success('引导节点 node-key 已保存');
      await loadLogs();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      message.error(msg);
    } finally {
      setNodeKeySaving(false);
    }
  }, [loadLogs, nodeKeyInput]);

  useEffect(() => {
    Promise.all([
      loadStatus(),
      loadLogs(),
      loadHealth(),
      loadBlockHeight(),
      loadWalletAddress(),
      loadNodeKey(),
    ]).catch((err) => setError(String(err)));

    const timer = window.setInterval(() => {
      loadStatus().catch(() => undefined);
      loadLogs().catch(() => undefined);
      loadHealth().catch(() => undefined);
      loadBlockHeight().catch(() => undefined);
    }, 2000);

    return () => window.clearInterval(timer);
  }, [loadBlockHeight, loadHealth, loadLogs, loadNodeKey, loadStatus, loadWalletAddress]);

  return (
    <Layout style={{ minHeight: '100vh', background: 'transparent' }}>
      <Content style={{ maxWidth: 960, width: '100%', margin: '0 auto', padding: '24px 20px' }}>
        <Space direction="vertical" size={16} style={{ width: '100%' }}>
          <Card>
            <Space direction="vertical" size={10} style={{ width: '100%' }}>
              <Space align="center" style={{ width: '100%', justifyContent: 'space-between' }}>
                <Typography.Title level={3} style={{ margin: 0 }}>
                  公民币区块链节点控制台
                </Typography.Title>
                <Space>
                  <Button onClick={() => setNodeKeyModalOpen(true)}>引导 Node-Key</Button>
                  <Button onClick={() => setWalletModalOpen(true)}>钱包绑定</Button>
                </Space>
              </Space>

              <Space size={10} wrap>
                <Typography.Text>节点状态</Typography.Text>
                <Tag color={tag.color}>{tag.text}</Tag>
              </Space>

              <Space size={10} wrap>
                <Button
                  type="primary"
                  loading={busy}
                  onClick={() => runAction('start_node')}
                  disabled={busy || status?.running === true}
                >
                  启动
                </Button>
                <Button
                  danger
                  loading={busy}
                  onClick={() => runAction('stop_node')}
                  disabled={busy || status?.running !== true}
                >
                  停止
                </Button>
              </Space>

              <Typography.Text type="secondary">
                当前绑定收款地址：{walletAddress || '-'}
              </Typography.Text>
              <Typography.Text type="secondary">
                当前引导 node-key：{bootnodeNodeKey || '-'}
              </Typography.Text>

              {error ? <Alert type="error" showIcon message={error} /> : null}
            </Space>
          </Card>

          <Card title="节点健康">
            <Space direction="vertical" size={8} style={{ width: '100%' }}>
              <Space size={10} wrap>
                <Typography.Text>RPC 连通</Typography.Text>
                <Tag color={health?.rpcReachable ? 'green' : 'red'}>
                  {health?.rpcReachable ? '通过' : '失败'}
                </Tag>
              </Space>
              <Typography.Text type="secondary">PID：{health?.pid ?? '-'}</Typography.Text>
              <Typography.Text type="secondary">
                运行时长：{health?.uptimeSec != null ? `${health.uptimeSec}s` : '-'}
              </Typography.Text>
              <Typography.Text type="secondary">
                当前区块高度：{blockHeight != null ? blockHeight : '-'}
              </Typography.Text>
            </Space>
          </Card>

          <Card title="运行日志（最近 80 行）">
            <pre
              style={{
                margin: 0,
                minHeight: 260,
                maxHeight: 420,
                overflow: 'auto',
                whiteSpace: 'pre-wrap',
                color: '#cbd5e1',
                fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
                fontSize: 12,
                lineHeight: 1.45,
              }}
            >
              {logs.length > 0 ? logs.join('\n') : '暂无日志'}
            </pre>
          </Card>
        </Space>
      </Content>

      <Modal
        title="绑定收款钱包"
        open={walletModalOpen}
        onCancel={() => setWalletModalOpen(false)}
        onOk={bindWallet}
        okText="绑定"
        confirmLoading={walletSaving}
      >
        <Space direction="vertical" size={10} style={{ width: '100%' }}>
          <Typography.Text type="secondary">
            输入合法钱包地址后点击绑定，将发起真实链上绑定交易。
          </Typography.Text>
          <Input
            value={walletInput}
            onChange={(event) => setWalletInput(event.target.value)}
            placeholder="输入钱包地址"
          />
        </Space>
      </Modal>

      <Modal
        title="配置引导节点 Node-Key"
        open={nodeKeyModalOpen}
        onCancel={() => setNodeKeyModalOpen(false)}
        onOk={saveNodeKey}
        okText="保存"
        confirmLoading={nodeKeySaving}
      >
        <Space direction="vertical" size={10} style={{ width: '100%' }}>
          <Typography.Text type="secondary">
            输入 64 位十六进制 node-key（可带 0x），保存后下次启动节点时自动带上 --node-key。
          </Typography.Text>
          <Input
            value={nodeKeyInput}
            onChange={(event) => setNodeKeyInput(event.target.value)}
            placeholder="例如：0x0123...abcd"
          />
        </Space>
      </Modal>
    </Layout>
  );
}
