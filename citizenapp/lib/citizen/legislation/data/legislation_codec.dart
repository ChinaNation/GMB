// 立法法律 SCALE 镜像解码(ADR-027 / ADR-028 P3)——单一源,浏览页与(将来 P4)
// 条款编辑器共用,绝不另写第二套(链端布局一改两处必裂)。
//
// 中文注释:链端 `LegislationApi` 故意返回 SCALE 字节(`Option<Vec<u8>>` / `Vec<u64>`),
// 客户端镜像解码。布局逐字段对齐 legislation-yuan(多字节 LE;`Option`=1 tag 字节;
// `Compact` 变长;`BoundedVec/Vec`=Compact(len)+items;`[u8;N]`=N 裸字节;
// String=`BoundedVec<u8>`=Compact(len)+utf8)。

import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/legislation/data/law_models.dart';

/// 游标式 SCALE 读取器(内部)。
class _Scale {
  _Scale(this.data);
  final Uint8List data;
  int _i = 0;

  int u8() => data[_i++];

  int u32() {
    final v = data[_i] |
        (data[_i + 1] << 8) |
        (data[_i + 2] << 16) |
        (data[_i + 3] << 24);
    _i += 4;
    return v;
  }

  int u64() {
    var v = 0;
    for (var k = 0; k < 8; k++) {
      v |= data[_i + k] << (8 * k);
    }
    _i += 8;
    return v;
  }

  /// SCALE Compact<u32>(长度/小整数用)。
  int compact() {
    final b0 = data[_i];
    final mode = b0 & 0x03;
    if (mode == 0) {
      _i += 1;
      return b0 >> 2;
    }
    if (mode == 1) {
      final v = (data[_i] | (data[_i + 1] << 8)) >> 2;
      _i += 2;
      return v;
    }
    if (mode == 2) {
      final v = (data[_i] |
              (data[_i + 1] << 8) |
              (data[_i + 2] << 16) |
              (data[_i + 3] << 24)) >>
          2;
      _i += 4;
      return v;
    }
    // mode 3:big-integer,low6+4 = 字节数(长度极少命中此分支)。
    final n = (b0 >> 2) + 4;
    _i += 1;
    var v = 0;
    for (var k = 0; k < n; k++) {
      v |= data[_i + k] << (8 * k);
    }
    _i += n;
    return v;
  }

  Uint8List bytes(int n) {
    final b = data.sublist(_i, _i + n);
    _i += n;
    return b;
  }

  /// Compact(len)+len 字节 → UTF-8 字符串。
  String boundedString() => utf8.decode(bytes(compact()), allowMalformed: true);

  /// N 裸字节 → 小写 hex(不含 0x)。
  String hex(int n) =>
      bytes(n).map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  /// `Option<T>`:1 tag 字节(0=None / 1=Some)。
  T? option<T>(T Function() some) => u8() == 0 ? null : some();

  /// `Vec<T>`/`BoundedVec<T>`:Compact(len)+items。
  List<T> vec<T>(T Function() item) {
    final n = compact();
    return [for (var k = 0; k < n; k++) item()];
  }
}

/// `[u8;4]` 机构码 → 字符串(去尾部 0)。
String _codeString(Uint8List b) {
  var end = b.length;
  while (end > 0 && b[end - 1] == 0) {
    end--;
  }
  return utf8.decode(b.sublist(0, end), allowMalformed: true);
}

/// 解 `Option<Vec<u8>>`(API 边界外层)→ 内层 SCALE 字节(None 返回 null)。
Uint8List? decodeOptionBytes(Uint8List raw) {
  final s = _Scale(raw);
  if (s.u8() == 0) return null;
  return s.bytes(s.compact());
}

/// 解 `list_laws` 返回的 `Vec<u64>`。
List<int> decodeLawIds(Uint8List raw) {
  final s = _Scale(raw);
  return s.vec(s.u64);
}

/// 解 `Law`(内层字节,调用方先 [decodeOptionBytes] 拆 Option)。
Law decodeLaw(Uint8List raw) {
  final s = _Scale(raw);
  final lawId = s.u64();
  final tier = LawTier.fromIndex(s.u8());
  final scopeCode = s.u32();
  final houses = s.vec(() => LawHouse(
        institutionCode: _codeString(s.bytes(4)),
        accountHex: s.hex(32),
      ));
  final effectiveVersion = s.option(s.u32);
  final latestVersion = s.u32();
  final pendingVersion = s.option(s.u32);
  final status = LawStatus.fromIndex(s.u8());
  return Law(
    lawId: lawId,
    tier: tier,
    scopeCode: scopeCode,
    houses: houses,
    effectiveVersion: effectiveVersion,
    latestVersion: latestVersion,
    pendingVersion: pendingVersion,
    status: status,
  );
}

/// 解 `LawVersion`(内层字节)。
LawVersion decodeLawVersion(Uint8List raw) {
  final s = _Scale(raw);
  final lawId = s.u64();
  final version = s.u32();
  final title = s.boundedString();
  final titleEn = s.option(s.boundedString);
  final chapters = s.vec(() => _chapter(s));
  final contentHash = s.hex(32);
  final voteType = s.u8();
  final proposalId = s.u64();
  final publishedAt = s.u64();
  final effectiveAt = s.u64();
  return LawVersion(
    lawId: lawId,
    version: version,
    title: title,
    titleEn: titleEn,
    chapters: chapters,
    contentHash: contentHash,
    voteType: voteType,
    proposalId: proposalId,
    publishedAt: publishedAt,
    effectiveAt: effectiveAt,
  );
}

/// 解 `ConstitutionImmutableManifest`(StorageValue 原始字节)。
ImmutableManifest decodeImmutableManifest(Uint8List raw) {
  final s = _Scale(raw);
  final numbers = s.vec(s.u32);
  final hashes = s.vec(() => s.hex(32));
  return ImmutableManifest(articleNumbers: numbers, articleHashes: hashes);
}

LawChapter _chapter(_Scale s) {
  final number = s.u32();
  final title = s.boundedString();
  final titleEn = s.option(s.boundedString);
  final sections = s.vec(() => _section(s));
  return LawChapter(
      number: number, title: title, titleEn: titleEn, sections: sections);
}

LawSection _section(_Scale s) {
  final number = s.u32();
  final title = s.boundedString();
  final titleEn = s.option(s.boundedString);
  final articles = s.vec(() => _article(s));
  return LawSection(
      number: number, title: title, titleEn: titleEn, articles: articles);
}

LawArticle _article(_Scale s) {
  final number = s.u32();
  final title = s.boundedString();
  final titleEn = s.option(s.boundedString);
  final body = s.boundedString();
  final bodyEn = s.option(s.boundedString);
  final clauses = s.vec(() => _clause(s));
  return LawArticle(
    number: number,
    title: title,
    titleEn: titleEn,
    body: body,
    bodyEn: bodyEn,
    clauses: clauses,
  );
}

LawClause _clause(_Scale s) {
  final number = s.u32();
  final text = s.boundedString();
  final textEn = s.option(s.boundedString);
  return LawClause(number: number, text: text, textEn: textEn);
}
