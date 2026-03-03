import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import {
  AmbiguousAdminMappingError,
  asHexAddress,
  resolveCitizenchainSessionRuntime,
  resolveOrganizationByAddressFromRegistry
} from '../src/services/auth/organization.js';

test('asHexAddress keeps valid hex pubkey unchanged', () => {
  const hex = '0x1111111111111111111111111111111111111111111111111111111111111111';
  const normalized = asHexAddress(hex);
  assert.equal(normalized, hex);
});

test('resolveCitizenchainSessionRuntime returns null without runtime snapshot', async () => {
  const unknown = '0x1111111111111111111111111111111111111111111111111111111111111111';
  const session = await resolveCitizenchainSessionRuntime(unknown);
  assert.equal(session, null);
});

test('resolveOrganizationByAddressFromRegistry returns org session for single match', () => {
  const admin = '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const session = resolveOrganizationByAddressFromRegistry(admin, [
    { role: 'prb', organizationName: '贵州省储行', province: '贵州', adminAddress: admin }
  ]);
  assert.deepEqual(session, {
    role: 'prb',
    publicKey: admin,
    province: '贵州',
    organizationName: '贵州省储行'
  });
});

test('resolveOrganizationByAddressFromRegistry throws for ambiguous admin mappings', () => {
  const admin = '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  assert.throws(
    () =>
      resolveOrganizationByAddressFromRegistry(admin, [
        { role: 'prc', organizationName: '贵州省储会', province: '贵州', adminAddress: admin },
        { role: 'prb', organizationName: '贵州省储行', province: '贵州', adminAddress: admin }
      ]),
    AmbiguousAdminMappingError
  );
});
