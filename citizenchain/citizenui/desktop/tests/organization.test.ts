import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { ORG_REGISTRY } from '../src/constants/orgRegistry.generated.js';
import { resolveCitizenchainSession, resolveOrganizationByAddress } from '../src/services/auth/organization.js';

test('resolveOrganizationByAddress matches known registry entry', () => {
  const first = ORG_REGISTRY[0];
  assert.ok(first, 'registry should contain entries');

  const session = resolveOrganizationByAddress(first.adminAddress);
  assert.ok(session);
  assert.equal(session?.publicKey, first.adminAddress);
  assert.equal(session?.role, first.role);
});

test('resolveCitizenchainSession rejects unknown address', () => {
  const unknown = '0x1111111111111111111111111111111111111111111111111111111111111111';
  const session = resolveCitizenchainSession(unknown);
  assert.equal(session, null);
});
