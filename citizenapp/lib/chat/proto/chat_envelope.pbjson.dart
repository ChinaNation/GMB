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
    {'1': 'peer_account_id', '3': 2, '4': 1, '5': 9, '10': 'peerAccountId'},
    {
      '1': 'route_display_name',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'routeDisplayName'
    },
    {'1': 'device_id', '3': 4, '4': 1, '5': 9, '10': 'deviceId'},
    {'1': 'device_public_key', '3': 5, '4': 1, '5': 9, '10': 'devicePublicKey'},
    {'1': 'safety_number', '3': 6, '4': 1, '5': 9, '10': 'safetyNumber'},
    {'1': 'nearby_peer_hint', '3': 7, '4': 1, '5': 9, '10': 'nearbyPeerHint'},
    {'1': 'created_at_millis', '3': 8, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 9, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `ChatRoute`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List chatRouteDescriptor = $convert.base64Decode(
    'CglDaGF0Um91dGUSKQoQcHJvdG9jb2xfdmVyc2lvbhgBIAEoDVIPcHJvdG9jb2xWZXJzaW9uEi'
    'YKD3BlZXJfYWNjb3VudF9pZBgCIAEoCVINcGVlckFjY291bnRJZBIsChJyb3V0ZV9kaXNwbGF5'
    'X25hbWUYAyABKAlSEHJvdXRlRGlzcGxheU5hbWUSGwoJZGV2aWNlX2lkGAQgASgJUghkZXZpY2'
    'VJZBIqChFkZXZpY2VfcHVibGljX2tleRgFIAEoCVIPZGV2aWNlUHVibGljS2V5EiMKDXNhZmV0'
    'eV9udW1iZXIYBiABKAlSDHNhZmV0eU51bWJlchIoChBuZWFyYnlfcGVlcl9oaW50GAcgASgJUg'
    '5uZWFyYnlQZWVySGludBIqChFjcmVhdGVkX2F0X21pbGxpcxgIIAEoBFIPY3JlYXRlZEF0TWls'
    'bGlzEioKEWV4cGlyZXNfYXRfbWlsbGlzGAkgASgEUg9leHBpcmVzQXRNaWxsaXM=');

@$core.Deprecated('Use chatEnvelopeDescriptor instead')
const ChatEnvelope$json = {
  '1': 'ChatEnvelope',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'envelope_id', '3': 2, '4': 1, '5': 9, '10': 'envelopeId'},
    {'1': 'conversation_id', '3': 3, '4': 1, '5': 9, '10': 'conversationId'},
    {'1': 'sender_account_id', '3': 4, '4': 1, '5': 9, '10': 'senderAccountId'},
    {
      '1': 'recipient_account_id',
      '3': 5,
      '4': 1,
      '5': 9,
      '10': 'recipientAccountId'
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
    'IAEoCVIOY29udmVyc2F0aW9uSWQSKgoRc2VuZGVyX2FjY291bnRfaWQYBCABKAlSD3NlbmRlck'
    'FjY291bnRJZBIwChRyZWNpcGllbnRfYWNjb3VudF9pZBgFIAEoCVIScmVjaXBpZW50QWNjb3Vu'
    'dElkEigKEHNlbmRlcl9kZXZpY2VfaWQYBiABKAlSDnNlbmRlckRldmljZUlkEigKEG1sc193aX'
    'JlX21lc3NhZ2UYByABKAxSDm1sc1dpcmVNZXNzYWdlEi0KEmVuY3J5cHRlZF9tZXRhZGF0YRgI'
    'IAEoDFIRZW5jcnlwdGVkTWV0YWRhdGESKgoRY3JlYXRlZF9hdF9taWxsaXMYCSABKARSD2NyZW'
    'F0ZWRBdE1pbGxpcxIdCgp0dGxfbWlsbGlzGAogASgEUgl0dGxNaWxsaXMSSQoQbWxzX21lc3Nh'
    'Z2Vfa2luZBgLIAEoDjIfLmdtYi5jaGF0LnYxLk1sc1dpcmVNZXNzYWdlS2luZFIObWxzTWVzc2'
    'FnZUtpbmQSIQoMcmF0Y2hldF90cmVlGAwgASgMUgtyYXRjaGV0VHJlZQ==');

@$core.Deprecated('Use chatKeyPackageDescriptor instead')
const ChatKeyPackage$json = {
  '1': 'ChatKeyPackage',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'account_id', '3': 2, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'device_id', '3': 3, '4': 1, '5': 9, '10': 'deviceId'},
    {'1': 'device_public_key', '3': 4, '4': 1, '5': 9, '10': 'devicePublicKey'},
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
    'Npb24SHQoKYWNjb3VudF9pZBgCIAEoCVIJYWNjb3VudElkEhsKCWRldmljZV9pZBgDIAEoCVII'
    'ZGV2aWNlSWQSKgoRZGV2aWNlX3B1YmxpY19rZXkYBCABKAlSD2RldmljZVB1YmxpY0tleRIkCg'
    '5rZXlfcGFja2FnZV9pZBgFIAEoCVIMa2V5UGFja2FnZUlkEh8KC2tleV9wYWNrYWdlGAYgASgM'
    'UgprZXlQYWNrYWdlEiEKDGNpcGhlcl9zdWl0ZRgHIAEoCVILY2lwaGVyU3VpdGUSKgoRY3JlYX'
    'RlZF9hdF9taWxsaXMYCCABKARSD2NyZWF0ZWRBdE1pbGxpcxIqChFleHBpcmVzX2F0X21pbGxp'
    'cxgJIAEoBFIPZXhwaXJlc0F0TWlsbGlz');

@$core.Deprecated('Use publishChatKeyPackageRequestDescriptor instead')
const PublishChatKeyPackageRequest$json = {
  '1': 'PublishChatKeyPackageRequest',
  '2': [
    {'1': 'account_id', '3': 1, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'device_id', '3': 2, '4': 1, '5': 9, '10': 'deviceId'},
    {'1': 'device_public_key', '3': 3, '4': 1, '5': 9, '10': 'devicePublicKey'},
    {'1': 'key_package_id', '3': 4, '4': 1, '5': 9, '10': 'keyPackageId'},
    {'1': 'key_package', '3': 5, '4': 1, '5': 12, '10': 'keyPackage'},
    {'1': 'cipher_suite', '3': 6, '4': 1, '5': 9, '10': 'cipherSuite'},
    {'1': 'created_at_millis', '3': 7, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 8, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `PublishChatKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishChatKeyPackageRequestDescriptor = $convert.base64Decode(
    'ChxQdWJsaXNoQ2hhdEtleVBhY2thZ2VSZXF1ZXN0Eh0KCmFjY291bnRfaWQYASABKAlSCWFjY2'
    '91bnRJZBIbCglkZXZpY2VfaWQYAiABKAlSCGRldmljZUlkEioKEWRldmljZV9wdWJsaWNfa2V5'
    'GAMgASgJUg9kZXZpY2VQdWJsaWNLZXkSJAoOa2V5X3BhY2thZ2VfaWQYBCABKAlSDGtleVBhY2'
    'thZ2VJZBIfCgtrZXlfcGFja2FnZRgFIAEoDFIKa2V5UGFja2FnZRIhCgxjaXBoZXJfc3VpdGUY'
    'BiABKAlSC2NpcGhlclN1aXRlEioKEWNyZWF0ZWRfYXRfbWlsbGlzGAcgASgEUg9jcmVhdGVkQX'
    'RNaWxsaXMSKgoRZXhwaXJlc19hdF9taWxsaXMYCCABKARSD2V4cGlyZXNBdE1pbGxpcw==');

@$core.Deprecated('Use fetchChatKeyPackagesRequestDescriptor instead')
const FetchChatKeyPackagesRequest$json = {
  '1': 'FetchChatKeyPackagesRequest',
  '2': [
    {'1': 'account_id', '3': 1, '4': 1, '5': 9, '10': 'accountId'},
    {
      '1': 'requester_account_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'requesterAccountId'
    },
    {'1': 'limit', '3': 3, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `FetchChatKeyPackagesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List fetchChatKeyPackagesRequestDescriptor =
    $convert.base64Decode(
        'ChtGZXRjaENoYXRLZXlQYWNrYWdlc1JlcXVlc3QSHQoKYWNjb3VudF9pZBgBIAEoCVIJYWNjb3'
        'VudElkEjAKFHJlcXVlc3Rlcl9hY2NvdW50X2lkGAIgASgJUhJyZXF1ZXN0ZXJBY2NvdW50SWQS'
        'FAoFbGltaXQYAyABKA1SBWxpbWl0');

@$core.Deprecated('Use consumeChatKeyPackageRequestDescriptor instead')
const ConsumeChatKeyPackageRequest$json = {
  '1': 'ConsumeChatKeyPackageRequest',
  '2': [
    {'1': 'account_id', '3': 1, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'key_package_id', '3': 2, '4': 1, '5': 9, '10': 'keyPackageId'},
    {
      '1': 'requester_account_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'requesterAccountId'
    },
  ],
};

/// Descriptor for `ConsumeChatKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List consumeChatKeyPackageRequestDescriptor =
    $convert.base64Decode(
        'ChxDb25zdW1lQ2hhdEtleVBhY2thZ2VSZXF1ZXN0Eh0KCmFjY291bnRfaWQYASABKAlSCWFjY2'
        '91bnRJZBIkCg5rZXlfcGFja2FnZV9pZBgCIAEoCVIMa2V5UGFja2FnZUlkEjAKFHJlcXVlc3Rl'
        'cl9hY2NvdW50X2lkGAMgASgJUhJyZXF1ZXN0ZXJBY2NvdW50SWQ=');
