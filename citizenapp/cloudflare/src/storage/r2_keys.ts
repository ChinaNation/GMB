import type { UploadItemInput } from '../types';

export interface ObjectKeyPlan {
  manifest_object_key: string;
  media_items: Array<{
    media_kind: UploadItemInput['media_kind'];
    content_type: string;
    byte_size: number;
    object_key: string;
  }>;
  object_keys: string[];
}

export function sanitizeOwnerAccount(ownerAccount: string): string {
  return ownerAccount.replace(/[^a-zA-Z0-9._-]/g, '_');
}

export function normalizeFileExt(contentType: string, fileExt?: string): string {
  if (fileExt && /^[a-z0-9]{2,8}$/i.test(fileExt)) {
    return fileExt.toLowerCase();
  }

  if (contentType === 'image/jpeg') {
    return 'jpg';
  }
  if (contentType === 'image/png') {
    return 'png';
  }
  if (contentType === 'image/webp') {
    return 'webp';
  }
  if (contentType === 'video/mp4') {
    return 'mp4';
  }
  if (contentType === 'video/webm') {
    return 'webm';
  }

  return 'bin';
}

export function buildObjectKeyPlan(
  ownerAccount: string,
  postId: string,
  mediaItems: UploadItemInput[]
): ObjectKeyPlan {
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  const basePath = `square/${safeOwner}/posts/${postId}`;
  const manifestObjectKey = `${basePath}/manifest.json`;
  let imageIndex = 0;
  let videoIndex = 0;
  let coverIndex = 0;

  const plannedMedia = mediaItems.map((item) => {
    const fileExt = normalizeFileExt(item.content_type, item.file_ext);
    let name: string;

    if (item.media_kind === 'video') {
      videoIndex += 1;
      name = `video_${String(videoIndex).padStart(3, '0')}.${fileExt}`;
    } else if (item.media_kind === 'cover') {
      coverIndex += 1;
      name = coverIndex === 1 ? `cover.${fileExt}` : `cover_${coverIndex}.${fileExt}`;
    } else {
      imageIndex += 1;
      name = `media_${String(imageIndex).padStart(3, '0')}.${fileExt}`;
    }

    return {
      media_kind: item.media_kind,
      content_type: item.content_type,
      byte_size: item.byte_size,
      object_key: `${basePath}/${name}`
    };
  });

  return {
    manifest_object_key: manifestObjectKey,
    media_items: plannedMedia,
    object_keys: [manifestObjectKey, ...plannedMedia.map((item) => item.object_key)]
  };
}
