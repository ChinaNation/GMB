import { describe, expect, it } from 'vitest';
import { resourceLimit, videoResource } from '../src/limits/catalog';
import {
  assertKnownRoute,
  assertRequestBodyLimit,
  readLimitedBytes,
} from '../src/limits/request';
import { assertDeclaredResource, validateUploadBytes } from '../src/limits/upload';
import { membershipUsagePeriod } from '../src/limits/usage';
import { HttpError } from '../src/shared/http';

describe('Cloudflare 统一资源限制', () => {
  it('固定压缩后的图片、视频和聊天硬上限', () => {
    expect(resourceLimit('profile_avatar').max_bytes).toBe(512 * 1024);
    expect(resourceLimit('square_image_sd').max_bytes).toBe(1024 * 1024);
    expect(resourceLimit('square_image_hd').max_bytes).toBe(3 * 1024 * 1024);
    expect(resourceLimit(videoResource('freedom')).max_bytes).toBe(40 * 1024 * 1024);
    expect(resourceLimit(videoResource('candidate')).max_seconds).toBe(3 * 60 * 60);
    expect(resourceLimit('chat_keypackage').max_count).toBe(20);
  });

  it('在进入风控和 D1 前拒绝未登记路由', () => {
    expect(() => assertKnownRoute('GET', '/v1/unknown')).toThrowError(HttpError);
    expect(() => assertKnownRoute('POST', '/v1/square/reports')).toThrowError(HttpError);
    expect(assertKnownRoute('PUT', '/v1/square/uploads/media')).toBe('square_image_hd');
  });

  it('拒绝没有 Content-Length 或声明超限的写请求', () => {
    expect(() => assertRequestBodyLimit(new Request('https://worker.test/v1/chat/signals', {
      method: 'POST',
      body: '{}',
    }), '/v1/chat/signals')).toThrow(expect.objectContaining({ code: 'content_length_required' }));

    expect(() => assertRequestBodyLimit(new Request('https://worker.test/v1/chat/signals', {
      method: 'POST',
      headers: { 'content-length': String(64 * 1024 + 1) },
    }), '/v1/chat/signals')).toThrow(expect.objectContaining({ code: 'request_too_large' }));
  });

  it('没有可信声明长度时仍在流读取阶段截断', async () => {
    const bytes = new Uint8Array(512 * 1024 + 1);
    const request = new Request('https://worker.test/v1/square/profile/assets', {
      method: 'PUT',
      body: bytes,
    });
    await expect(readLimitedBytes(request, 'profile_avatar')).rejects.toMatchObject({
      code: 'request_too_large',
    });
  });

  it('校验图片文件头、真实尺寸、字节和哈希后才签发限制凭证', async () => {
    const png = pngHeader(100, 80);
    const ticket = await validateUploadBytes({
      resource_key: 'profile_avatar',
      bytes: png,
      content_type: 'image/png',
      expected_bytes: png.length,
    });
    expect(ticket.width).toBe(100);
    expect(ticket.height).toBe(80);
    expect(ticket.content_hash).toMatch(/^[a-f0-9]{64}$/);

    await expect(validateUploadBytes({
      resource_key: 'profile_avatar',
      bytes: pngHeader(2000, 80),
      content_type: 'image/png',
    })).rejects.toMatchObject({ code: 'image_dimensions_exceeded' });
  });

  it('环境外声明不能突破最高档视频硬上限', () => {
    expect(() => assertDeclaredResource({
      resource_key: 'square_video_candidate',
      byte_size: resourceLimit('square_video_candidate').max_bytes + 1,
      content_type: 'video/mp4',
      duration_seconds: 1,
    })).toThrow(expect.objectContaining({ code: 'resource_size_invalid' }));
  });

  it('订阅周期起点缺失时使用稳定周期而不是请求时间', () => {
    const periodEnd = 2_000_000_000_000;
    const first = membershipUsagePeriod({
      current_period_start: null,
      current_period_end: periodEnd,
      expires_at: periodEnd,
    });
    const second = membershipUsagePeriod({
      current_period_start: null,
      current_period_end: periodEnd,
      expires_at: periodEnd,
    });
    expect(first).toEqual(second);
    expect(first.periodStart).toBe(periodEnd - 31 * 24 * 60 * 60 * 1000);
  });
});

function pngHeader(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(24);
  bytes.set([137, 80, 78, 71, 13, 10, 26, 10]);
  const view = new DataView(bytes.buffer);
  view.setUint32(16, width);
  view.setUint32(20, height);
  return bytes;
}
