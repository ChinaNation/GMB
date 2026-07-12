// This is a generated file - do not edit.
//
// Generated from chat_envelope.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use mlsWireMessageKindDescriptor instead')
const MlsWireMessageKind$json = {
  '1': 'MlsWireMessageKind',
  '2': [
    {'1': 'MLS_WIRE_MESSAGE_KIND_UNSPECIFIED', '2': 0},
    {'1': 'MLS_WIRE_MESSAGE_KIND_WELCOME', '2': 1},
    {'1': 'MLS_WIRE_MESSAGE_KIND_APPLICATION', '2': 2},
  ],
};

/// Descriptor for `MlsWireMessageKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List mlsWireMessageKindDescriptor = $convert.base64Decode(
    'ChJNbHNXaXJlTWVzc2FnZUtpbmQSJQohTUxTX1dJUkVfTUVTU0FHRV9LSU5EX1VOU1BFQ0lGSU'
    'VEEAASIQodTUxTX1dJUkVfTUVTU0FHRV9LSU5EX1dFTENPTUUQARIlCiFNTFNfV0lSRV9NRVNT'
    'QUdFX0tJTkRfQVBQTElDQVRJT04QAg==');

@$core.Deprecated('Use chatRouteDescriptor instead')
const ChatRoute$json = {
  '1': 'ChatRoute',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'peer_account', '3': 2, '4': 1, '5': 9, '10': 'peerAccount'},
    {
      '1': 'route_display_name',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'routeDisplayName'
    },
    {'1': 'device_id', '3': 4, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_public_key_hex',
      '3': 5,
      '4': 1,
      '5': 9,
      '10': 'devicePublicKeyHex'
    },
    {'1': 'safety_number', '3': 6, '4': 1, '5': 9, '10': 'safetyNumber'},
    {'1': 'nearby_peer_hint', '3': 7, '4': 1, '5': 9, '10': 'nearbyPeerHint'},
    {'1': 'created_at_millis', '3': 8, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 9, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `ChatRoute`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List chatRouteDescriptor = $convert.base64Decode(
    'CglDaGF0Um91dGUSKQoQcHJvdG9jb2xfdmVyc2lvbhgBIAEoDVIPcHJvdG9jb2xWZXJzaW9uEi'
    'EKDHBlZXJfYWNjb3VudBgCIAEoCVILcGVlckFjY291bnQSLAoScm91dGVfZGlzcGxheV9uYW1l'
    'GAMgASgJUhByb3V0ZURpc3BsYXlOYW1lEhsKCWRldmljZV9pZBgEIAEoCVIIZGV2aWNlSWQSMQ'
    'oVZGV2aWNlX3B1YmxpY19rZXlfaGV4GAUgASgJUhJkZXZpY2VQdWJsaWNLZXlIZXgSIwoNc2Fm'
    'ZXR5X251bWJlchgGIAEoCVIMc2FmZXR5TnVtYmVyEigKEG5lYXJieV9wZWVyX2hpbnQYByABKA'
    'lSDm5lYXJieVBlZXJIaW50EioKEWNyZWF0ZWRfYXRfbWlsbGlzGAggASgEUg9jcmVhdGVkQXRN'
    'aWxsaXMSKgoRZXhwaXJlc19hdF9taWxsaXMYCSABKARSD2V4cGlyZXNBdE1pbGxpcw==');

@$core.Deprecated('Use chatEnvelopeDescriptor instead')
const ChatEnvelope$json = {
  '1': 'ChatEnvelope',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'envelope_id', '3': 2, '4': 1, '5': 9, '10': 'envelopeId'},
    {'1': 'conversation_id', '3': 3, '4': 1, '5': 9, '10': 'conversationId'},
    {'1': 'sender_account', '3': 4, '4': 1, '5': 9, '10': 'senderAccount'},
    {
      '1': 'recipient_account',
      '3': 5,
      '4': 1,
      '5': 9,
      '10': 'recipientAccount'
    },
    {'1': 'sender_device_id', '3': 6, '4': 1, '5': 9, '10': 'senderDeviceId'},
    {'1': 'mls_wire_message', '3': 7, '4': 1, '5': 12, '10': 'mlsWireMessage'},
    {
      '1': 'encrypted_metadata',
      '3': 8,
      '4': 1,
      '5': 12,
      '10': 'encryptedMetadata'
    },
    {'1': 'created_at_millis', '3': 9, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'ttl_millis', '3': 10, '4': 1, '5': 4, '10': 'ttlMillis'},
    {
      '1': 'mls_message_kind',
      '3': 11,
      '4': 1,
      '5': 14,
      '6': '.gmb.chat.v1.MlsWireMessageKind',
      '10': 'mlsMessageKind'
    },
    {'1': 'ratchet_tree', '3': 12, '4': 1, '5': 12, '10': 'ratchetTree'},
  ],
};

/// Descriptor for `ChatEnvelope`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List chatEnvelopeDescriptor = $convert.base64Decode(
    'CgxDaGF0RW52ZWxvcGUSKQoQcHJvdG9jb2xfdmVyc2lvbhgBIAEoDVIPcHJvdG9jb2xWZXJzaW'
    '9uEh8KC2VudmVsb3BlX2lkGAIgASgJUgplbnZlbG9wZUlkEicKD2NvbnZlcnNhdGlvbl9pZBgD'
    'IAEoCVIOY29udmVyc2F0aW9uSWQSJQoOc2VuZGVyX2FjY291bnQYBCABKAlSDXNlbmRlckFjY2'
    '91bnQSKwoRcmVjaXBpZW50X2FjY291bnQYBSABKAlSEHJlY2lwaWVudEFjY291bnQSKAoQc2Vu'
    'ZGVyX2RldmljZV9pZBgGIAEoCVIOc2VuZGVyRGV2aWNlSWQSKAoQbWxzX3dpcmVfbWVzc2FnZR'
    'gHIAEoDFIObWxzV2lyZU1lc3NhZ2USLQoSZW5jcnlwdGVkX21ldGFkYXRhGAggASgMUhFlbmNy'
    'eXB0ZWRNZXRhZGF0YRIqChFjcmVhdGVkX2F0X21pbGxpcxgJIAEoBFIPY3JlYXRlZEF0TWlsbG'
    'lzEh0KCnR0bF9taWxsaXMYCiABKARSCXR0bE1pbGxpcxJJChBtbHNfbWVzc2FnZV9raW5kGAsg'
    'ASgOMh8uZ21iLmNoYXQudjEuTWxzV2lyZU1lc3NhZ2VLaW5kUg5tbHNNZXNzYWdlS2luZBIhCg'
    'xyYXRjaGV0X3RyZWUYDCABKAxSC3JhdGNoZXRUcmVl');

@$core.Deprecated('Use chatKeyPackageDescriptor instead')
const ChatKeyPackage$json = {
  '1': 'ChatKeyPackage',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'owner_account', '3': 2, '4': 1, '5': 9, '10': 'ownerAccount'},
    {'1': 'device_id', '3': 3, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_public_key_hex',
      '3': 4,
      '4': 1,
      '5': 9,
      '10': 'devicePublicKeyHex'
    },
    {'1': 'key_package_id', '3': 5, '4': 1, '5': 9, '10': 'keyPackageId'},
    {'1': 'key_package', '3': 6, '4': 1, '5': 12, '10': 'keyPackage'},
    {'1': 'cipher_suite', '3': 7, '4': 1, '5': 9, '10': 'cipherSuite'},
    {'1': 'created_at_millis', '3': 8, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 9, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `ChatKeyPackage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List chatKeyPackageDescriptor = $convert.base64Decode(
    'Cg5DaGF0S2V5UGFja2FnZRIpChBwcm90b2NvbF92ZXJzaW9uGAEgASgNUg9wcm90b2NvbFZlcn'
    'Npb24SIwoNb3duZXJfYWNjb3VudBgCIAEoCVIMb3duZXJBY2NvdW50EhsKCWRldmljZV9pZBgD'
    'IAEoCVIIZGV2aWNlSWQSMQoVZGV2aWNlX3B1YmxpY19rZXlfaGV4GAQgASgJUhJkZXZpY2VQdW'
    'JsaWNLZXlIZXgSJAoOa2V5X3BhY2thZ2VfaWQYBSABKAlSDGtleVBhY2thZ2VJZBIfCgtrZXlf'
    'cGFja2FnZRgGIAEoDFIKa2V5UGFja2FnZRIhCgxjaXBoZXJfc3VpdGUYByABKAlSC2NpcGhlcl'
    'N1aXRlEioKEWNyZWF0ZWRfYXRfbWlsbGlzGAggASgEUg9jcmVhdGVkQXRNaWxsaXMSKgoRZXhw'
    'aXJlc19hdF9taWxsaXMYCSABKARSD2V4cGlyZXNBdE1pbGxpcw==');

@$core.Deprecated('Use publishChatKeyPackageRequestDescriptor instead')
const PublishChatKeyPackageRequest$json = {
  '1': 'PublishChatKeyPackageRequest',
  '2': [
    {'1': 'owner_account', '3': 1, '4': 1, '5': 9, '10': 'ownerAccount'},
    {'1': 'device_id', '3': 2, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_public_key_hex',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'devicePublicKeyHex'
    },
    {'1': 'key_package_id', '3': 4, '4': 1, '5': 9, '10': 'keyPackageId'},
    {'1': 'key_package', '3': 5, '4': 1, '5': 12, '10': 'keyPackage'},
    {'1': 'cipher_suite', '3': 6, '4': 1, '5': 9, '10': 'cipherSuite'},
    {'1': 'created_at_millis', '3': 7, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 8, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `PublishChatKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishChatKeyPackageRequestDescriptor = $convert.base64Decode(
    'ChxQdWJsaXNoQ2hhdEtleVBhY2thZ2VSZXF1ZXN0EiMKDW93bmVyX2FjY291bnQYASABKAlSDG'
    '93bmVyQWNjb3VudBIbCglkZXZpY2VfaWQYAiABKAlSCGRldmljZUlkEjEKFWRldmljZV9wdWJs'
    'aWNfa2V5X2hleBgDIAEoCVISZGV2aWNlUHVibGljS2V5SGV4EiQKDmtleV9wYWNrYWdlX2lkGA'
    'QgASgJUgxrZXlQYWNrYWdlSWQSHwoLa2V5X3BhY2thZ2UYBSABKAxSCmtleVBhY2thZ2USIQoM'
    'Y2lwaGVyX3N1aXRlGAYgASgJUgtjaXBoZXJTdWl0ZRIqChFjcmVhdGVkX2F0X21pbGxpcxgHIA'
    'EoBFIPY3JlYXRlZEF0TWlsbGlzEioKEWV4cGlyZXNfYXRfbWlsbGlzGAggASgEUg9leHBpcmVz'
    'QXRNaWxsaXM=');

@$core.Deprecated('Use fetchChatKeyPackagesRequestDescriptor instead')
const FetchChatKeyPackagesRequest$json = {
  '1': 'FetchChatKeyPackagesRequest',
  '2': [
    {'1': 'owner_account', '3': 1, '4': 1, '5': 9, '10': 'ownerAccount'},
    {
      '1': 'requester_account',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'requesterAccount'
    },
    {'1': 'limit', '3': 3, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `FetchChatKeyPackagesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List fetchChatKeyPackagesRequestDescriptor =
    $convert.base64Decode(
        'ChtGZXRjaENoYXRLZXlQYWNrYWdlc1JlcXVlc3QSIwoNb3duZXJfYWNjb3VudBgBIAEoCVIMb3'
        'duZXJBY2NvdW50EisKEXJlcXVlc3Rlcl9hY2NvdW50GAIgASgJUhByZXF1ZXN0ZXJBY2NvdW50'
        'EhQKBWxpbWl0GAMgASgNUgVsaW1pdA==');

@$core.Deprecated('Use consumeChatKeyPackageRequestDescriptor instead')
const ConsumeChatKeyPackageRequest$json = {
  '1': 'ConsumeChatKeyPackageRequest',
  '2': [
    {'1': 'owner_account', '3': 1, '4': 1, '5': 9, '10': 'ownerAccount'},
    {'1': 'key_package_id', '3': 2, '4': 1, '5': 9, '10': 'keyPackageId'},
    {
      '1': 'requester_account',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'requesterAccount'
    },
  ],
};

/// Descriptor for `ConsumeChatKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List consumeChatKeyPackageRequestDescriptor =
    $convert.base64Decode(
        'ChxDb25zdW1lQ2hhdEtleVBhY2thZ2VSZXF1ZXN0EiMKDW93bmVyX2FjY291bnQYASABKAlSDG'
        '93bmVyQWNjb3VudBIkCg5rZXlfcGFja2FnZV9pZBgCIAEoCVIMa2V5UGFja2FnZUlkEisKEXJl'
        'cXVlc3Rlcl9hY2NvdW50GAMgASgJUhByZXF1ZXN0ZXJBY2NvdW50');
