import { describe, expect, it } from 'vitest';
import { membershipPlans } from '../src/membership/plans';
import {
  assertDeclaredContentQuota,
  assertIdentityCanPublishCategory,
  assertManifestQuota
} from '../src/uploads/quota';
import type { MediaAssetRow, UploadItemInput } from '../src/types';

describe('membership upload quotas', () => {
  it('rejects a dynamic post whose text exceeds the member quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'freedom',
        plan: membershipPlans.freedom,
        postCategory: 'normal',
        contentFormat: 'normal',
        titleLength: 0,
        textLength: 301,
        mediaItems: [image()]
      })
    ).toThrow(expect.objectContaining({ code: 'dynamic_text_too_long' }));
  });

  it('allows a dynamic post with 9 images and 1 video under the member quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'spark',
        plan: membershipPlans.spark,
        postCategory: 'normal',
        contentFormat: 'normal',
        titleLength: 0,
        textLength: 300,
        mediaItems: [...Array.from({ length: 9 }, image), video()]
      })
    ).not.toThrow();
  });

  it('gates campaign category by identity, not membership (ADR-036 + 用户 2026-07-16)', () => {
    // 用量额度按会员校验，不再把关分类权限：民主会员发竞选帖过额度校验（分类由身份另管）。
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'democracy',
        plan: membershipPlans.democracy,
        postCategory: 'campaign',
        contentFormat: 'normal',
        titleLength: 0,
        textLength: 120,
        mediaItems: [image()]
      })
    ).not.toThrow();
    // 分类权限按身份：非竞选身份发竞选帖被拒；竞选身份放行；普通帖任意身份放行。
    expect(() => assertIdentityCanPublishCategory('visitor', 'campaign')).toThrow(
      expect.objectContaining({ code: 'campaign_identity_required' })
    );
    expect(() => assertIdentityCanPublishCategory('voting', 'campaign')).toThrow(
      expect.objectContaining({ code: 'campaign_identity_required' })
    );
    expect(() => assertIdentityCanPublishCategory('candidate', 'campaign')).not.toThrow();
    expect(() => assertIdentityCanPublishCategory('visitor', 'normal')).not.toThrow();
  });

  it('rejects freedom article body images over its quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'freedom',
        plan: membershipPlans.freedom,
        postCategory: 'normal',
        contentFormat: 'article',
        titleLength: 12,
        textLength: 200,
        mediaItems: [image(), ...Array.from({ length: 51 }, image)]
      })
    ).toThrow(expect.objectContaining({ code: 'article_image_count_exceeded' }));
  });

  it('allows a campaign article within the article quota (category not membership-gated)', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'spark',
        plan: membershipPlans.spark,
        postCategory: 'campaign',
        contentFormat: 'article',
        titleLength: 12,
        textLength: 30_000,
        mediaItems: [image(), ...Array.from({ length: 100 }, image)]
      })
    ).not.toThrow();
  });

  it('checks the uploaded R2 manifest against actual media assets', async () => {
    const manifestText = JSON.stringify({
      schema: 'citizenapp.square.post.v1',
      owner_account: 'owner_1',
      post_category: 'normal',
      content_format: 'article',
      title: '标题标题标题标题标题',
      text: '正文',
      media_items: [
        {
          media_kind: 'image',
          content_type: 'image/jpeg',
          byte_size: 1024,
          sha256: '11'.repeat(32)
        }
      ]
    });

    await expect(
      assertManifestQuota({
        membershipLevel: 'freedom',
        plan: membershipPlans.freedom,
        upload: {
          owner_account: 'owner_1',
          post_category: 'normal'
        },
        manifestText,
        mediaAssets: [mediaAsset()]
      })
    ).resolves.toBeUndefined();
  });

  it('rejects a manifest whose media does not match the signed upload assets', async () => {
    const manifestText = JSON.stringify({
      schema: 'citizenapp.square.post.v1',
      owner_account: 'owner_1',
      post_category: 'normal',
      text: '正文',
      media_items: [
        {
          media_kind: 'image',
          content_type: 'image/png',
          byte_size: 1024,
          sha256: '11'.repeat(32)
        }
      ]
    });

    await expect(
      assertManifestQuota({
        membershipLevel: 'freedom',
        plan: membershipPlans.freedom,
        upload: {
          owner_account: 'owner_1',
          post_category: 'normal'
        },
        manifestText,
        mediaAssets: [mediaAsset()]
      })
    ).rejects.toMatchObject({ code: 'manifest_media_mismatch' });
  });
});

function image(): UploadItemInput {
  return {
    media_kind: 'image',
    content_type: 'image/jpeg',
    byte_size: 1024
  };
}

function video(): UploadItemInput {
  return {
    media_kind: 'video',
    content_type: 'video/mp4',
    byte_size: 2048
  };
}

function mediaAsset(): MediaAssetRow {
  return {
    upload_id: 'squ_test',
    post_id: 'sqp_test',
    owner_account: 'owner_1',
    media_index: 0,
    media_kind: 'image',
    provider: 'cloudflare_images',
    provider_asset_id: 'img_test',
    upload_method: 'worker',
    resource_key: 'square_image_sd',
    content_type: 'image/jpeg',
    byte_size: 1024,
    asset_state: 'ready',
    declared_duration_seconds: null,
    duration_seconds: null,
    width: null,
    height: null,
    error_code: null,
    created_at: 1,
    updated_at: 1,
    ready_at: 1,
    archive_state: 'live',
    archived_at: null,
    r2_archive_key: null
  };
}
