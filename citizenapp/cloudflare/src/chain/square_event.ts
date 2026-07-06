import { encodeAddress } from '@polkadot/util-crypto';
import type { PostCategory } from '../types';

const squarePostPalletIndex = 36;
const squarePostPublishedEventIndex = 0;
const citizenSs58Prefix = 2027;

export interface SquarePostPublishedEvent {
  post_id: string;
  owner_account: string;
  owner_account_hex: string;
  cid_number: string | null;
  post_category: PostCategory;
  content_hash: string;
  storage_receipt_id: string;
  storage_until: number;
  created_block: number;
}

export function decodeSquarePostPublishedEvents(eventsHex: string): SquarePostPublishedEvent[] {
  const data = hexToBytes(eventsHex);
  if (data.length === 0) return [];
  const [, countSize] = readCompactU32(data, 0);
  const events: SquarePostPublishedEvent[] = [];

  for (let scanOffset = countSize; scanOffset < data.length; scanOffset += 1) {
    try {
      let offset = scanOffset;
      const phase = data[offset];
      offset += 1;
      if (phase === 0x00) {
        if (offset + 4 > data.length) continue;
        offset += 4;
      } else if (phase !== 0x01 && phase !== 0x02) {
        continue;
      }

      if (offset + 2 > data.length) continue;
      const palletIndex = data[offset];
      const eventIndex = data[offset + 1];
      offset += 2;
      if (palletIndex !== squarePostPalletIndex || eventIndex !== squarePostPublishedEventIndex) {
        continue;
      }

      const decoded = decodeSquarePostPublishedEventPayload(data, offset);
      if (decoded) {
        events.push(decoded);
      }
    } catch {
      // System.Events 中混有大量其它 pallet 事件，扫描失败继续尝试下一个 offset。
    }
  }

  return dedupeByPostId(events);
}

export function decodeSquarePostPublishedEventPayload(
  data: Uint8Array,
  offset: number
): SquarePostPublishedEvent | null {
  let cursor = offset;
  const postId = readCompactBytes(data, cursor);
  cursor = postId.nextOffset;
  if (cursor + 32 > data.length) return null;
  const ownerBytes = data.slice(cursor, cursor + 32);
  cursor += 32;

  if (cursor >= data.length) return null;
  let cidNumber: string | null = null;
  const optionFlag = data[cursor];
  cursor += 1;
  if (optionFlag === 1) {
    const cid = readCompactBytes(data, cursor);
    cursor = cid.nextOffset;
    cidNumber = utf8(cid.value);
  } else if (optionFlag !== 0) {
    return null;
  }

  if (cursor + 1 + 32 > data.length) return null;
  const categoryByte = data[cursor];
  cursor += 1;
  if (categoryByte !== 0 && categoryByte !== 1) return null;
  const postCategory: PostCategory = categoryByte === 0 ? 'normal' : 'campaign';

  const contentHash = `0x${hex(data.slice(cursor, cursor + 32))}`;
  cursor += 32;
  const receipt = readCompactBytes(data, cursor);
  cursor = receipt.nextOffset;
  if (cursor + 8 + 4 > data.length) return null;
  const storageUntil = readU64Le(data, cursor);
  cursor += 8;
  const createdBlock = readU32Le(data, cursor);

  return {
    post_id: utf8(postId.value),
    owner_account: encodeAddress(ownerBytes, citizenSs58Prefix),
    owner_account_hex: `0x${hex(ownerBytes)}`,
    cid_number: cidNumber,
    post_category: postCategory,
    content_hash: contentHash,
    storage_receipt_id: utf8(receipt.value),
    storage_until: storageUntil,
    created_block: createdBlock
  };
}

export function compactBytes(value: string): Uint8Array {
  const bytes = new TextEncoder().encode(value);
  return concat([compactU32(bytes.length), bytes]);
}

export function compactU32(value: number): Uint8Array {
  if (value < 0) throw new Error('compact value must be positive');
  if (value < 1 << 6) return Uint8Array.of(value << 2);
  if (value < 1 << 14) {
    const encoded = (value << 2) | 0x01;
    return Uint8Array.of(encoded & 0xff, (encoded >> 8) & 0xff);
  }
  if (value < 1 << 30) {
    const encoded = (value << 2) | 0x02;
    return Uint8Array.of(
      encoded & 0xff,
      (encoded >> 8) & 0xff,
      (encoded >> 16) & 0xff,
      (encoded >> 24) & 0xff
    );
  }
  throw new Error('compact big integer mode is not supported');
}

export function u32Le(value: number): Uint8Array {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, value, true);
  return out;
}

export function u64Le(value: number): Uint8Array {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, BigInt(value), true);
  return out;
}

export function hex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

function dedupeByPostId(events: SquarePostPublishedEvent[]): SquarePostPublishedEvent[] {
  const seen = new Set<string>();
  const deduped: SquarePostPublishedEvent[] = [];
  for (const event of events) {
    if (seen.has(event.post_id)) continue;
    seen.add(event.post_id);
    deduped.push(event);
  }
  return deduped;
}

function readCompactBytes(data: Uint8Array, offset: number): { value: Uint8Array; nextOffset: number } {
  const [length, lengthSize] = readCompactU32(data, offset);
  const start = offset + lengthSize;
  const end = start + length;
  if (end > data.length) {
    throw new Error('compact bytes out of range');
  }
  return {
    value: data.slice(start, end),
    nextOffset: end
  };
}

function readCompactU32(data: Uint8Array, offset: number): [number, number] {
  if (offset >= data.length) throw new Error('compact offset out of range');
  const first = data[offset];
  const mode = first & 0x03;
  if (mode === 0) return [first >> 2, 1];
  if (mode === 1) {
    if (offset + 1 >= data.length) throw new Error('compact mode1 out of range');
    return [(first >> 2) | (data[offset + 1] << 6), 2];
  }
  if (mode === 2) {
    if (offset + 3 >= data.length) throw new Error('compact mode2 out of range');
    return [
      (first >> 2) |
        (data[offset + 1] << 6) |
        (data[offset + 2] << 14) |
        (data[offset + 3] << 22),
      4
    ];
  }
  throw new Error('compact big integer mode is not supported');
}

function readU32Le(data: Uint8Array, offset: number): number {
  return new DataView(data.buffer, data.byteOffset + offset, 4).getUint32(0, true);
}

function readU64Le(data: Uint8Array, offset: number): number {
  const value = new DataView(data.buffer, data.byteOffset + offset, 8).getBigUint64(0, true);
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new Error('u64 exceeds safe integer range');
  }
  return Number(value);
}

function hexToBytes(input: string): Uint8Array {
  const text = input.startsWith('0x') ? input.slice(2) : input;
  if (text.length % 2 !== 0) throw new Error('hex length must be even');
  const out = new Uint8Array(text.length / 2);
  for (let index = 0; index < out.length; index += 1) {
    out[index] = Number.parseInt(text.slice(index * 2, index * 2 + 2), 16);
  }
  return out;
}

function utf8(bytes: Uint8Array): string {
  return new TextDecoder('utf-8', { fatal: false }).decode(bytes);
}

function concat(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}
