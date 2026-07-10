/// 广场前端模型。
///
/// 广场正文和 manifest 进入 Worker/R2，主媒体进入 Cloudflare Images / Stream；链上只记录发布索引、内容哈希和
/// 存储回执，App 本地模型也按这条边界拆分展示态、上传态和发布态。
enum SquareFeedKind {
  recommended('推荐', 'recommended'),
  following('关注', 'following'),
  campaign('竞选', 'campaign');

  const SquareFeedKind(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

enum SquarePostCategory {
  normal('普通', 'normal'),
  campaign('竞选', 'campaign');

  const SquarePostCategory(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

/// 内容形态：普通短动态 vs 长文文章。与链上 post_category 正交，只落链下。
enum SquarePostContentFormat {
  normal('动态', 'normal'),
  article('文章', 'article');

  const SquarePostContentFormat(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

enum SquareMediaKind {
  image('图片', 'image'),
  video('视频', 'video');

  const SquareMediaKind(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

enum SquarePublishStage {
  idle('待发布'),
  checkingBalance('校验余额'),
  signingIn('钱包登录'),
  preparingStorage('准备存储'),
  submittingChain('扣费入块'),
  waitingInBlock('等待入块'),
  uploadingMedia('上传媒体'),
  completingStorage('确认存储'),
  confirmingPost('发布可见'),
  completed('已发布');

  const SquarePublishStage(this.label);

  final String label;
}

class SquareAuthor {
  const SquareAuthor({
    required this.ownerAccount,
    this.cidNumber,
    this.displayName,
    this.identityLevel,
    this.membershipLevel,
    this.membershipActive = false,
  });

  final String ownerAccount;
  final String? cidNumber;
  final String? displayName;

  /// 作者链上身份档（徽章颜色）：visitor/voting/candidate/null。
  final String? identityLevel;

  /// 作者已购买会员档（徽章勾）：visitor/voting/candidate/null。
  final String? membershipLevel;

  /// 作者会员是否有效。
  final bool membershipActive;

  bool get isCertified {
    final level = identityLevel;
    if (level != null) return level != 'visitor';
    return cidNumber != null && cidNumber!.isNotEmpty;
  }

  String get title {
    final name = displayName;
    if (name != null && name.isNotEmpty) return name;
    if (ownerAccount.length <= 12) return ownerAccount;
    return '${ownerAccount.substring(0, 6)}...${ownerAccount.substring(ownerAccount.length - 6)}';
  }
}

class SquareMediaItem {
  const SquareMediaItem({
    required this.mediaKind,
    required this.url,
    this.coverUrl,
    this.byteSize,
    this.assetState,
    this.archiveState,
  });

  final SquareMediaKind mediaKind;
  final String url;
  final String? coverUrl;
  final int? byteSize;
  final String? assetState;
  // 视频冷归档态：'archived'=已归档不可播（作者未续订），'restoring'=恢复中；null/'live'=正常。
  final String? archiveState;

  bool get isArchived => archiveState == 'archived';
  bool get isRestoring => archiveState == 'restoring';
}

class SquareLocalMediaDraft {
  const SquareLocalMediaDraft({
    required this.mediaKind,
    required this.path,
    required this.fileName,
    required this.contentType,
    required this.byteSize,
  });

  final SquareMediaKind mediaKind;
  final String path;
  final String fileName;
  final String contentType;
  final int byteSize;

  String get fileExt {
    final dot = fileName.lastIndexOf('.');
    if (dot < 0 || dot == fileName.length - 1) return '';
    return fileName.substring(dot + 1).toLowerCase();
  }
}

class SquarePost {
  const SquarePost({
    required this.postId,
    required this.author,
    required this.postCategory,
    required this.text,
    required this.createdAt,
    this.contentFormat = SquarePostContentFormat.normal,
    this.title,
    this.mediaItems = const <SquareMediaItem>[],
    this.contentHash,
    this.storageReceiptId,
    this.chainBlock,
    this.campaignInstitutionCid,
    this.campaignPosition,
  });

  final String postId;
  final SquareAuthor author;
  final SquarePostCategory postCategory;

  /// 内容形态（普通/文章）。文章为长文，另带标题。
  final SquarePostContentFormat contentFormat;

  /// 文章标题；普通动态为空。
  final String? title;

  final String text;
  final DateTime createdAt;
  final List<SquareMediaItem> mediaItems;
  final String? contentHash;
  final String? storageReceiptId;
  final int? chainBlock;

  // 竞选目标（预留，待公民身份上链完成后填充与校验）：竞选哪个机构的哪个岗位。
  // 公民 CID 复用 author.cidNumber；下面两项当前不生成、不校验、不入 UI。

  /// 竞选目标机构 CID（预留）。
  final String? campaignInstitutionCid;

  /// 竞选目标岗位（预留）。
  final String? campaignPosition;
}
