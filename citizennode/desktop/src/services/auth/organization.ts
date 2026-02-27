import { decodeAddress } from '@polkadot/util-crypto';
import { ORG_REGISTRY } from '../../constants/orgRegistry.generated';
import type { LoginSession } from '../../types/auth';

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

export function resolveOrganizationByAddress(address: string): LoginSession | null {
  let normalizedInput: string;
  try {
    normalizedInput = normalized(asHexAddress(address));
  } catch {
    return null;
  }

  const hit = ORG_REGISTRY.find((item) => normalized(item.adminAddress) === normalizedInput);
  if (!hit) return null;

  return {
    role: hit.role,
    publicKey: hit.adminAddress,
    province: hit.province,
    organizationName: hit.organizationName
  };
}

export function resolveCitizenchainSession(address: string): LoginSession | null {
  let normalizedInput: string;
  try {
    normalizedInput = normalized(asHexAddress(address));
  } catch {
    return null;
  }

  const org = resolveOrganizationByAddress(normalizedInput);
  if (org) {
    return org;
  }

  return {
    role: 'full',
    publicKey: normalizedInput,
    organizationName: '全节点'
  };
}
