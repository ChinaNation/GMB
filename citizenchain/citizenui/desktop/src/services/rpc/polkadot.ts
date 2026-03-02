import { ApiPromise, WsProvider } from '@polkadot/api';
import { asHexAddress } from '../auth/organization';
import { assertSafeLocalRpcEndpoint } from '../../utils/rpcEndpoint';

let apiInstance: ApiPromise | null = null;
let apiEndpoint: string | null = null;
let connectInFlight: Promise<ApiPromise> | null = null;

export type RecentTransaction = {
  blockNumber: number;
  extrinsicIndex: number;
  hash: string;
  section: string;
  method: string;
  signer?: string;
};

export async function connectNode(endpoint: string): Promise<ApiPromise> {
  const safeEndpoint = assertSafeLocalRpcEndpoint(endpoint);

  if (apiInstance && apiEndpoint === safeEndpoint) {
    return apiInstance;
  }
  if (connectInFlight) {
    return connectInFlight;
  }

  connectInFlight = (async () => {
    if (apiInstance) {
      await apiInstance.disconnect();
      apiInstance = null;
      apiEndpoint = null;
    }

    const provider = new WsProvider(safeEndpoint);
    apiInstance = await ApiPromise.create({ provider });
    apiEndpoint = safeEndpoint;
    return apiInstance;
  })();

  try {
    return await connectInFlight;
  } finally {
    connectInFlight = null;
  }
}

export async function getApi(endpoint: string): Promise<ApiPromise> {
  const safeEndpoint = assertSafeLocalRpcEndpoint(endpoint);
  if (apiInstance && apiEndpoint === safeEndpoint) {
    return apiInstance;
  }
  return connectNode(safeEndpoint);
}

export async function readChainHead(endpoint: string): Promise<number> {
  const api = await getApi(endpoint);
  const header = await api.rpc.chain.getHeader();
  return header.number.toNumber();
}

export async function readAccountBalance(endpoint: string, address: string): Promise<string> {
  const api = await getApi(endpoint);
  const account = await api.query.system.account(address);
  const json = account.toJSON() as { data?: { free?: string | number | bigint } };
  const raw = json.data?.free;
  if (raw === undefined) {
    return '0';
  }
  try {
    return BigInt(raw).toString(10);
  } catch {
    return '0';
  }
}

function normalizeComparableAddress(value: string | undefined): string | null {
  if (!value) {
    return null;
  }
  const text = value.trim();
  if (!text) {
    return null;
  }
  try {
    return asHexAddress(text).toLowerCase();
  } catch {
    return text.toLowerCase();
  }
}

export async function readRecentTransactions(
  endpoint: string,
  options?: { address?: string; depth?: number; limit?: number; signal?: AbortSignal }
): Promise<RecentTransaction[]> {
  const api = await getApi(endpoint);
  const head = await readChainHead(endpoint);

  const address = options?.address?.trim();
  const normalizedAddress = normalizeComparableAddress(address);
  const depth = options?.depth ?? 30;
  const limit = options?.limit ?? 20;
  const signal = options?.signal;
  const batchSize = 5;
  const start = Math.max(head - depth + 1, 0);

  const rows: RecentTransaction[] = [];

  const blockNumbers = Array.from({ length: head - start + 1 }, (_, idx) => head - idx);

  for (let i = 0; i < blockNumbers.length; i += batchSize) {
    if (signal?.aborted) {
      throw new DOMException('Query aborted', 'AbortError');
    }

    const batch = blockNumbers.slice(i, i + batchSize);
    const blocks = await Promise.all(
      batch.map(async (blockNumber) => {
        const hash = await api.rpc.chain.getBlockHash(blockNumber);
        const signedBlock = await api.rpc.chain.getBlock(hash);
        return { blockNumber, extrinsics: signedBlock.block.extrinsics };
      })
    );

    for (const block of blocks) {
      if (signal?.aborted) {
        throw new DOMException('Query aborted', 'AbortError');
      }
      block.extrinsics.forEach((ex, index) => {
        const signer = ex.isSigned ? ex.signer.toString() : undefined;
        const section = ex.method.section.toString();
        const method = ex.method.method.toString();
        const hashText = ex.hash.toHex();

        if (normalizedAddress) {
          const normalizedSigner = normalizeComparableAddress(signer);
          const relatedBySigner = normalizedSigner === normalizedAddress;
          const relatedByArgs = ex.method.args.some((arg) => {
            const normalizedArg = normalizeComparableAddress(arg.toString());
            return normalizedArg === normalizedAddress;
          });
          const related = relatedBySigner || relatedByArgs;
          if (!related) return;
        }

        rows.push({
          blockNumber: block.blockNumber,
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
  }

  return rows;
}
