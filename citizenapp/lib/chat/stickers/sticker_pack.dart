/// 内置 Fluent 3D 贴纸包的**唯一清单单源**。
///
/// 贴纸不上链、不上云、不走 WebRTC:发送只把 `(packId, stickerId)` 两个短串塞进
/// MLS 明文信封(见 `chat_flow.sendSticker`),接收端按 id 查本包内置 PNG 渲染。
/// 此文件是 id 与资产的唯一真源——面板展示、渲染解析、降级判断都只读这里;新增
/// /删除贴纸只改这里 + `assets/stickers/fluent3d/` 目录,由 `sticker_pack_test`
/// 反核对"清单 ⇔ 磁盘 png 一一对应、零孤儿"。
///
/// 素材来自 microsoft/fluentui-emoji(MIT),3D 风格,256×256 PNG,已内置分发。
library;

/// 贴纸分类(面板分组用,顺序即分组展示顺序)。
enum StickerCategory { smileys, gestures, hearts, celebration }

/// 单个内置贴纸。id 同时是资产文件名(不含扩展名)。
class StickerItem {
  const StickerItem(this.id, this.category);

  final String id;
  final StickerCategory category;
}

/// Fluent 3D 内置贴纸包。
class StickerPack {
  StickerPack._();

  /// 唯一内置包 id。载荷的 `pack_id` 与此比对,不匹配即渲染降级。
  static const String packId = 'fluent3d';

  static const String _assetDir = 'assets/stickers/fluent3d';

  /// 清单单源。**列表顺序即面板内展示顺序**;分类字段仅用于分组标题。
  static const List<StickerItem> items = [
    // smileys (16)
    StickerItem('grinning_face', StickerCategory.smileys),
    StickerItem('face_with_tears_of_joy', StickerCategory.smileys),
    StickerItem('rolling_on_the_floor_laughing', StickerCategory.smileys),
    StickerItem('smiling_face_with_heart_eyes', StickerCategory.smileys),
    StickerItem('smiling_face_with_sunglasses', StickerCategory.smileys),
    StickerItem('winking_face', StickerCategory.smileys),
    StickerItem('face_blowing_a_kiss', StickerCategory.smileys),
    StickerItem('thinking_face', StickerCategory.smileys),
    StickerItem('face_with_rolling_eyes', StickerCategory.smileys),
    StickerItem('loudly_crying_face', StickerCategory.smileys),
    StickerItem('pleading_face', StickerCategory.smileys),
    StickerItem('star_struck', StickerCategory.smileys),
    StickerItem('partying_face', StickerCategory.smileys),
    StickerItem('woozy_face', StickerCategory.smileys),
    StickerItem('smiling_face_with_halo', StickerCategory.smileys),
    StickerItem('zany_face', StickerCategory.smileys),
    // gestures (12)
    StickerItem('thumbs_up', StickerCategory.gestures),
    StickerItem('thumbs_down', StickerCategory.gestures),
    StickerItem('waving_hand', StickerCategory.gestures),
    StickerItem('folded_hands', StickerCategory.gestures),
    StickerItem('clapping_hands', StickerCategory.gestures),
    StickerItem('victory_hand', StickerCategory.gestures),
    StickerItem('raising_hands', StickerCategory.gestures),
    StickerItem('flexed_biceps', StickerCategory.gestures),
    StickerItem('backhand_index_pointing_up', StickerCategory.gestures),
    StickerItem('call_me_hand', StickerCategory.gestures),
    StickerItem('raised_fist', StickerCategory.gestures),
    StickerItem('ok_hand', StickerCategory.gestures),
    // hearts (10)
    StickerItem('red_heart', StickerCategory.hearts),
    StickerItem('sparkling_heart', StickerCategory.hearts),
    StickerItem('two_hearts', StickerCategory.hearts),
    StickerItem('broken_heart', StickerCategory.hearts),
    StickerItem('heart_with_arrow', StickerCategory.hearts),
    StickerItem('growing_heart', StickerCategory.hearts),
    StickerItem('beating_heart', StickerCategory.hearts),
    StickerItem('blue_heart', StickerCategory.hearts),
    StickerItem('purple_heart', StickerCategory.hearts),
    StickerItem('revolving_hearts', StickerCategory.hearts),
    // celebration (10)
    StickerItem('party_popper', StickerCategory.celebration),
    StickerItem('confetti_ball', StickerCategory.celebration),
    StickerItem('balloon', StickerCategory.celebration),
    StickerItem('birthday_cake', StickerCategory.celebration),
    StickerItem('wrapped_gift', StickerCategory.celebration),
    StickerItem('trophy', StickerCategory.celebration),
    StickerItem('hundred_points', StickerCategory.celebration),
    StickerItem('rocket', StickerCategory.celebration),
    StickerItem('sparkles', StickerCategory.celebration),
    StickerItem('fire', StickerCategory.celebration),
  ];

  static final Set<String> _ids = {for (final item in items) item.id};

  /// id → 资产路径,供 `Image.asset` 渲染与面板缩略。
  static String assetPath(String stickerId) => '$_assetDir/$stickerId.png';

  /// `(packId, stickerId)` 是否为本包已知贴纸;未知则渲染端降级为占位。
  static bool isKnown({required String packId, required String stickerId}) =>
      packId == StickerPack.packId && _ids.contains(stickerId);

  /// 按分类分组(面板用),保持 [items] 内的相对顺序。
  static Map<StickerCategory, List<StickerItem>> grouped() {
    final map = <StickerCategory, List<StickerItem>>{};
    for (final item in items) {
      (map[item.category] ??= <StickerItem>[]).add(item);
    }
    return map;
  }
}
