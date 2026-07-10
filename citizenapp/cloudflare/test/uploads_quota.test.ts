import { describe, expect, it } from 'vitest';
import { membershipPlans } from '../src/membership/plans';
import { assertDeclaredContentQuota, assertManifestQuota } from '../src/uploads/quota';
import type { MediaAssetRow, UploadItemInput } from '../src/types';

describe('membership upload quotas', () => {
  it('rejects a dynamic post whose text exceeds the member quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'visitor',
        plan: membershipPlans.visitor,
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
        membershipLevel: 'candidate',
        plan: membershipPlans.candidate,
        postCategory: 'normal',
        contentFormat: 'normal',
        titleLength: 0,
        textLength: 300,
        mediaItems: [...Array.from({ length: 9 }, image), video()]
      })
    ).not.toThrow();
  });

  it('rejects campaign content for non-candidate membership', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'voting',
        plan: membershipPlans.voting,
        postCategory: 'campaign',
        contentFormat: 'normal',
        titleLength: 0,
        textLength: 120,
        mediaItems: [image()]
      })
    ).toThrow(expect.objectContaining({ code: 'campaign_membership_required' }));
  });

  it('rejects visitor article body images over the visitor quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'visitor',
        plan: membershipPlans.visitor,
        postCategory: 'normal',
        contentFormat: 'article',
        titleLength: 12,
        textLength: 200,
        mediaItems: [image(), ...Array.from({ length: 51 }, image)]
      })
    ).toThrow(expect.objectContaining({ code: 'article_image_count_exceeded' }));
  });

  it('allows candidate campaign articles inside the article quota', () => {
    expect(() =>
      assertDeclaredContentQuota({
        membershipLevel: 'candidate',
        plan: membershipPlans.candidate,
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
        membershipLevel: 'visitor',
        plan: membershipPlans.visitor,
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
        membershipLevel: 'visitor',
        plan: membershipPlans.visitor,
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
    upload_method: 'direct_form',
    content_type: 'image/jpeg',
    byte_size: 1024,
    asset_state: 'ready',
    delivery_url: 'https://img.test/1',
    playback_hls_url: null,
    playback_dash_url: null,
    thumbnail_url: null,
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
