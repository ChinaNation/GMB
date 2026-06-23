// This is a generated file - do not edit.
//
// Generated from im_envelope.proto.

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

@$core.Deprecated('Use imMlsWireMessageKindDescriptor instead')
const ImMlsWireMessageKind$json = {
  '1': 'ImMlsWireMessageKind',
  '2': [
    {'1': 'IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED', '2': 0},
    {'1': 'IM_MLS_WIRE_MESSAGE_KIND_WELCOME', '2': 1},
    {'1': 'IM_MLS_WIRE_MESSAGE_KIND_APPLICATION', '2': 2},
  ],
};

/// Descriptor for `ImMlsWireMessageKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List imMlsWireMessageKindDescriptor = $convert.base64Decode(
    'ChRJbU1sc1dpcmVNZXNzYWdlS2luZBIoCiRJTV9NTFNfV0lSRV9NRVNTQUdFX0tJTkRfVU5TUE'
    'VDSUZJRUQQABIkCiBJTV9NTFNfV0lSRV9NRVNTQUdFX0tJTkRfV0VMQ09NRRABEigKJElNX01M'
    'U19XSVJFX01FU1NBR0VfS0lORF9BUFBMSUNBVElPThAC');

@$core.Deprecated('Use imNodeEndpointDescriptor instead')
const ImNodeEndpoint$json = {
  '1': 'ImNodeEndpoint',
  '2': [
    {'1': 'peer_id', '3': 1, '4': 1, '5': 9, '10': 'peerId'},
    {'1': 'multiaddr', '3': 2, '4': 1, '5': 9, '10': 'multiaddr'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
  ],
};

/// Descriptor for `ImNodeEndpoint`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imNodeEndpointDescriptor = $convert.base64Decode(
    'Cg5JbU5vZGVFbmRwb2ludBIXCgdwZWVyX2lkGAEgASgJUgZwZWVySWQSHAoJbXVsdGlhZGRyGA'
    'IgASgJUgltdWx0aWFkZHISEgoEa2luZBgDIAEoCVIEa2luZA==');

@$core.Deprecated('Use imRouteRecordDescriptor instead')
const ImRouteRecord$json = {
  '1': 'ImRouteRecord',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {
      '1': 'wallet_chat_account',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'walletChatAccount'
    },
    {
      '1': 'route_display_name',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'routeDisplayName'
    },
    {'1': 'im_device_id', '3': 4, '4': 1, '5': 9, '10': 'imDeviceId'},
    {
      '1': 'im_device_pubkey_hex',
      '3': 5,
      '4': 1,
      '5': 9,
      '10': 'imDevicePubkeyHex'
    },
    {'1': 'safety_number', '3': 6, '4': 1, '5': 9, '10': 'safetyNumber'},
    {
      '1': 'node_endpoints',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.gmb.im.v1.ImNodeEndpoint',
      '10': 'nodeEndpoints'
    },
    {'1': 'created_at_millis', '3': 8, '4': 1, '5': 4, '10': 'createdAtMillis'},
    {'1': 'expires_at_millis', '3': 9, '4': 1, '5': 4, '10': 'expiresAtMillis'},
  ],
};

/// Descriptor for `ImRouteRecord`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imRouteRecordDescriptor = $convert.base64Decode(
    'Cg1JbVJvdXRlUmVjb3JkEikKEHByb3RvY29sX3ZlcnNpb24YASABKA1SD3Byb3RvY29sVmVyc2'
    'lvbhIuChN3YWxsZXRfY2hhdF9hY2NvdW50GAIgASgJUhF3YWxsZXRDaGF0QWNjb3VudBIsChJy'
    'b3V0ZV9kaXNwbGF5X25hbWUYAyABKAlSEHJvdXRlRGlzcGxheU5hbWUSIAoMaW1fZGV2aWNlX2'
    'lkGAQgASgJUgppbURldmljZUlkEi8KFGltX2RldmljZV9wdWJrZXlfaGV4GAUgASgJUhFpbURl'
    'dmljZVB1YmtleUhleBIjCg1zYWZldHlfbnVtYmVyGAYgASgJUgxzYWZldHlOdW1iZXISQAoObm'
    '9kZV9lbmRwb2ludHMYByADKAsyGS5nbWIuaW0udjEuSW1Ob2RlRW5kcG9pbnRSDW5vZGVFbmRw'
    'b2ludHMSKgoRY3JlYXRlZF9hdF9taWxsaXMYCCABKARSD2NyZWF0ZWRBdE1pbGxpcxIqChFleH'
    'BpcmVzX2F0X21pbGxpcxgJIAEoBFIPZXhwaXJlc0F0TWlsbGlz');

@$core.Deprecated('Use imEnvelopeDescriptor instead')
const ImEnvelope$json = {
  '1': 'ImEnvelope',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {'1': 'envelope_id', '3': 2, '4': 1, '5': 9, '10': 'envelopeId'},
    {'1': 'conversation_id', '3': 3, '4': 1, '5': 9, '10': 'conversationId'},
    {
      '1': 'sender_chat_account',
      '3': 4,
      '4': 1,
      '5': 9,
      '10': 'senderChatAccount'
    },
    {
      '1': 'recipient_chat_account',
      '3': 5,
      '4': 1,
      '5': 9,
      '10': 'recipientChatAccount'
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
    {
      '1': 'attachment_manifest_hash',
      '3': 9,
      '4': 1,
      '5': 9,
      '10': 'attachmentManifestHash'
    },
    {'1': 'chunk_refs', '3': 10, '4': 3, '5': 9, '10': 'chunkRefs'},
    {
      '1': 'created_at_millis',
      '3': 11,
      '4': 1,
      '5': 4,
      '10': 'createdAtMillis'
    },
    {'1': 'ttl_millis', '3': 12, '4': 1, '5': 4, '10': 'ttlMillis'},
    {'1': 'ack_policy', '3': 13, '4': 1, '5': 9, '10': 'ackPolicy'},
    {
      '1': 'mls_message_kind',
      '3': 14,
      '4': 1,
      '5': 14,
      '6': '.gmb.im.v1.ImMlsWireMessageKind',
      '10': 'mlsMessageKind'
    },
    {'1': 'ratchet_tree', '3': 15, '4': 1, '5': 12, '10': 'ratchetTree'},
  ],
};

/// Descriptor for `ImEnvelope`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imEnvelopeDescriptor = $convert.base64Decode(
    'CgpJbUVudmVsb3BlEikKEHByb3RvY29sX3ZlcnNpb24YASABKA1SD3Byb3RvY29sVmVyc2lvbh'
    'IfCgtlbnZlbG9wZV9pZBgCIAEoCVIKZW52ZWxvcGVJZBInCg9jb252ZXJzYXRpb25faWQYAyAB'
    'KAlSDmNvbnZlcnNhdGlvbklkEi4KE3NlbmRlcl9jaGF0X2FjY291bnQYBCABKAlSEXNlbmRlck'
    'NoYXRBY2NvdW50EjQKFnJlY2lwaWVudF9jaGF0X2FjY291bnQYBSABKAlSFHJlY2lwaWVudENo'
    'YXRBY2NvdW50EigKEHNlbmRlcl9kZXZpY2VfaWQYBiABKAlSDnNlbmRlckRldmljZUlkEigKEG'
    '1sc193aXJlX21lc3NhZ2UYByABKAxSDm1sc1dpcmVNZXNzYWdlEi0KEmVuY3J5cHRlZF9tZXRh'
    'ZGF0YRgIIAEoDFIRZW5jcnlwdGVkTWV0YWRhdGESOAoYYXR0YWNobWVudF9tYW5pZmVzdF9oYX'
    'NoGAkgASgJUhZhdHRhY2htZW50TWFuaWZlc3RIYXNoEh0KCmNodW5rX3JlZnMYCiADKAlSCWNo'
    'dW5rUmVmcxIqChFjcmVhdGVkX2F0X21pbGxpcxgLIAEoBFIPY3JlYXRlZEF0TWlsbGlzEh0KCn'
    'R0bF9taWxsaXMYDCABKARSCXR0bE1pbGxpcxIdCgphY2tfcG9saWN5GA0gASgJUglhY2tQb2xp'
    'Y3kSSQoQbWxzX21lc3NhZ2Vfa2luZBgOIAEoDjIfLmdtYi5pbS52MS5JbU1sc1dpcmVNZXNzYW'
    'dlS2luZFIObWxzTWVzc2FnZUtpbmQSIQoMcmF0Y2hldF90cmVlGA8gASgMUgtyYXRjaGV0VHJl'
    'ZQ==');

@$core.Deprecated('Use imEnvelopeAckDescriptor instead')
const ImEnvelopeAck$json = {
  '1': 'ImEnvelopeAck',
  '2': [
    {'1': 'envelope_id', '3': 1, '4': 1, '5': 9, '10': 'envelopeId'},
    {'1': 'state', '3': 2, '4': 1, '5': 9, '10': 'state'},
  ],
};

/// Descriptor for `ImEnvelopeAck`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imEnvelopeAckDescriptor = $convert.base64Decode(
    'Cg1JbUVudmVsb3BlQWNrEh8KC2VudmVsb3BlX2lkGAEgASgJUgplbnZlbG9wZUlkEhQKBXN0YX'
    'RlGAIgASgJUgVzdGF0ZQ==');

@$core.Deprecated('Use imKeyPackageDescriptor instead')
const ImKeyPackage$json = {
  '1': 'ImKeyPackage',
  '2': [
    {'1': 'protocol_version', '3': 1, '4': 1, '5': 13, '10': 'protocolVersion'},
    {
      '1': 'owner_wallet_account',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'ownerWalletAccount'
    },
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
    {
      '1': 'consumed_at_millis',
      '3': 10,
      '4': 1,
      '5': 4,
      '10': 'consumedAtMillis'
    },
  ],
};

/// Descriptor for `ImKeyPackage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imKeyPackageDescriptor = $convert.base64Decode(
    'CgxJbUtleVBhY2thZ2USKQoQcHJvdG9jb2xfdmVyc2lvbhgBIAEoDVIPcHJvdG9jb2xWZXJzaW'
    '9uEjAKFG93bmVyX3dhbGxldF9hY2NvdW50GAIgASgJUhJvd25lcldhbGxldEFjY291bnQSGwoJ'
    'ZGV2aWNlX2lkGAMgASgJUghkZXZpY2VJZBIxChVkZXZpY2VfcHVibGljX2tleV9oZXgYBCABKA'
    'lSEmRldmljZVB1YmxpY0tleUhleBIkCg5rZXlfcGFja2FnZV9pZBgFIAEoCVIMa2V5UGFja2Fn'
    'ZUlkEh8KC2tleV9wYWNrYWdlGAYgASgMUgprZXlQYWNrYWdlEiEKDGNpcGhlcl9zdWl0ZRgHIA'
    'EoCVILY2lwaGVyU3VpdGUSKgoRY3JlYXRlZF9hdF9taWxsaXMYCCABKARSD2NyZWF0ZWRBdE1p'
    'bGxpcxIqChFleHBpcmVzX2F0X21pbGxpcxgJIAEoBFIPZXhwaXJlc0F0TWlsbGlzEiwKEmNvbn'
    'N1bWVkX2F0X21pbGxpcxgKIAEoBFIQY29uc3VtZWRBdE1pbGxpcw==');

@$core.Deprecated('Use publishImKeyPackageRequestDescriptor instead')
const PublishImKeyPackageRequest$json = {
  '1': 'PublishImKeyPackageRequest',
  '2': [
    {
      '1': 'owner_wallet_account',
      '3': 1,
      '4': 1,
      '5': 9,
      '10': 'ownerWalletAccount'
    },
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

/// Descriptor for `PublishImKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishImKeyPackageRequestDescriptor = $convert.base64Decode(
    'ChpQdWJsaXNoSW1LZXlQYWNrYWdlUmVxdWVzdBIwChRvd25lcl93YWxsZXRfYWNjb3VudBgBIA'
    'EoCVISb3duZXJXYWxsZXRBY2NvdW50EhsKCWRldmljZV9pZBgCIAEoCVIIZGV2aWNlSWQSMQoV'
    'ZGV2aWNlX3B1YmxpY19rZXlfaGV4GAMgASgJUhJkZXZpY2VQdWJsaWNLZXlIZXgSJAoOa2V5X3'
    'BhY2thZ2VfaWQYBCABKAlSDGtleVBhY2thZ2VJZBIfCgtrZXlfcGFja2FnZRgFIAEoDFIKa2V5'
    'UGFja2FnZRIhCgxjaXBoZXJfc3VpdGUYBiABKAlSC2NpcGhlclN1aXRlEioKEWNyZWF0ZWRfYX'
    'RfbWlsbGlzGAcgASgEUg9jcmVhdGVkQXRNaWxsaXMSKgoRZXhwaXJlc19hdF9taWxsaXMYCCAB'
    'KARSD2V4cGlyZXNBdE1pbGxpcw==');

@$core.Deprecated('Use fetchImKeyPackagesRequestDescriptor instead')
const FetchImKeyPackagesRequest$json = {
  '1': 'FetchImKeyPackagesRequest',
  '2': [
    {
      '1': 'owner_wallet_account',
      '3': 1,
      '4': 1,
      '5': 9,
      '10': 'ownerWalletAccount'
    },
    {
      '1': 'requester_chat_account',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'requesterChatAccount'
    },
    {'1': 'limit', '3': 3, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `FetchImKeyPackagesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List fetchImKeyPackagesRequestDescriptor = $convert.base64Decode(
    'ChlGZXRjaEltS2V5UGFja2FnZXNSZXF1ZXN0EjAKFG93bmVyX3dhbGxldF9hY2NvdW50GAEgAS'
    'gJUhJvd25lcldhbGxldEFjY291bnQSNAoWcmVxdWVzdGVyX2NoYXRfYWNjb3VudBgCIAEoCVIU'
    'cmVxdWVzdGVyQ2hhdEFjY291bnQSFAoFbGltaXQYAyABKA1SBWxpbWl0');

@$core.Deprecated('Use consumeImKeyPackageRequestDescriptor instead')
const ConsumeImKeyPackageRequest$json = {
  '1': 'ConsumeImKeyPackageRequest',
  '2': [
    {
      '1': 'owner_wallet_account',
      '3': 1,
      '4': 1,
      '5': 9,
      '10': 'ownerWalletAccount'
    },
    {'1': 'key_package_id', '3': 2, '4': 1, '5': 9, '10': 'keyPackageId'},
    {
      '1': 'requester_chat_account',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'requesterChatAccount'
    },
  ],
};

/// Descriptor for `ConsumeImKeyPackageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List consumeImKeyPackageRequestDescriptor = $convert.base64Decode(
    'ChpDb25zdW1lSW1LZXlQYWNrYWdlUmVxdWVzdBIwChRvd25lcl93YWxsZXRfYWNjb3VudBgBIA'
    'EoCVISb3duZXJXYWxsZXRBY2NvdW50EiQKDmtleV9wYWNrYWdlX2lkGAIgASgJUgxrZXlQYWNr'
    'YWdlSWQSNAoWcmVxdWVzdGVyX2NoYXRfYWNjb3VudBgDIAEoCVIUcmVxdWVzdGVyQ2hhdEFjY2'
    '91bnQ=');

@$core.Deprecated('Use imDirectDeliveryRequestDescriptor instead')
const ImDirectDeliveryRequest$json = {
  '1': 'ImDirectDeliveryRequest',
  '2': [
    {
      '1': 'remote_endpoint',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.ImNodeEndpoint',
      '10': 'remoteEndpoint'
    },
    {
      '1': 'envelope',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.ImEnvelope',
      '10': 'envelope'
    },
  ],
};

/// Descriptor for `ImDirectDeliveryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imDirectDeliveryRequestDescriptor = $convert.base64Decode(
    'ChdJbURpcmVjdERlbGl2ZXJ5UmVxdWVzdBJCCg9yZW1vdGVfZW5kcG9pbnQYASABKAsyGS5nbW'
    'IuaW0udjEuSW1Ob2RlRW5kcG9pbnRSDnJlbW90ZUVuZHBvaW50EjEKCGVudmVsb3BlGAIgASgL'
    'MhUuZ21iLmltLnYxLkltRW52ZWxvcGVSCGVudmVsb3Bl');

@$core.Deprecated('Use imDirectKeyPackageFetchRequestDescriptor instead')
const ImDirectKeyPackageFetchRequest$json = {
  '1': 'ImDirectKeyPackageFetchRequest',
  '2': [
    {
      '1': 'remote_endpoint',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.ImNodeEndpoint',
      '10': 'remoteEndpoint'
    },
    {
      '1': 'fetch',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.FetchImKeyPackagesRequest',
      '10': 'fetch'
    },
  ],
};

/// Descriptor for `ImDirectKeyPackageFetchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imDirectKeyPackageFetchRequestDescriptor =
    $convert.base64Decode(
        'Ch5JbURpcmVjdEtleVBhY2thZ2VGZXRjaFJlcXVlc3QSQgoPcmVtb3RlX2VuZHBvaW50GAEgAS'
        'gLMhkuZ21iLmltLnYxLkltTm9kZUVuZHBvaW50Ug5yZW1vdGVFbmRwb2ludBI6CgVmZXRjaBgC'
        'IAEoCzIkLmdtYi5pbS52MS5GZXRjaEltS2V5UGFja2FnZXNSZXF1ZXN0UgVmZXRjaA==');

@$core.Deprecated('Use imDirectKeyPackageConsumeRequestDescriptor instead')
const ImDirectKeyPackageConsumeRequest$json = {
  '1': 'ImDirectKeyPackageConsumeRequest',
  '2': [
    {
      '1': 'remote_endpoint',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.ImNodeEndpoint',
      '10': 'remoteEndpoint'
    },
    {
      '1': 'consume',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.gmb.im.v1.ConsumeImKeyPackageRequest',
      '10': 'consume'
    },
  ],
};

/// Descriptor for `ImDirectKeyPackageConsumeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List imDirectKeyPackageConsumeRequestDescriptor =
    $convert.base64Decode(
        'CiBJbURpcmVjdEtleVBhY2thZ2VDb25zdW1lUmVxdWVzdBJCCg9yZW1vdGVfZW5kcG9pbnQYAS'
        'ABKAsyGS5nbWIuaW0udjEuSW1Ob2RlRW5kcG9pbnRSDnJlbW90ZUVuZHBvaW50Ej8KB2NvbnN1'
        'bWUYAiABKAsyJS5nbWIuaW0udjEuQ29uc3VtZUltS2V5UGFja2FnZVJlcXVlc3RSB2NvbnN1bW'
        'U=');
