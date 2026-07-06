/// 广场前端模型。
///
/// 广场内容正文和媒体只进入 Worker/R2；链上只记录发布索引、内容哈希和
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
  });

  final String ownerAccount;
  final String? cidNumber;
  final String? displayName;

  bool get isCertified => cidNumber != null && cidNumber!.isNotEmpty;

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
  });

  final SquareMediaKind mediaKind;
  final String url;
  final String? coverUrl;
  final int? byteSize;
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
    this.mediaItems = const <SquareMediaItem>[],
    this.contentHash,
    this.storageReceiptId,
    this.chainBlock,
  });

  final String postId;
  final SquareAuthor author;
  final SquarePostCategory postCategory;
  final String text;
  final DateTime createdAt;
  final List<SquareMediaItem> mediaItems;
  final String? contentHash;
  final String? storageReceiptId;
  final int? chainBlock;
}
