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

import 'package:protobuf/protobuf.dart' as $pb;

class MlsWireMessageKind extends $pb.ProtobufEnum {
  static const MlsWireMessageKind MLS_WIRE_MESSAGE_KIND_UNSPECIFIED =
      MlsWireMessageKind._(
          0, _omitEnumNames ? '' : 'MLS_WIRE_MESSAGE_KIND_UNSPECIFIED');
  static const MlsWireMessageKind MLS_WIRE_MESSAGE_KIND_WELCOME =
      MlsWireMessageKind._(
          1, _omitEnumNames ? '' : 'MLS_WIRE_MESSAGE_KIND_WELCOME');
  static const MlsWireMessageKind MLS_WIRE_MESSAGE_KIND_APPLICATION =
      MlsWireMessageKind._(
          2, _omitEnumNames ? '' : 'MLS_WIRE_MESSAGE_KIND_APPLICATION');

  static const $core.List<MlsWireMessageKind> values = <MlsWireMessageKind>[
    MLS_WIRE_MESSAGE_KIND_UNSPECIFIED,
    MLS_WIRE_MESSAGE_KIND_WELCOME,
    MLS_WIRE_MESSAGE_KIND_APPLICATION,
  ];

  static final $core.List<MlsWireMessageKind?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 2);
  static MlsWireMessageKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const MlsWireMessageKind._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
