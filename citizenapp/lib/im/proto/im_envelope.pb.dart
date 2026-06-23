// This is a generated file - do not edit.
//
// Generated from im_envelope.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'im_envelope.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'im_envelope.pbenum.dart';

class ImNodeEndpoint extends $pb.GeneratedMessage {
  factory ImNodeEndpoint({
    $core.String? peerId,
    $core.String? multiaddr,
    $core.String? kind,
  }) {
    final result = create();
    if (peerId != null) result.peerId = peerId;
    if (multiaddr != null) result.multiaddr = multiaddr;
    if (kind != null) result.kind = kind;
    return result;
  }

  ImNodeEndpoint._();

  factory ImNodeEndpoint.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImNodeEndpoint.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImNodeEndpoint',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'peerId')
    ..aOS(2, _omitFieldNames ? '' : 'multiaddr')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImNodeEndpoint clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImNodeEndpoint copyWith(void Function(ImNodeEndpoint) updates) =>
      super.copyWith((message) => updates(message as ImNodeEndpoint))
          as ImNodeEndpoint;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImNodeEndpoint create() => ImNodeEndpoint._();
  @$core.override
  ImNodeEndpoint createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImNodeEndpoint getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImNodeEndpoint>(create);
  static ImNodeEndpoint? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get peerId => $_getSZ(0);
  @$pb.TagNumber(1)
  set peerId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPeerId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPeerId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get multiaddr => $_getSZ(1);
  @$pb.TagNumber(2)
  set multiaddr($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMultiaddr() => $_has(1);
  @$pb.TagNumber(2)
  void clearMultiaddr() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);
}

class ImRouteRecord extends $pb.GeneratedMessage {
  factory ImRouteRecord({
    $core.int? protocolVersion,
    $core.String? walletChatAccount,
    $core.String? routeDisplayName,
    $core.String? imDeviceId,
    $core.String? imDevicePubkeyHex,
    $core.String? safetyNumber,
    $core.Iterable<ImNodeEndpoint>? nodeEndpoints,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (walletChatAccount != null) result.walletChatAccount = walletChatAccount;
    if (routeDisplayName != null) result.routeDisplayName = routeDisplayName;
    if (imDeviceId != null) result.imDeviceId = imDeviceId;
    if (imDevicePubkeyHex != null) result.imDevicePubkeyHex = imDevicePubkeyHex;
    if (safetyNumber != null) result.safetyNumber = safetyNumber;
    if (nodeEndpoints != null) result.nodeEndpoints.addAll(nodeEndpoints);
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (expiresAtMillis != null) result.expiresAtMillis = expiresAtMillis;
    return result;
  }

  ImRouteRecord._();

  factory ImRouteRecord.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImRouteRecord.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImRouteRecord',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'walletChatAccount')
    ..aOS(3, _omitFieldNames ? '' : 'routeDisplayName')
    ..aOS(4, _omitFieldNames ? '' : 'imDeviceId')
    ..aOS(5, _omitFieldNames ? '' : 'imDevicePubkeyHex')
    ..aOS(6, _omitFieldNames ? '' : 'safetyNumber')
    ..pPM<ImNodeEndpoint>(7, _omitFieldNames ? '' : 'nodeEndpoints',
        subBuilder: ImNodeEndpoint.create)
    ..a<$fixnum.Int64>(
        8, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'expiresAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImRouteRecord clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImRouteRecord copyWith(void Function(ImRouteRecord) updates) =>
      super.copyWith((message) => updates(message as ImRouteRecord))
          as ImRouteRecord;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImRouteRecord create() => ImRouteRecord._();
  @$core.override
  ImRouteRecord createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImRouteRecord getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImRouteRecord>(create);
  static ImRouteRecord? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get protocolVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set protocolVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get walletChatAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set walletChatAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWalletChatAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearWalletChatAccount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get routeDisplayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set routeDisplayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRouteDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearRouteDisplayName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get imDeviceId => $_getSZ(3);
  @$pb.TagNumber(4)
  set imDeviceId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasImDeviceId() => $_has(3);
  @$pb.TagNumber(4)
  void clearImDeviceId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get imDevicePubkeyHex => $_getSZ(4);
  @$pb.TagNumber(5)
  set imDevicePubkeyHex($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasImDevicePubkeyHex() => $_has(4);
  @$pb.TagNumber(5)
  void clearImDevicePubkeyHex() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get safetyNumber => $_getSZ(5);
  @$pb.TagNumber(6)
  set safetyNumber($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSafetyNumber() => $_has(5);
  @$pb.TagNumber(6)
  void clearSafetyNumber() => $_clearField(6);

  @$pb.TagNumber(7)
  $pb.PbList<ImNodeEndpoint> get nodeEndpoints => $_getList(6);

  @$pb.TagNumber(8)
  $fixnum.Int64 get createdAtMillis => $_getI64(7);
  @$pb.TagNumber(8)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAtMillis() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAtMillis() => $_clearField(8);

  @$pb.TagNumber(9)
  $fixnum.Int64 get expiresAtMillis => $_getI64(8);
  @$pb.TagNumber(9)
  set expiresAtMillis($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasExpiresAtMillis() => $_has(8);
  @$pb.TagNumber(9)
  void clearExpiresAtMillis() => $_clearField(9);
}

class ImEnvelope extends $pb.GeneratedMessage {
  factory ImEnvelope({
    $core.int? protocolVersion,
    $core.String? envelopeId,
    $core.String? conversationId,
    $core.String? senderChatAccount,
    $core.String? recipientChatAccount,
    $core.String? senderDeviceId,
    $core.List<$core.int>? mlsWireMessage,
    $core.List<$core.int>? encryptedMetadata,
    $core.String? attachmentManifestHash,
    $core.Iterable<$core.String>? chunkRefs,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? ttlMillis,
    $core.String? ackPolicy,
    ImMlsWireMessageKind? mlsMessageKind,
    $core.List<$core.int>? ratchetTree,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (envelopeId != null) result.envelopeId = envelopeId;
    if (conversationId != null) result.conversationId = conversationId;
    if (senderChatAccount != null) result.senderChatAccount = senderChatAccount;
    if (recipientChatAccount != null)
      result.recipientChatAccount = recipientChatAccount;
    if (senderDeviceId != null) result.senderDeviceId = senderDeviceId;
    if (mlsWireMessage != null) result.mlsWireMessage = mlsWireMessage;
    if (encryptedMetadata != null) result.encryptedMetadata = encryptedMetadata;
    if (attachmentManifestHash != null)
      result.attachmentManifestHash = attachmentManifestHash;
    if (chunkRefs != null) result.chunkRefs.addAll(chunkRefs);
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (ttlMillis != null) result.ttlMillis = ttlMillis;
    if (ackPolicy != null) result.ackPolicy = ackPolicy;
    if (mlsMessageKind != null) result.mlsMessageKind = mlsMessageKind;
    if (ratchetTree != null) result.ratchetTree = ratchetTree;
    return result;
  }

  ImEnvelope._();

  factory ImEnvelope.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImEnvelope.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImEnvelope',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'envelopeId')
    ..aOS(3, _omitFieldNames ? '' : 'conversationId')
    ..aOS(4, _omitFieldNames ? '' : 'senderChatAccount')
    ..aOS(5, _omitFieldNames ? '' : 'recipientChatAccount')
    ..aOS(6, _omitFieldNames ? '' : 'senderDeviceId')
    ..a<$core.List<$core.int>>(
        7, _omitFieldNames ? '' : 'mlsWireMessage', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        8, _omitFieldNames ? '' : 'encryptedMetadata', $pb.PbFieldType.OY)
    ..aOS(9, _omitFieldNames ? '' : 'attachmentManifestHash')
    ..pPS(10, _omitFieldNames ? '' : 'chunkRefs')
    ..a<$fixnum.Int64>(
        11, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        12, _omitFieldNames ? '' : 'ttlMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(13, _omitFieldNames ? '' : 'ackPolicy')
    ..aE<ImMlsWireMessageKind>(14, _omitFieldNames ? '' : 'mlsMessageKind',
        enumValues: ImMlsWireMessageKind.values)
    ..a<$core.List<$core.int>>(
        15, _omitFieldNames ? '' : 'ratchetTree', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImEnvelope clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImEnvelope copyWith(void Function(ImEnvelope) updates) =>
      super.copyWith((message) => updates(message as ImEnvelope)) as ImEnvelope;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImEnvelope create() => ImEnvelope._();
  @$core.override
  ImEnvelope createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImEnvelope getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImEnvelope>(create);
  static ImEnvelope? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get protocolVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set protocolVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get envelopeId => $_getSZ(1);
  @$pb.TagNumber(2)
  set envelopeId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnvelopeId() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnvelopeId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get conversationId => $_getSZ(2);
  @$pb.TagNumber(3)
  set conversationId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasConversationId() => $_has(2);
  @$pb.TagNumber(3)
  void clearConversationId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get senderChatAccount => $_getSZ(3);
  @$pb.TagNumber(4)
  set senderChatAccount($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSenderChatAccount() => $_has(3);
  @$pb.TagNumber(4)
  void clearSenderChatAccount() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get recipientChatAccount => $_getSZ(4);
  @$pb.TagNumber(5)
  set recipientChatAccount($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRecipientChatAccount() => $_has(4);
  @$pb.TagNumber(5)
  void clearRecipientChatAccount() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get senderDeviceId => $_getSZ(5);
  @$pb.TagNumber(6)
  set senderDeviceId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSenderDeviceId() => $_has(5);
  @$pb.TagNumber(6)
  void clearSenderDeviceId() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.List<$core.int> get mlsWireMessage => $_getN(6);
  @$pb.TagNumber(7)
  set mlsWireMessage($core.List<$core.int> value) => $_setBytes(6, value);
  @$pb.TagNumber(7)
  $core.bool hasMlsWireMessage() => $_has(6);
  @$pb.TagNumber(7)
  void clearMlsWireMessage() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.List<$core.int> get encryptedMetadata => $_getN(7);
  @$pb.TagNumber(8)
  set encryptedMetadata($core.List<$core.int> value) => $_setBytes(7, value);
  @$pb.TagNumber(8)
  $core.bool hasEncryptedMetadata() => $_has(7);
  @$pb.TagNumber(8)
  void clearEncryptedMetadata() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get attachmentManifestHash => $_getSZ(8);
  @$pb.TagNumber(9)
  set attachmentManifestHash($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasAttachmentManifestHash() => $_has(8);
  @$pb.TagNumber(9)
  void clearAttachmentManifestHash() => $_clearField(9);

  @$pb.TagNumber(10)
  $pb.PbList<$core.String> get chunkRefs => $_getList(9);

  @$pb.TagNumber(11)
  $fixnum.Int64 get createdAtMillis => $_getI64(10);
  @$pb.TagNumber(11)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCreatedAtMillis() => $_has(10);
  @$pb.TagNumber(11)
  void clearCreatedAtMillis() => $_clearField(11);

  @$pb.TagNumber(12)
  $fixnum.Int64 get ttlMillis => $_getI64(11);
  @$pb.TagNumber(12)
  set ttlMillis($fixnum.Int64 value) => $_setInt64(11, value);
  @$pb.TagNumber(12)
  $core.bool hasTtlMillis() => $_has(11);
  @$pb.TagNumber(12)
  void clearTtlMillis() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.String get ackPolicy => $_getSZ(12);
  @$pb.TagNumber(13)
  set ackPolicy($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasAckPolicy() => $_has(12);
  @$pb.TagNumber(13)
  void clearAckPolicy() => $_clearField(13);

  @$pb.TagNumber(14)
  ImMlsWireMessageKind get mlsMessageKind => $_getN(13);
  @$pb.TagNumber(14)
  set mlsMessageKind(ImMlsWireMessageKind value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasMlsMessageKind() => $_has(13);
  @$pb.TagNumber(14)
  void clearMlsMessageKind() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.List<$core.int> get ratchetTree => $_getN(14);
  @$pb.TagNumber(15)
  set ratchetTree($core.List<$core.int> value) => $_setBytes(14, value);
  @$pb.TagNumber(15)
  $core.bool hasRatchetTree() => $_has(14);
  @$pb.TagNumber(15)
  void clearRatchetTree() => $_clearField(15);
}

class ImEnvelopeAck extends $pb.GeneratedMessage {
  factory ImEnvelopeAck({
    $core.String? envelopeId,
    $core.String? state,
  }) {
    final result = create();
    if (envelopeId != null) result.envelopeId = envelopeId;
    if (state != null) result.state = state;
    return result;
  }

  ImEnvelopeAck._();

  factory ImEnvelopeAck.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImEnvelopeAck.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImEnvelopeAck',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'envelopeId')
    ..aOS(2, _omitFieldNames ? '' : 'state')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImEnvelopeAck clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImEnvelopeAck copyWith(void Function(ImEnvelopeAck) updates) =>
      super.copyWith((message) => updates(message as ImEnvelopeAck))
          as ImEnvelopeAck;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImEnvelopeAck create() => ImEnvelopeAck._();
  @$core.override
  ImEnvelopeAck createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImEnvelopeAck getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImEnvelopeAck>(create);
  static ImEnvelopeAck? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get envelopeId => $_getSZ(0);
  @$pb.TagNumber(1)
  set envelopeId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEnvelopeId() => $_has(0);
  @$pb.TagNumber(1)
  void clearEnvelopeId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get state => $_getSZ(1);
  @$pb.TagNumber(2)
  set state($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasState() => $_has(1);
  @$pb.TagNumber(2)
  void clearState() => $_clearField(2);
}

class ImKeyPackage extends $pb.GeneratedMessage {
  factory ImKeyPackage({
    $core.int? protocolVersion,
    $core.String? ownerWalletAccount,
    $core.String? deviceId,
    $core.String? devicePublicKeyHex,
    $core.String? keyPackageId,
    $core.List<$core.int>? keyPackage,
    $core.String? cipherSuite,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
    $fixnum.Int64? consumedAtMillis,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (ownerWalletAccount != null)
      result.ownerWalletAccount = ownerWalletAccount;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKeyHex != null)
      result.devicePublicKeyHex = devicePublicKeyHex;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (keyPackage != null) result.keyPackage = keyPackage;
    if (cipherSuite != null) result.cipherSuite = cipherSuite;
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (expiresAtMillis != null) result.expiresAtMillis = expiresAtMillis;
    if (consumedAtMillis != null) result.consumedAtMillis = consumedAtMillis;
    return result;
  }

  ImKeyPackage._();

  factory ImKeyPackage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImKeyPackage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImKeyPackage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'ownerWalletAccount')
    ..aOS(3, _omitFieldNames ? '' : 'deviceId')
    ..aOS(4, _omitFieldNames ? '' : 'devicePublicKeyHex')
    ..aOS(5, _omitFieldNames ? '' : 'keyPackageId')
    ..a<$core.List<$core.int>>(
        6, _omitFieldNames ? '' : 'keyPackage', $pb.PbFieldType.OY)
    ..aOS(7, _omitFieldNames ? '' : 'cipherSuite')
    ..a<$fixnum.Int64>(
        8, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'expiresAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        10, _omitFieldNames ? '' : 'consumedAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImKeyPackage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImKeyPackage copyWith(void Function(ImKeyPackage) updates) =>
      super.copyWith((message) => updates(message as ImKeyPackage))
          as ImKeyPackage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImKeyPackage create() => ImKeyPackage._();
  @$core.override
  ImKeyPackage createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImKeyPackage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImKeyPackage>(create);
  static ImKeyPackage? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get protocolVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set protocolVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ownerWalletAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set ownerWalletAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOwnerWalletAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearOwnerWalletAccount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get deviceId => $_getSZ(2);
  @$pb.TagNumber(3)
  set deviceId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDeviceId() => $_has(2);
  @$pb.TagNumber(3)
  void clearDeviceId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get devicePublicKeyHex => $_getSZ(3);
  @$pb.TagNumber(4)
  set devicePublicKeyHex($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDevicePublicKeyHex() => $_has(3);
  @$pb.TagNumber(4)
  void clearDevicePublicKeyHex() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get keyPackageId => $_getSZ(4);
  @$pb.TagNumber(5)
  set keyPackageId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasKeyPackageId() => $_has(4);
  @$pb.TagNumber(5)
  void clearKeyPackageId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.List<$core.int> get keyPackage => $_getN(5);
  @$pb.TagNumber(6)
  set keyPackage($core.List<$core.int> value) => $_setBytes(5, value);
  @$pb.TagNumber(6)
  $core.bool hasKeyPackage() => $_has(5);
  @$pb.TagNumber(6)
  void clearKeyPackage() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get cipherSuite => $_getSZ(6);
  @$pb.TagNumber(7)
  set cipherSuite($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCipherSuite() => $_has(6);
  @$pb.TagNumber(7)
  void clearCipherSuite() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get createdAtMillis => $_getI64(7);
  @$pb.TagNumber(8)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAtMillis() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAtMillis() => $_clearField(8);

  @$pb.TagNumber(9)
  $fixnum.Int64 get expiresAtMillis => $_getI64(8);
  @$pb.TagNumber(9)
  set expiresAtMillis($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasExpiresAtMillis() => $_has(8);
  @$pb.TagNumber(9)
  void clearExpiresAtMillis() => $_clearField(9);

  @$pb.TagNumber(10)
  $fixnum.Int64 get consumedAtMillis => $_getI64(9);
  @$pb.TagNumber(10)
  set consumedAtMillis($fixnum.Int64 value) => $_setInt64(9, value);
  @$pb.TagNumber(10)
  $core.bool hasConsumedAtMillis() => $_has(9);
  @$pb.TagNumber(10)
  void clearConsumedAtMillis() => $_clearField(10);
}

class PublishImKeyPackageRequest extends $pb.GeneratedMessage {
  factory PublishImKeyPackageRequest({
    $core.String? ownerWalletAccount,
    $core.String? deviceId,
    $core.String? devicePublicKeyHex,
    $core.String? keyPackageId,
    $core.List<$core.int>? keyPackage,
    $core.String? cipherSuite,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (ownerWalletAccount != null)
      result.ownerWalletAccount = ownerWalletAccount;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKeyHex != null)
      result.devicePublicKeyHex = devicePublicKeyHex;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (keyPackage != null) result.keyPackage = keyPackage;
    if (cipherSuite != null) result.cipherSuite = cipherSuite;
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (expiresAtMillis != null) result.expiresAtMillis = expiresAtMillis;
    return result;
  }

  PublishImKeyPackageRequest._();

  factory PublishImKeyPackageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PublishImKeyPackageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PublishImKeyPackageRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerWalletAccount')
    ..aOS(2, _omitFieldNames ? '' : 'deviceId')
    ..aOS(3, _omitFieldNames ? '' : 'devicePublicKeyHex')
    ..aOS(4, _omitFieldNames ? '' : 'keyPackageId')
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'keyPackage', $pb.PbFieldType.OY)
    ..aOS(6, _omitFieldNames ? '' : 'cipherSuite')
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        8, _omitFieldNames ? '' : 'expiresAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishImKeyPackageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishImKeyPackageRequest copyWith(
          void Function(PublishImKeyPackageRequest) updates) =>
      super.copyWith(
              (message) => updates(message as PublishImKeyPackageRequest))
          as PublishImKeyPackageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishImKeyPackageRequest create() => PublishImKeyPackageRequest._();
  @$core.override
  PublishImKeyPackageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PublishImKeyPackageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PublishImKeyPackageRequest>(create);
  static PublishImKeyPackageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerWalletAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerWalletAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerWalletAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerWalletAccount() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get deviceId => $_getSZ(1);
  @$pb.TagNumber(2)
  set deviceId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDeviceId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDeviceId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get devicePublicKeyHex => $_getSZ(2);
  @$pb.TagNumber(3)
  set devicePublicKeyHex($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDevicePublicKeyHex() => $_has(2);
  @$pb.TagNumber(3)
  void clearDevicePublicKeyHex() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get keyPackageId => $_getSZ(3);
  @$pb.TagNumber(4)
  set keyPackageId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasKeyPackageId() => $_has(3);
  @$pb.TagNumber(4)
  void clearKeyPackageId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get keyPackage => $_getN(4);
  @$pb.TagNumber(5)
  set keyPackage($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasKeyPackage() => $_has(4);
  @$pb.TagNumber(5)
  void clearKeyPackage() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get cipherSuite => $_getSZ(5);
  @$pb.TagNumber(6)
  set cipherSuite($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCipherSuite() => $_has(5);
  @$pb.TagNumber(6)
  void clearCipherSuite() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get createdAtMillis => $_getI64(6);
  @$pb.TagNumber(7)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedAtMillis() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedAtMillis() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get expiresAtMillis => $_getI64(7);
  @$pb.TagNumber(8)
  set expiresAtMillis($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasExpiresAtMillis() => $_has(7);
  @$pb.TagNumber(8)
  void clearExpiresAtMillis() => $_clearField(8);
}

class FetchImKeyPackagesRequest extends $pb.GeneratedMessage {
  factory FetchImKeyPackagesRequest({
    $core.String? ownerWalletAccount,
    $core.String? requesterChatAccount,
    $core.int? limit,
  }) {
    final result = create();
    if (ownerWalletAccount != null)
      result.ownerWalletAccount = ownerWalletAccount;
    if (requesterChatAccount != null)
      result.requesterChatAccount = requesterChatAccount;
    if (limit != null) result.limit = limit;
    return result;
  }

  FetchImKeyPackagesRequest._();

  factory FetchImKeyPackagesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FetchImKeyPackagesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FetchImKeyPackagesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerWalletAccount')
    ..aOS(2, _omitFieldNames ? '' : 'requesterChatAccount')
    ..aI(3, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FetchImKeyPackagesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FetchImKeyPackagesRequest copyWith(
          void Function(FetchImKeyPackagesRequest) updates) =>
      super.copyWith((message) => updates(message as FetchImKeyPackagesRequest))
          as FetchImKeyPackagesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FetchImKeyPackagesRequest create() => FetchImKeyPackagesRequest._();
  @$core.override
  FetchImKeyPackagesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FetchImKeyPackagesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FetchImKeyPackagesRequest>(create);
  static FetchImKeyPackagesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerWalletAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerWalletAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerWalletAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerWalletAccount() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get requesterChatAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set requesterChatAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRequesterChatAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearRequesterChatAccount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get limit => $_getIZ(2);
  @$pb.TagNumber(3)
  set limit($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);
}

class ConsumeImKeyPackageRequest extends $pb.GeneratedMessage {
  factory ConsumeImKeyPackageRequest({
    $core.String? ownerWalletAccount,
    $core.String? keyPackageId,
    $core.String? requesterChatAccount,
  }) {
    final result = create();
    if (ownerWalletAccount != null)
      result.ownerWalletAccount = ownerWalletAccount;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (requesterChatAccount != null)
      result.requesterChatAccount = requesterChatAccount;
    return result;
  }

  ConsumeImKeyPackageRequest._();

  factory ConsumeImKeyPackageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConsumeImKeyPackageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConsumeImKeyPackageRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerWalletAccount')
    ..aOS(2, _omitFieldNames ? '' : 'keyPackageId')
    ..aOS(3, _omitFieldNames ? '' : 'requesterChatAccount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsumeImKeyPackageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsumeImKeyPackageRequest copyWith(
          void Function(ConsumeImKeyPackageRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ConsumeImKeyPackageRequest))
          as ConsumeImKeyPackageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConsumeImKeyPackageRequest create() => ConsumeImKeyPackageRequest._();
  @$core.override
  ConsumeImKeyPackageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConsumeImKeyPackageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConsumeImKeyPackageRequest>(create);
  static ConsumeImKeyPackageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerWalletAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerWalletAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerWalletAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerWalletAccount() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get keyPackageId => $_getSZ(1);
  @$pb.TagNumber(2)
  set keyPackageId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKeyPackageId() => $_has(1);
  @$pb.TagNumber(2)
  void clearKeyPackageId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get requesterChatAccount => $_getSZ(2);
  @$pb.TagNumber(3)
  set requesterChatAccount($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRequesterChatAccount() => $_has(2);
  @$pb.TagNumber(3)
  void clearRequesterChatAccount() => $_clearField(3);
}

class ImDirectDeliveryRequest extends $pb.GeneratedMessage {
  factory ImDirectDeliveryRequest({
    ImNodeEndpoint? remoteEndpoint,
    ImEnvelope? envelope,
  }) {
    final result = create();
    if (remoteEndpoint != null) result.remoteEndpoint = remoteEndpoint;
    if (envelope != null) result.envelope = envelope;
    return result;
  }

  ImDirectDeliveryRequest._();

  factory ImDirectDeliveryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImDirectDeliveryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImDirectDeliveryRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOM<ImNodeEndpoint>(1, _omitFieldNames ? '' : 'remoteEndpoint',
        subBuilder: ImNodeEndpoint.create)
    ..aOM<ImEnvelope>(2, _omitFieldNames ? '' : 'envelope',
        subBuilder: ImEnvelope.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectDeliveryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectDeliveryRequest copyWith(
          void Function(ImDirectDeliveryRequest) updates) =>
      super.copyWith((message) => updates(message as ImDirectDeliveryRequest))
          as ImDirectDeliveryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImDirectDeliveryRequest create() => ImDirectDeliveryRequest._();
  @$core.override
  ImDirectDeliveryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImDirectDeliveryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImDirectDeliveryRequest>(create);
  static ImDirectDeliveryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  ImNodeEndpoint get remoteEndpoint => $_getN(0);
  @$pb.TagNumber(1)
  set remoteEndpoint(ImNodeEndpoint value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRemoteEndpoint() => $_has(0);
  @$pb.TagNumber(1)
  void clearRemoteEndpoint() => $_clearField(1);
  @$pb.TagNumber(1)
  ImNodeEndpoint ensureRemoteEndpoint() => $_ensure(0);

  @$pb.TagNumber(2)
  ImEnvelope get envelope => $_getN(1);
  @$pb.TagNumber(2)
  set envelope(ImEnvelope value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEnvelope() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnvelope() => $_clearField(2);
  @$pb.TagNumber(2)
  ImEnvelope ensureEnvelope() => $_ensure(1);
}

class ImDirectKeyPackageFetchRequest extends $pb.GeneratedMessage {
  factory ImDirectKeyPackageFetchRequest({
    ImNodeEndpoint? remoteEndpoint,
    FetchImKeyPackagesRequest? fetch,
  }) {
    final result = create();
    if (remoteEndpoint != null) result.remoteEndpoint = remoteEndpoint;
    if (fetch != null) result.fetch = fetch;
    return result;
  }

  ImDirectKeyPackageFetchRequest._();

  factory ImDirectKeyPackageFetchRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImDirectKeyPackageFetchRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImDirectKeyPackageFetchRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOM<ImNodeEndpoint>(1, _omitFieldNames ? '' : 'remoteEndpoint',
        subBuilder: ImNodeEndpoint.create)
    ..aOM<FetchImKeyPackagesRequest>(2, _omitFieldNames ? '' : 'fetch',
        subBuilder: FetchImKeyPackagesRequest.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectKeyPackageFetchRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectKeyPackageFetchRequest copyWith(
          void Function(ImDirectKeyPackageFetchRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ImDirectKeyPackageFetchRequest))
          as ImDirectKeyPackageFetchRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImDirectKeyPackageFetchRequest create() =>
      ImDirectKeyPackageFetchRequest._();
  @$core.override
  ImDirectKeyPackageFetchRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImDirectKeyPackageFetchRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImDirectKeyPackageFetchRequest>(create);
  static ImDirectKeyPackageFetchRequest? _defaultInstance;

  @$pb.TagNumber(1)
  ImNodeEndpoint get remoteEndpoint => $_getN(0);
  @$pb.TagNumber(1)
  set remoteEndpoint(ImNodeEndpoint value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRemoteEndpoint() => $_has(0);
  @$pb.TagNumber(1)
  void clearRemoteEndpoint() => $_clearField(1);
  @$pb.TagNumber(1)
  ImNodeEndpoint ensureRemoteEndpoint() => $_ensure(0);

  @$pb.TagNumber(2)
  FetchImKeyPackagesRequest get fetch => $_getN(1);
  @$pb.TagNumber(2)
  set fetch(FetchImKeyPackagesRequest value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasFetch() => $_has(1);
  @$pb.TagNumber(2)
  void clearFetch() => $_clearField(2);
  @$pb.TagNumber(2)
  FetchImKeyPackagesRequest ensureFetch() => $_ensure(1);
}

class ImDirectKeyPackageConsumeRequest extends $pb.GeneratedMessage {
  factory ImDirectKeyPackageConsumeRequest({
    ImNodeEndpoint? remoteEndpoint,
    ConsumeImKeyPackageRequest? consume,
  }) {
    final result = create();
    if (remoteEndpoint != null) result.remoteEndpoint = remoteEndpoint;
    if (consume != null) result.consume = consume;
    return result;
  }

  ImDirectKeyPackageConsumeRequest._();

  factory ImDirectKeyPackageConsumeRequest.fromBuffer(
          $core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImDirectKeyPackageConsumeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImDirectKeyPackageConsumeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.im.v1'),
      createEmptyInstance: create)
    ..aOM<ImNodeEndpoint>(1, _omitFieldNames ? '' : 'remoteEndpoint',
        subBuilder: ImNodeEndpoint.create)
    ..aOM<ConsumeImKeyPackageRequest>(2, _omitFieldNames ? '' : 'consume',
        subBuilder: ConsumeImKeyPackageRequest.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectKeyPackageConsumeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImDirectKeyPackageConsumeRequest copyWith(
          void Function(ImDirectKeyPackageConsumeRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ImDirectKeyPackageConsumeRequest))
          as ImDirectKeyPackageConsumeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImDirectKeyPackageConsumeRequest create() =>
      ImDirectKeyPackageConsumeRequest._();
  @$core.override
  ImDirectKeyPackageConsumeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImDirectKeyPackageConsumeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImDirectKeyPackageConsumeRequest>(
          create);
  static ImDirectKeyPackageConsumeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  ImNodeEndpoint get remoteEndpoint => $_getN(0);
  @$pb.TagNumber(1)
  set remoteEndpoint(ImNodeEndpoint value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRemoteEndpoint() => $_has(0);
  @$pb.TagNumber(1)
  void clearRemoteEndpoint() => $_clearField(1);
  @$pb.TagNumber(1)
  ImNodeEndpoint ensureRemoteEndpoint() => $_ensure(0);

  @$pb.TagNumber(2)
  ConsumeImKeyPackageRequest get consume => $_getN(1);
  @$pb.TagNumber(2)
  set consume(ConsumeImKeyPackageRequest value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasConsume() => $_has(1);
  @$pb.TagNumber(2)
  void clearConsume() => $_clearField(2);
  @$pb.TagNumber(2)
  ConsumeImKeyPackageRequest ensureConsume() => $_ensure(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
