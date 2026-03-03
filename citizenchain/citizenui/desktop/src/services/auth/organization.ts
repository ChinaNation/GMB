import { decodeAddress } from '@polkadot/util-crypto';
import type { OrganizationRegistryItem } from '../../constants/orgRegistry.types';
import type { LoginIdentity } from '../../types/auth';
import { loadRuntimeOrgRegistry } from './runtimeRegistry';

export class AmbiguousAdminMappingError extends Error {
  constructor() {
    super('admin address matches multiple organizations');
    this.name = 'AmbiguousAdminMappingError';
  }
}

function normalized(value: string): string {
  return value.trim().toLowerCase();
}

export function asHexAddress(address: string): string {
  const input = address.trim();
  if (input.startsWith('0x') && input.length === 66) {
    return input.toLowerCase();
  }

  const decoded = decodeAddress(input);
  return `0x${Array.from(decoded)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')}`;
}

export function resolveOrganizationByAddressFromRegistry(
  address: string,
  registry: OrganizationRegistryItem[]
): LoginIdentity | null {
  let normalizedInput: string;
  try {
    normalizedInput = normalized(asHexAddress(address));
  } catch {
    return null;
  }

  const hits = registry.filter((item) => normalized(item.adminAddress) === normalizedInput);
  if (hits.length === 0) return null;
  if (hits.length > 1) {
    throw new AmbiguousAdminMappingError();
  }
  const hit = hits[0];

  return {
    role: hit.role,
    publicKey: hit.adminAddress,
    province: hit.province,
    organizationName: hit.organizationName
  };
}

export async function resolveCitizenchainSessionRuntime(
  address: string
): Promise<LoginIdentity | null> {
  let normalizedInput: string;
  try {
    normalizedInput = normalized(asHexAddress(address));
  } catch {
    return null;
  }
  const runtimeRegistry = await loadRuntimeOrgRegistry();
  if (runtimeRegistry && runtimeRegistry.length > 0) {
    return resolveOrganizationByAddressFromRegistry(normalizedInput, runtimeRegistry);
  }
  return null;
}
