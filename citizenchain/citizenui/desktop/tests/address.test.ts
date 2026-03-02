import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { isValidAddress } from '../src/utils/address.js';

test('isValidAddress accepts hex and ss58-like addresses', () => {
  assert.equal(
    isValidAddress('0x9aa1e0672efcf2e186a6237da9fa706279e2c1d785212c48334bde7cae400215'),
    true
  );
  assert.equal(
    isValidAddress('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'),
    true
  );
});

test('isValidAddress rejects malformed addresses', () => {
  assert.equal(isValidAddress(''), false);
  assert.equal(isValidAddress('0x1234'), false);
  assert.equal(isValidAddress('4NotSs58Address'), false);
});
