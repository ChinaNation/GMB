import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import {
  assertSafeLocalRpcEndpoint,
  isSafeLocalRpcEndpoint,
  normalizeRpcEndpoint
} from '../src/utils/rpcEndpoint.js';

test('normalizeRpcEndpoint trims whitespace and trailing slash', () => {
  assert.equal(normalizeRpcEndpoint('  ws://127.0.0.1:9944/  '), 'ws://127.0.0.1:9944');
});

test('isSafeLocalRpcEndpoint accepts local ws endpoints with explicit port', () => {
  assert.equal(isSafeLocalRpcEndpoint('ws://127.0.0.1:9944'), true);
  assert.equal(isSafeLocalRpcEndpoint('ws://localhost:9944'), true);
});

test('isSafeLocalRpcEndpoint rejects remote hosts, wss, and missing port', () => {
  assert.equal(isSafeLocalRpcEndpoint('ws://example.com:9944'), false);
  assert.equal(isSafeLocalRpcEndpoint('wss://127.0.0.1:9944'), false);
  assert.equal(isSafeLocalRpcEndpoint('ws://127.0.0.1'), false);
  assert.equal(isSafeLocalRpcEndpoint('ws://user:pass@127.0.0.1:9944'), false);
  assert.equal(isSafeLocalRpcEndpoint('ws://127.0.0.1:9944/evil'), false);
  assert.equal(isSafeLocalRpcEndpoint('ws://127.0.0.1:9944?x=1'), false);
});

test('assertSafeLocalRpcEndpoint throws on unsafe input', () => {
  assert.throws(() => assertSafeLocalRpcEndpoint('ws://10.0.0.7:9944'));
});
