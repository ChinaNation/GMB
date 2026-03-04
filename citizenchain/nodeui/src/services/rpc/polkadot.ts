import { ApiPromise, WsProvider } from '@polkadot/api';
import { assertSafeLocalRpcEndpoint } from '../../utils/rpcEndpoint';

let apiInstance: ApiPromise | null = null;
let apiEndpoint: string | null = null;
let connectInFlight: Promise<ApiPromise> | null = null;

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
