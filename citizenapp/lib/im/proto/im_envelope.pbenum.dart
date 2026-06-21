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

import 'package:protobuf/protobuf.dart' as $pb;

class ImMlsWireMessageKind extends $pb.ProtobufEnum {
  static const ImMlsWireMessageKind IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED =
      ImMlsWireMessageKind._(
          0, _omitEnumNames ? '' : 'IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED');
  static const ImMlsWireMessageKind IM_MLS_WIRE_MESSAGE_KIND_WELCOME =
      ImMlsWireMessageKind._(
          1, _omitEnumNames ? '' : 'IM_MLS_WIRE_MESSAGE_KIND_WELCOME');
  static const ImMlsWireMessageKind IM_MLS_WIRE_MESSAGE_KIND_APPLICATION =
      ImMlsWireMessageKind._(
          2, _omitEnumNames ? '' : 'IM_MLS_WIRE_MESSAGE_KIND_APPLICATION');

  static const $core.List<ImMlsWireMessageKind> values = <ImMlsWireMessageKind>[
    IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED,
    IM_MLS_WIRE_MESSAGE_KIND_WELCOME,
    IM_MLS_WIRE_MESSAGE_KIND_APPLICATION,
  ];

  static final $core.List<ImMlsWireMessageKind?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 2);
  static ImMlsWireMessageKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ImMlsWireMessageKind._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
