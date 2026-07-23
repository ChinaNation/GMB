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
    $core.String? peerAccountId,
    $core.String? routeDisplayName,
    $core.String? deviceId,
    $core.String? devicePublicKey,
    $core.String? safetyNumber,
    $core.String? nearbyPeerHint,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (peerAccountId != null) result.peerAccountId = peerAccountId;
    if (routeDisplayName != null) result.routeDisplayName = routeDisplayName;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKey != null) result.devicePublicKey = devicePublicKey;
    if (safetyNumber != null) result.safetyNumber = safetyNumber;
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
    ..aOS(2, _omitFieldNames ? '' : 'peerAccountId')
    ..aOS(3, _omitFieldNames ? '' : 'routeDisplayName')
    ..aOS(4, _omitFieldNames ? '' : 'deviceId')
    ..aOS(5, _omitFieldNames ? '' : 'devicePublicKey')
    ..aOS(6, _omitFieldNames ? '' : 'safetyNumber')
    ..aOS(7, _omitFieldNames ? '' : 'nearbyPeerHint')
    ..a<$fixnum.Int64>(
        8, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'expiresAtMillis', $pb.PbFieldType.OU6,
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
  $core.String get peerAccountId => $_getSZ(1);
  @$pb.TagNumber(2)
  set peerAccountId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPeerAccountId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPeerAccountId() => $_clearField(2);

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
  $core.String get devicePublicKey => $_getSZ(4);
  @$pb.TagNumber(5)
  set devicePublicKey($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDevicePublicKey() => $_has(4);
  @$pb.TagNumber(5)
  void clearDevicePublicKey() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get safetyNumber => $_getSZ(5);
  @$pb.TagNumber(6)
  set safetyNumber($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSafetyNumber() => $_has(5);
  @$pb.TagNumber(6)
  void clearSafetyNumber() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get nearbyPeerHint => $_getSZ(6);
  @$pb.TagNumber(7)
  set nearbyPeerHint($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasNearbyPeerHint() => $_has(6);
  @$pb.TagNumber(7)
  void clearNearbyPeerHint() => $_clearField(7);

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

class ChatEnvelope extends $pb.GeneratedMessage {
  factory ChatEnvelope({
    $core.int? protocolVersion,
    $core.String? envelopeId,
    $core.String? conversationId,
    $core.String? senderAccountId,
    $core.String? recipientAccountId,
    $core.String? senderDeviceId,
    $core.List<$core.int>? mlsWireMessage,
    $core.List<$core.int>? encryptedMetadata,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? ttlMillis,
    MlsWireMessageKind? mlsMessageKind,
    $core.List<$core.int>? ratchetTree,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (envelopeId != null) result.envelopeId = envelopeId;
    if (conversationId != null) result.conversationId = conversationId;
    if (senderAccountId != null) result.senderAccountId = senderAccountId;
    if (recipientAccountId != null)
      result.recipientAccountId = recipientAccountId;
    if (senderDeviceId != null) result.senderDeviceId = senderDeviceId;
    if (mlsWireMessage != null) result.mlsWireMessage = mlsWireMessage;
    if (encryptedMetadata != null) result.encryptedMetadata = encryptedMetadata;
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (ttlMillis != null) result.ttlMillis = ttlMillis;
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
    ..aOS(4, _omitFieldNames ? '' : 'senderAccountId')
    ..aOS(5, _omitFieldNames ? '' : 'recipientAccountId')
    ..aOS(6, _omitFieldNames ? '' : 'senderDeviceId')
    ..a<$core.List<$core.int>>(
        7, _omitFieldNames ? '' : 'mlsWireMessage', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        8, _omitFieldNames ? '' : 'encryptedMetadata', $pb.PbFieldType.OY)
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'createdAtMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        10, _omitFieldNames ? '' : 'ttlMillis', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<MlsWireMessageKind>(11, _omitFieldNames ? '' : 'mlsMessageKind',
        enumValues: MlsWireMessageKind.values)
    ..a<$core.List<$core.int>>(
        12, _omitFieldNames ? '' : 'ratchetTree', $pb.PbFieldType.OY)
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
  $core.String get senderAccountId => $_getSZ(3);
  @$pb.TagNumber(4)
  set senderAccountId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSenderAccountId() => $_has(3);
  @$pb.TagNumber(4)
  void clearSenderAccountId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get recipientAccountId => $_getSZ(4);
  @$pb.TagNumber(5)
  set recipientAccountId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRecipientAccountId() => $_has(4);
  @$pb.TagNumber(5)
  void clearRecipientAccountId() => $_clearField(5);

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
  $fixnum.Int64 get createdAtMillis => $_getI64(8);
  @$pb.TagNumber(9)
  set createdAtMillis($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAtMillis() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAtMillis() => $_clearField(9);

  @$pb.TagNumber(10)
  $fixnum.Int64 get ttlMillis => $_getI64(9);
  @$pb.TagNumber(10)
  set ttlMillis($fixnum.Int64 value) => $_setInt64(9, value);
  @$pb.TagNumber(10)
  $core.bool hasTtlMillis() => $_has(9);
  @$pb.TagNumber(10)
  void clearTtlMillis() => $_clearField(10);

  @$pb.TagNumber(11)
  MlsWireMessageKind get mlsMessageKind => $_getN(10);
  @$pb.TagNumber(11)
  set mlsMessageKind(MlsWireMessageKind value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasMlsMessageKind() => $_has(10);
  @$pb.TagNumber(11)
  void clearMlsMessageKind() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.List<$core.int> get ratchetTree => $_getN(11);
  @$pb.TagNumber(12)
  set ratchetTree($core.List<$core.int> value) => $_setBytes(11, value);
  @$pb.TagNumber(12)
  $core.bool hasRatchetTree() => $_has(11);
  @$pb.TagNumber(12)
  void clearRatchetTree() => $_clearField(12);
}

class ChatKeyPackage extends $pb.GeneratedMessage {
  factory ChatKeyPackage({
    $core.int? protocolVersion,
    $core.String? accountId,
    $core.String? deviceId,
    $core.String? devicePublicKey,
    $core.String? keyPackageId,
    $core.List<$core.int>? keyPackage,
    $core.String? cipherSuite,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (accountId != null) result.accountId = accountId;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKey != null) result.devicePublicKey = devicePublicKey;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (keyPackage != null) result.keyPackage = keyPackage;
    if (cipherSuite != null) result.cipherSuite = cipherSuite;
    if (createdAtMillis != null) result.createdAtMillis = createdAtMillis;
    if (expiresAtMillis != null) result.expiresAtMillis = expiresAtMillis;
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
    ..aOS(2, _omitFieldNames ? '' : 'accountId')
    ..aOS(3, _omitFieldNames ? '' : 'deviceId')
    ..aOS(4, _omitFieldNames ? '' : 'devicePublicKey')
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
  $core.String get accountId => $_getSZ(1);
  @$pb.TagNumber(2)
  set accountId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAccountId() => $_has(1);
  @$pb.TagNumber(2)
  void clearAccountId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get deviceId => $_getSZ(2);
  @$pb.TagNumber(3)
  set deviceId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDeviceId() => $_has(2);
  @$pb.TagNumber(3)
  void clearDeviceId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get devicePublicKey => $_getSZ(3);
  @$pb.TagNumber(4)
  set devicePublicKey($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDevicePublicKey() => $_has(3);
  @$pb.TagNumber(4)
  void clearDevicePublicKey() => $_clearField(4);

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
}

class PublishChatKeyPackageRequest extends $pb.GeneratedMessage {
  factory PublishChatKeyPackageRequest({
    $core.String? accountId,
    $core.String? deviceId,
    $core.String? devicePublicKey,
    $core.String? keyPackageId,
    $core.List<$core.int>? keyPackage,
    $core.String? cipherSuite,
    $fixnum.Int64? createdAtMillis,
    $fixnum.Int64? expiresAtMillis,
  }) {
    final result = create();
    if (accountId != null) result.accountId = accountId;
    if (deviceId != null) result.deviceId = deviceId;
    if (devicePublicKey != null) result.devicePublicKey = devicePublicKey;
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
    ..aOS(1, _omitFieldNames ? '' : 'accountId')
    ..aOS(2, _omitFieldNames ? '' : 'deviceId')
    ..aOS(3, _omitFieldNames ? '' : 'devicePublicKey')
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
  $core.String get accountId => $_getSZ(0);
  @$pb.TagNumber(1)
  set accountId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccountId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccountId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get deviceId => $_getSZ(1);
  @$pb.TagNumber(2)
  set deviceId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDeviceId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDeviceId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get devicePublicKey => $_getSZ(2);
  @$pb.TagNumber(3)
  set devicePublicKey($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDevicePublicKey() => $_has(2);
  @$pb.TagNumber(3)
  void clearDevicePublicKey() => $_clearField(3);

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
    $core.String? accountId,
    $core.String? requesterAccountId,
    $core.int? limit,
  }) {
    final result = create();
    if (accountId != null) result.accountId = accountId;
    if (requesterAccountId != null)
      result.requesterAccountId = requesterAccountId;
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
    ..aOS(1, _omitFieldNames ? '' : 'accountId')
    ..aOS(2, _omitFieldNames ? '' : 'requesterAccountId')
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
  $core.String get accountId => $_getSZ(0);
  @$pb.TagNumber(1)
  set accountId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccountId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccountId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get requesterAccountId => $_getSZ(1);
  @$pb.TagNumber(2)
  set requesterAccountId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRequesterAccountId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRequesterAccountId() => $_clearField(2);

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
    $core.String? accountId,
    $core.String? keyPackageId,
    $core.String? requesterAccountId,
  }) {
    final result = create();
    if (accountId != null) result.accountId = accountId;
    if (keyPackageId != null) result.keyPackageId = keyPackageId;
    if (requesterAccountId != null)
      result.requesterAccountId = requesterAccountId;
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
    ..aOS(1, _omitFieldNames ? '' : 'accountId')
    ..aOS(2, _omitFieldNames ? '' : 'keyPackageId')
    ..aOS(3, _omitFieldNames ? '' : 'requesterAccountId')
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
  $core.String get accountId => $_getSZ(0);
  @$pb.TagNumber(1)
  set accountId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccountId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccountId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get keyPackageId => $_getSZ(1);
  @$pb.TagNumber(2)
  set keyPackageId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKeyPackageId() => $_has(1);
  @$pb.TagNumber(2)
  void clearKeyPackageId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get requesterAccountId => $_getSZ(2);
  @$pb.TagNumber(3)
  set requesterAccountId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRequesterAccountId() => $_has(2);
  @$pb.TagNumber(3)
  void clearRequesterAccountId() => $_clearField(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
