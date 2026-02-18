import { ApiPromise, WsProvider } from '@polkadot/api';

let apiInstance: ApiPromise | null = null;

export type RecentTransaction = {
  blockNumber: number;
  extrinsicIndex: number;
  hash: string;
  section: string;
  method: string;
  signer?: string;
};

export async function connectNode(endpoint: string): Promise<ApiPromise> {
  if (apiInstance) {
    await apiInstance.disconnect();
    apiInstance = null;
  }

  const provider = new WsProvider(endpoint);
  apiInstance = await ApiPromise.create({ provider });
  return apiInstance;
}

export async function getApi(endpoint: string): Promise<ApiPromise> {
  return apiInstance ?? connectNode(endpoint);
}

export async function readChainHead(endpoint: string): Promise<number> {
  const api = await getApi(endpoint);
  const header = await api.rpc.chain.getHeader();
  return header.number.toNumber();
}

export async function readAccountBalance(endpoint: string, address: string): Promise<string> {
  const api = await getApi(endpoint);
  const account = (await api.query.system.account(address)) as unknown as {
    data?: { free?: { toString: () => string } };
  };
  return account.data?.free?.toString?.() ?? '0';
}

export async function readRecentTransactions(
  endpoint: string,
  options?: { address?: string; depth?: number; limit?: number }
): Promise<RecentTransaction[]> {
  const api = await getApi(endpoint);
  const head = await readChainHead(endpoint);

  const address = options?.address?.trim();
  const depth = options?.depth ?? 30;
  const limit = options?.limit ?? 20;
  const start = Math.max(head - depth + 1, 0);

  const rows: RecentTransaction[] = [];

  for (let blockNumber = head; blockNumber >= start; blockNumber -= 1) {
    const hash = await api.rpc.chain.getBlockHash(blockNumber);
    const signedBlock = await api.rpc.chain.getBlock(hash);
    const extrinsics = signedBlock.block.extrinsics;

    extrinsics.forEach((ex, index) => {
      const signer = ex.isSigned ? ex.signer.toString() : undefined;
      const section = ex.method.section;
      const method = ex.method.method;
      const hashText = ex.hash.toHex();
      const argsText = ex.method.args.map((arg) => arg.toString()).join(' ');

      if (address) {
        const related = signer === address || argsText.includes(address);
        if (!related) return;
      }

      rows.push({
        blockNumber,
        extrinsicIndex: index,
        hash: hashText,
        section,
        method,
        signer
      });
    });

    if (rows.length >= limit) {
      return rows.slice(0, limit);
    }
  }

  return rows;
}
