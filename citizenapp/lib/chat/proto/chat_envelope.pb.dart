// This is a generated file - do not edit.
//
// Generated from chat_envelope.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'chat_envelope.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'chat_envelope.pbenum.dart';

class ChatRoute extends $pb.GeneratedMessage {
  factory ChatRoute({
    $core.int? protocolVersion,
    $core.String? peerAccount,
    $core.String? routeDisplayName,
    $core.String? deviceId,
    $core.String? devicePublicKeyHex,
    $core.String? safetyNumber,
    $core.String? cloudflareMailboxId,
    $core.String? nearbyPeerHint,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (peerAccount != null) result.peerAccount = peerAccount;
    if (routeDisplayName != null) result.routeDisplayName = routeDisplayName;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKeyHex != null)
      result.devicePublicKeyHex = devicePublicKeyHex;
    if (safetyNumber != null) result.safetyNumber = safetyNumber;
    if (cloudflareMailboxId != null)
      result.cloudflareMailboxId = cloudflareMailboxId;
    if (nearbyPeerHint != null) result.nearbyPeerHint = nearbyPeerHint;
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (expiresAtMillis != null) result.expiresAtMillis = expiresAtMillis;
    return result;
  }

  ChatRoute._();

  factory ChatRoute.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ChatRoute.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChatRoute',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'peerAccount')
    ..aOS(3, _omitFieldNames ? '' : 'routeDisplayName')
    ..aOS(4, _omitFieldNames ? '' : 'deviceId')
    ..aOS(5, _omitFieldNames ? '' : 'devicePublicKeyHex')
    ..aOS(6, _omitFieldNames ? '' : 'safetyNumber')
    ..aOS(7, _omitFieldNames ? '' : 'cloudflareMailboxId')
    ..aOS(8, _omitFieldNames ? '' : 'nearbyPeerHint')
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        10, _omitFieldNames ? '' : 'expiresAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatRoute clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatRoute copyWith(void Function(ChatRoute) updates) =>
      super.copyWith((message) => updates(message as ChatRoute)) as ChatRoute;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ChatRoute create() => ChatRoute._();
  @$core.override
  ChatRoute createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ChatRoute getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChatRoute>(create);
  static ChatRoute? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get protocolVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set protocolVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get peerAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set peerAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPeerAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearPeerAccount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get routeDisplayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set routeDisplayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRouteDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearRouteDisplayName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get deviceId => $_getSZ(3);
  @$pb.TagNumber(4)
  set deviceId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDeviceId() => $_has(3);
  @$pb.TagNumber(4)
  void clearDeviceId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get devicePublicKeyHex => $_getSZ(4);
  @$pb.TagNumber(5)
  set devicePublicKeyHex($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDevicePublicKeyHex() => $_has(4);
  @$pb.TagNumber(5)
  void clearDevicePublicKeyHex() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get safetyNumber => $_getSZ(5);
  @$pb.TagNumber(6)
  set safetyNumber($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSafetyNumber() => $_has(5);
  @$pb.TagNumber(6)
  void clearSafetyNumber() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get cloudflareMailboxId => $_getSZ(6);
  @$pb.TagNumber(7)
  set cloudflareMailboxId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCloudflareMailboxId() => $_has(6);
  @$pb.TagNumber(7)
  void clearCloudflareMailboxId() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get nearbyPeerHint => $_getSZ(7);
  @$pb.TagNumber(8)
  set nearbyPeerHint($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasNearbyPeerHint() => $_has(7);
  @$pb.TagNumber(8)
  void clearNearbyPeerHint() => $_clearField(8);

  @$pb.TagNumber(9)
  $fixnum.Int64 get createdAtMillis => $_getI64(8);
  @$pb.TagNumber(9)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAtMillis() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAtMillis() => $_clearField(9);

  @$pb.TagNumber(10)
  $fixnum.Int64 get expiresAtMillis => $_getI64(9);
  @$pb.TagNumber(10)
  set expiresAtMillis($fixnum.Int64 value) => $_setInt64(9, value);
  @$pb.TagNumber(10)
  $core.bool hasExpiresAtMillis() => $_has(9);
  @$pb.TagNumber(10)
  void clearExpiresAtMillis() => $_clearField(10);
}

class ChatEnvelope extends $pb.GeneratedMessage {
  factory ChatEnvelope({
    $core.int? protocolVersion,
    $core.String? envelopeId,
    $core.String? conversationId,
    $core.String? senderAccount,
    $core.String? recipientAccount,
    $core.String? senderDeviceId,
    $core.List<$core.int>? mlsWireMessage,
    $core.List<$core.int>? encryptedMetadata,
    $core.String? attachmentManifestHash,
    $core.Iterable<$core.String>? chunkRefs,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? ttlMillis,
    $core.String? ackPolicy,
    MlsWireMessageKind? mlsMessageKind,
    $core.List<$core.int>? ratchetTree,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (envelopeId != null) result.envelopeId = envelopeId;
    if (conversationId != null) result.conversationId = conversationId;
    if (senderAccount != null) result.senderAccount = senderAccount;
    if (recipientAccount != null) result.recipientAccount = recipientAccount;
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

  ChatEnvelope._();

  factory ChatEnvelope.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ChatEnvelope.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChatEnvelope',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'envelopeId')
    ..aOS(3, _omitFieldNames ? '' : 'conversationId')
    ..aOS(4, _omitFieldNames ? '' : 'senderAccount')
    ..aOS(5, _omitFieldNames ? '' : 'recipientAccount')
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
    ..aE<MlsWireMessageKind>(14, _omitFieldNames ? '' : 'mlsMessageKind',
        enumValues: MlsWireMessageKind.values)
    ..a<$core.List<$core.int>>(
        15, _omitFieldNames ? '' : 'ratchetTree', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatEnvelope clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatEnvelope copyWith(void Function(ChatEnvelope) updates) =>
      super.copyWith((message) => updates(message as ChatEnvelope))
          as ChatEnvelope;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ChatEnvelope create() => ChatEnvelope._();
  @$core.override
  ChatEnvelope createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ChatEnvelope getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ChatEnvelope>(create);
  static ChatEnvelope? _defaultInstance;

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
  $core.String get senderAccount => $_getSZ(3);
  @$pb.TagNumber(4)
  set senderAccount($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSenderAccount() => $_has(3);
  @$pb.TagNumber(4)
  void clearSenderAccount() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get recipientAccount => $_getSZ(4);
  @$pb.TagNumber(5)
  set recipientAccount($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRecipientAccount() => $_has(4);
  @$pb.TagNumber(5)
  void clearRecipientAccount() => $_clearField(5);

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
  MlsWireMessageKind get mlsMessageKind => $_getN(13);
  @$pb.TagNumber(14)
  set mlsMessageKind(MlsWireMessageKind value) => $_setField(14, value);
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

class ChatEnvelopeAck extends $pb.GeneratedMessage {
  factory ChatEnvelopeAck({
    $core.String? envelopeId,
    $core.String? state,
  }) {
    final result = create();
    if (envelopeId != null) result.envelopeId = envelopeId;
    if (state != null) result.state = state;
    return result;
  }

  ChatEnvelopeAck._();

  factory ChatEnvelopeAck.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ChatEnvelopeAck.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChatEnvelopeAck',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'envelopeId')
    ..aOS(2, _omitFieldNames ? '' : 'state')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatEnvelopeAck clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatEnvelopeAck copyWith(void Function(ChatEnvelopeAck) updates) =>
      super.copyWith((message) => updates(message as ChatEnvelopeAck))
          as ChatEnvelopeAck;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ChatEnvelopeAck create() => ChatEnvelopeAck._();
  @$core.override
  ChatEnvelopeAck createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ChatEnvelopeAck getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ChatEnvelopeAck>(create);
  static ChatEnvelopeAck? _defaultInstance;

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

class ChatKeyPackage extends $pb.GeneratedMessage {
  factory ChatKeyPackage({
    $core.int? protocolVersion,
    $core.String? ownerAccount,
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
    if (ownerAccount != null) result.ownerAccount = ownerAccount;
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

  ChatKeyPackage._();

  factory ChatKeyPackage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ChatKeyPackage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChatKeyPackage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'protocolVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'ownerAccount')
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
  ChatKeyPackage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChatKeyPackage copyWith(void Function(ChatKeyPackage) updates) =>
      super.copyWith((message) => updates(message as ChatKeyPackage))
          as ChatKeyPackage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ChatKeyPackage create() => ChatKeyPackage._();
  @$core.override
  ChatKeyPackage createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ChatKeyPackage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ChatKeyPackage>(create);
  static ChatKeyPackage? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get protocolVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set protocolVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ownerAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set ownerAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOwnerAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearOwnerAccount() => $_clearField(2);

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

class PublishChatKeyPackageRequest extends $pb.GeneratedMessage {
  factory PublishChatKeyPackageRequest({
    $core.String? ownerAccount,
    $core.String? deviceId,
    $core.String? devicePublicKeyHex,
    $core.String? keyPackageId,
    $core.List<$core.int>? keyPackage,
    $core.String? cipherSuite,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (ownerAccount != null) result.ownerAccount = ownerAccount;
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

  PublishChatKeyPackageRequest._();

  factory PublishChatKeyPackageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PublishChatKeyPackageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PublishChatKeyPackageRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerAccount')
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
  PublishChatKeyPackageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishChatKeyPackageRequest copyWith(
          void Function(PublishChatKeyPackageRequest) updates) =>
      super.copyWith(
              (message) => updates(message as PublishChatKeyPackageRequest))
          as PublishChatKeyPackageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishChatKeyPackageRequest create() =>
      PublishChatKeyPackageRequest._();
  @$core.override
  PublishChatKeyPackageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PublishChatKeyPackageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PublishChatKeyPackageRequest>(create);
  static PublishChatKeyPackageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerAccount() => $_clearField(1);

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

class FetchChatKeyPackagesRequest extends $pb.GeneratedMessage {
  factory FetchChatKeyPackagesRequest({
    $core.String? ownerAccount,
    $core.String? requesterAccount,
    $core.int? limit,
  }) {
    final result = create();
    if (ownerAccount != null) result.ownerAccount = ownerAccount;
    if (requesterAccount != null) result.requesterAccount = requesterAccount;
    if (limit != null) result.limit = limit;
    return result;
  }

  FetchChatKeyPackagesRequest._();

  factory FetchChatKeyPackagesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FetchChatKeyPackagesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FetchChatKeyPackagesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerAccount')
    ..aOS(2, _omitFieldNames ? '' : 'requesterAccount')
    ..aI(3, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FetchChatKeyPackagesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FetchChatKeyPackagesRequest copyWith(
          void Function(FetchChatKeyPackagesRequest) updates) =>
      super.copyWith(
              (message) => updates(message as FetchChatKeyPackagesRequest))
          as FetchChatKeyPackagesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FetchChatKeyPackagesRequest create() =>
      FetchChatKeyPackagesRequest._();
  @$core.override
  FetchChatKeyPackagesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FetchChatKeyPackagesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FetchChatKeyPackagesRequest>(create);
  static FetchChatKeyPackagesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerAccount() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get requesterAccount => $_getSZ(1);
  @$pb.TagNumber(2)
  set requesterAccount($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRequesterAccount() => $_has(1);
  @$pb.TagNumber(2)
  void clearRequesterAccount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get limit => $_getIZ(2);
  @$pb.TagNumber(3)
  set limit($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);
}

class ConsumeChatKeyPackageRequest extends $pb.GeneratedMessage {
  factory ConsumeChatKeyPackageRequest({
    $core.String? ownerAccount,
    $core.String? keyPackageId,
    $core.String? requesterAccount,
  }) {
    final result = create();
    if (ownerAccount != null) result.ownerAccount = ownerAccount;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (requesterAccount != null) result.requesterAccount = requesterAccount;
    return result;
  }

  ConsumeChatKeyPackageRequest._();

  factory ConsumeChatKeyPackageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConsumeChatKeyPackageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConsumeChatKeyPackageRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'gmb.chat.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ownerAccount')
    ..aOS(2, _omitFieldNames ? '' : 'keyPackageId')
    ..aOS(3, _omitFieldNames ? '' : 'requesterAccount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsumeChatKeyPackageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsumeChatKeyPackageRequest copyWith(
          void Function(ConsumeChatKeyPackageRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ConsumeChatKeyPackageRequest))
          as ConsumeChatKeyPackageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConsumeChatKeyPackageRequest create() =>
      ConsumeChatKeyPackageRequest._();
  @$core.override
  ConsumeChatKeyPackageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConsumeChatKeyPackageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConsumeChatKeyPackageRequest>(create);
  static ConsumeChatKeyPackageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ownerAccount => $_getSZ(0);
  @$pb.TagNumber(1)
  set ownerAccount($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOwnerAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearOwnerAccount() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get keyPackageId => $_getSZ(1);
  @$pb.TagNumber(2)
  set keyPackageId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKeyPackageId() => $_has(1);
  @$pb.TagNumber(2)
  void clearKeyPackageId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get requesterAccount => $_getSZ(2);
  @$pb.TagNumber(3)
  set requesterAccount($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRequesterAccount() => $_has(2);
  @$pb.TagNumber(3)
  void clearRequesterAccount() => $_clearField(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
