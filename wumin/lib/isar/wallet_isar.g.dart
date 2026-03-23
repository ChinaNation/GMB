// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_isar.dart';

// **************************************************************************
// IsarCollectionGenerator
// **************************************************************************

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetWalletProfileEntityCollection on Isar {
  IsarCollection<WalletProfileEntity> get walletProfileEntitys =>
      this.collection();
}

const WalletProfileEntitySchema = CollectionSchema(
  name: r'WalletProfileEntity',
  id: -5044143814062565046,
  properties: {
    r'address': PropertySchema(
      id: 0,
      name: r'address',
      type: IsarType.string,
    ),
    r'alg': PropertySchema(
      id: 1,
      name: r'alg',
      type: IsarType.string,
    ),
    r'balance': PropertySchema(
      id: 2,
      name: r'balance',
      type: IsarType.double,
    ),
    r'createdAtMillis': PropertySchema(
      id: 3,
      name: r'createdAtMillis',
      type: IsarType.long,
    ),
    r'groupNames': PropertySchema(
      id: 4,
      name: r'groupNames',
      type: IsarType.string,
    ),
    r'pubkeyHex': PropertySchema(
      id: 5,
      name: r'pubkeyHex',
      type: IsarType.string,
    ),
    r'signMode': PropertySchema(
      id: 6,
      name: r'signMode',
      type: IsarType.string,
    ),
    r'sortOrder': PropertySchema(
      id: 7,
      name: r'sortOrder',
      type: IsarType.long,
    ),
    r'source': PropertySchema(
      id: 8,
      name: r'source',
      type: IsarType.string,
    ),
    r'ss58': PropertySchema(
      id: 9,
      name: r'ss58',
      type: IsarType.long,
    ),
    r'walletIcon': PropertySchema(
      id: 10,
      name: r'walletIcon',
      type: IsarType.string,
    ),
    r'walletIndex': PropertySchema(
      id: 11,
      name: r'walletIndex',
      type: IsarType.long,
    ),
    r'walletName': PropertySchema(
      id: 12,
      name: r'walletName',
      type: IsarType.string,
    )
  },
  estimateSize: _walletProfileEntityEstimateSize,
  serialize: _walletProfileEntitySerialize,
  deserialize: _walletProfileEntityDeserialize,
  deserializeProp: _walletProfileEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'walletIndex': IndexSchema(
      id: 3929031194099616871,
      name: r'walletIndex',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'walletIndex',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    ),
    r'address': IndexSchema(
      id: -259407546592846288,
      name: r'address',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'address',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'pubkeyHex': IndexSchema(
      id: 5838006650964594011,
      name: r'pubkeyHex',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'pubkeyHex',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _walletProfileEntityGetId,
  getLinks: _walletProfileEntityGetLinks,
  attach: _walletProfileEntityAttach,
  version: '3.1.0+1',
);

int _walletProfileEntityEstimateSize(
  WalletProfileEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.address.length * 3;
  bytesCount += 3 + object.alg.length * 3;
  bytesCount += 3 + object.groupNames.length * 3;
  bytesCount += 3 + object.pubkeyHex.length * 3;
  bytesCount += 3 + object.signMode.length * 3;
  bytesCount += 3 + object.source.length * 3;
  bytesCount += 3 + object.walletIcon.length * 3;
  bytesCount += 3 + object.walletName.length * 3;
  return bytesCount;
}

void _walletProfileEntitySerialize(
  WalletProfileEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeString(offsets[0], object.address);
  writer.writeString(offsets[1], object.alg);
  writer.writeDouble(offsets[2], object.balance);
  writer.writeLong(offsets[3], object.createdAtMillis);
  writer.writeString(offsets[4], object.groupNames);
  writer.writeString(offsets[5], object.pubkeyHex);
  writer.writeString(offsets[6], object.signMode);
  writer.writeLong(offsets[7], object.sortOrder);
  writer.writeString(offsets[8], object.source);
  writer.writeLong(offsets[9], object.ss58);
  writer.writeString(offsets[10], object.walletIcon);
  writer.writeLong(offsets[11], object.walletIndex);
  writer.writeString(offsets[12], object.walletName);
}

WalletProfileEntity _walletProfileEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = WalletProfileEntity();
  object.address = reader.readString(offsets[0]);
  object.alg = reader.readString(offsets[1]);
  object.balance = reader.readDouble(offsets[2]);
  object.createdAtMillis = reader.readLong(offsets[3]);
  object.groupNames = reader.readString(offsets[4]);
  object.id = id;
  object.pubkeyHex = reader.readString(offsets[5]);
  object.signMode = reader.readString(offsets[6]);
  object.sortOrder = reader.readLong(offsets[7]);
  object.source = reader.readString(offsets[8]);
  object.ss58 = reader.readLong(offsets[9]);
  object.walletIcon = reader.readString(offsets[10]);
  object.walletIndex = reader.readLong(offsets[11]);
  object.walletName = reader.readString(offsets[12]);
  return object;
}

P _walletProfileEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readString(offset)) as P;
    case 1:
      return (reader.readString(offset)) as P;
    case 2:
      return (reader.readDouble(offset)) as P;
    case 3:
      return (reader.readLong(offset)) as P;
    case 4:
      return (reader.readString(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    case 6:
      return (reader.readString(offset)) as P;
    case 7:
      return (reader.readLong(offset)) as P;
    case 8:
      return (reader.readString(offset)) as P;
    case 9:
      return (reader.readLong(offset)) as P;
    case 10:
      return (reader.readString(offset)) as P;
    case 11:
      return (reader.readLong(offset)) as P;
    case 12:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _walletProfileEntityGetId(WalletProfileEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _walletProfileEntityGetLinks(
    WalletProfileEntity object) {
  return [];
}

void _walletProfileEntityAttach(
    IsarCollection<dynamic> col, Id id, WalletProfileEntity object) {
  object.id = id;
}

extension WalletProfileEntityByIndex on IsarCollection<WalletProfileEntity> {
  Future<WalletProfileEntity?> getByWalletIndex(int walletIndex) {
    return getByIndex(r'walletIndex', [walletIndex]);
  }

  WalletProfileEntity? getByWalletIndexSync(int walletIndex) {
    return getByIndexSync(r'walletIndex', [walletIndex]);
  }

  Future<bool> deleteByWalletIndex(int walletIndex) {
    return deleteByIndex(r'walletIndex', [walletIndex]);
  }

  bool deleteByWalletIndexSync(int walletIndex) {
    return deleteByIndexSync(r'walletIndex', [walletIndex]);
  }

  Future<List<WalletProfileEntity?>> getAllByWalletIndex(
      List<int> walletIndexValues) {
    final values = walletIndexValues.map((e) => [e]).toList();
    return getAllByIndex(r'walletIndex', values);
  }

  List<WalletProfileEntity?> getAllByWalletIndexSync(
      List<int> walletIndexValues) {
    final values = walletIndexValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'walletIndex', values);
  }

  Future<int> deleteAllByWalletIndex(List<int> walletIndexValues) {
    final values = walletIndexValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'walletIndex', values);
  }

  int deleteAllByWalletIndexSync(List<int> walletIndexValues) {
    final values = walletIndexValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'walletIndex', values);
  }

  Future<Id> putByWalletIndex(WalletProfileEntity object) {
    return putByIndex(r'walletIndex', object);
  }

  Id putByWalletIndexSync(WalletProfileEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'walletIndex', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByWalletIndex(List<WalletProfileEntity> objects) {
    return putAllByIndex(r'walletIndex', objects);
  }

  List<Id> putAllByWalletIndexSync(List<WalletProfileEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'walletIndex', objects, saveLinks: saveLinks);
  }

  Future<WalletProfileEntity?> getByAddress(String address) {
    return getByIndex(r'address', [address]);
  }

  WalletProfileEntity? getByAddressSync(String address) {
    return getByIndexSync(r'address', [address]);
  }

  Future<bool> deleteByAddress(String address) {
    return deleteByIndex(r'address', [address]);
  }

  bool deleteByAddressSync(String address) {
    return deleteByIndexSync(r'address', [address]);
  }

  Future<List<WalletProfileEntity?>> getAllByAddress(
      List<String> addressValues) {
    final values = addressValues.map((e) => [e]).toList();
    return getAllByIndex(r'address', values);
  }

  List<WalletProfileEntity?> getAllByAddressSync(List<String> addressValues) {
    final values = addressValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'address', values);
  }

  Future<int> deleteAllByAddress(List<String> addressValues) {
    final values = addressValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'address', values);
  }

  int deleteAllByAddressSync(List<String> addressValues) {
    final values = addressValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'address', values);
  }

  Future<Id> putByAddress(WalletProfileEntity object) {
    return putByIndex(r'address', object);
  }

  Id putByAddressSync(WalletProfileEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'address', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByAddress(List<WalletProfileEntity> objects) {
    return putAllByIndex(r'address', objects);
  }

  List<Id> putAllByAddressSync(List<WalletProfileEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'address', objects, saveLinks: saveLinks);
  }

  Future<WalletProfileEntity?> getByPubkeyHex(String pubkeyHex) {
    return getByIndex(r'pubkeyHex', [pubkeyHex]);
  }

  WalletProfileEntity? getByPubkeyHexSync(String pubkeyHex) {
    return getByIndexSync(r'pubkeyHex', [pubkeyHex]);
  }

  Future<bool> deleteByPubkeyHex(String pubkeyHex) {
    return deleteByIndex(r'pubkeyHex', [pubkeyHex]);
  }

  bool deleteByPubkeyHexSync(String pubkeyHex) {
    return deleteByIndexSync(r'pubkeyHex', [pubkeyHex]);
  }

  Future<List<WalletProfileEntity?>> getAllByPubkeyHex(
      List<String> pubkeyHexValues) {
    final values = pubkeyHexValues.map((e) => [e]).toList();
    return getAllByIndex(r'pubkeyHex', values);
  }

  List<WalletProfileEntity?> getAllByPubkeyHexSync(
      List<String> pubkeyHexValues) {
    final values = pubkeyHexValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'pubkeyHex', values);
  }

  Future<int> deleteAllByPubkeyHex(List<String> pubkeyHexValues) {
    final values = pubkeyHexValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'pubkeyHex', values);
  }

  int deleteAllByPubkeyHexSync(List<String> pubkeyHexValues) {
    final values = pubkeyHexValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'pubkeyHex', values);
  }

  Future<Id> putByPubkeyHex(WalletProfileEntity object) {
    return putByIndex(r'pubkeyHex', object);
  }

  Id putByPubkeyHexSync(WalletProfileEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'pubkeyHex', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByPubkeyHex(List<WalletProfileEntity> objects) {
    return putAllByIndex(r'pubkeyHex', objects);
  }

  List<Id> putAllByPubkeyHexSync(List<WalletProfileEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'pubkeyHex', objects, saveLinks: saveLinks);
  }
}

extension WalletProfileEntityQueryWhereSort
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QWhere> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhere>
      anyWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'walletIndex'),
      );
    });
  }
}

extension WalletProfileEntityQueryWhere
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QWhereClause> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      idNotEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: lowerId,
        includeLower: includeLower,
        upper: upperId,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      walletIndexEqualTo(int walletIndex) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'walletIndex',
        value: [walletIndex],
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      walletIndexNotEqualTo(int walletIndex) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletIndex',
              lower: [],
              upper: [walletIndex],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletIndex',
              lower: [walletIndex],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletIndex',
              lower: [walletIndex],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletIndex',
              lower: [],
              upper: [walletIndex],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      walletIndexGreaterThan(
    int walletIndex, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'walletIndex',
        lower: [walletIndex],
        includeLower: include,
        upper: [],
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      walletIndexLessThan(
    int walletIndex, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'walletIndex',
        lower: [],
        upper: [walletIndex],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      walletIndexBetween(
    int lowerWalletIndex,
    int upperWalletIndex, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'walletIndex',
        lower: [lowerWalletIndex],
        includeLower: includeLower,
        upper: [upperWalletIndex],
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      addressEqualTo(String address) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'address',
        value: [address],
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      addressNotEqualTo(String address) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'address',
              lower: [],
              upper: [address],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'address',
              lower: [address],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'address',
              lower: [address],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'address',
              lower: [],
              upper: [address],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      pubkeyHexEqualTo(String pubkeyHex) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'pubkeyHex',
        value: [pubkeyHex],
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterWhereClause>
      pubkeyHexNotEqualTo(String pubkeyHex) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'pubkeyHex',
              lower: [],
              upper: [pubkeyHex],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'pubkeyHex',
              lower: [pubkeyHex],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'pubkeyHex',
              lower: [pubkeyHex],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'pubkeyHex',
              lower: [],
              upper: [pubkeyHex],
              includeUpper: false,
            ));
      }
    });
  }
}

extension WalletProfileEntityQueryFilter on QueryBuilder<WalletProfileEntity,
    WalletProfileEntity, QFilterCondition> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'address',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'address',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'address',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      addressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'address',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'alg',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'alg',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'alg',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'alg',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      algIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'alg',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      balanceEqualTo(
    double value, {
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'balance',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      balanceGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'balance',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      balanceLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'balance',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      balanceBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'balance',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      createdAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      createdAtMillisGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      createdAtMillisLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      createdAtMillisBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'createdAtMillis',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'groupNames',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'groupNames',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'groupNames',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'groupNames',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      groupNamesIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'groupNames',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      idGreaterThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      idLessThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'id',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'pubkeyHex',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'pubkeyHex',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'pubkeyHex',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      pubkeyHexIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'pubkeyHex',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'signMode',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'signMode',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'signMode',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'signMode',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      signModeIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'signMode',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sortOrderEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sortOrderGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sortOrderLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sortOrderBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'sortOrder',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'source',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'source',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'source',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      sourceIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'source',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      ss58EqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'ss58',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      ss58GreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'ss58',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      ss58LessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'ss58',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      ss58Between(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'ss58',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'walletIcon',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'walletIcon',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'walletIcon',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletIcon',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIconIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'walletIcon',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIndexEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIndexGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'walletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIndexLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'walletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletIndexBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'walletIndex',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'walletName',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'walletName',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletName',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterFilterCondition>
      walletNameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'walletName',
        value: '',
      ));
    });
  }
}

extension WalletProfileEntityQueryObject on QueryBuilder<WalletProfileEntity,
    WalletProfileEntity, QFilterCondition> {}

extension WalletProfileEntityQueryLinks on QueryBuilder<WalletProfileEntity,
    WalletProfileEntity, QFilterCondition> {}

extension WalletProfileEntityQuerySortBy
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QSortBy> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByAlg() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'alg', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByAlgDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'alg', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByBalanceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByGroupNames() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'groupNames', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByGroupNamesDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'groupNames', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByPubkeyHex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByPubkeyHexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySignMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'signMode', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySignModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'signMode', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySs58() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortBySs58Desc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletIcon() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIcon', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletIconDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIcon', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      sortByWalletNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.desc);
    });
  }
}

extension WalletProfileEntityQuerySortThenBy
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QSortThenBy> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByAlg() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'alg', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByAlgDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'alg', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByBalanceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByGroupNames() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'groupNames', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByGroupNamesDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'groupNames', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByPubkeyHex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByPubkeyHexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySignMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'signMode', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySignModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'signMode', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySs58() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenBySs58Desc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletIcon() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIcon', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletIconDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIcon', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.asc);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QAfterSortBy>
      thenByWalletNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.desc);
    });
  }
}

extension WalletProfileEntityQueryWhereDistinct
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct> {
  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByAddress({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'address', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByAlg({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'alg', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'balance');
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'createdAtMillis');
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByGroupNames({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'groupNames', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByPubkeyHex({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'pubkeyHex', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctBySignMode({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'signMode', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sortOrder');
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctBySource({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'source', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctBySs58() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'ss58');
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByWalletIcon({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletIcon', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletIndex');
    });
  }

  QueryBuilder<WalletProfileEntity, WalletProfileEntity, QDistinct>
      distinctByWalletName({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletName', caseSensitive: caseSensitive);
    });
  }
}

extension WalletProfileEntityQueryProperty
    on QueryBuilder<WalletProfileEntity, WalletProfileEntity, QQueryProperty> {
  QueryBuilder<WalletProfileEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      addressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'address');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations> algProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'alg');
    });
  }

  QueryBuilder<WalletProfileEntity, double, QQueryOperations>
      balanceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'balance');
    });
  }

  QueryBuilder<WalletProfileEntity, int, QQueryOperations>
      createdAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'createdAtMillis');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      groupNamesProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'groupNames');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      pubkeyHexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'pubkeyHex');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      signModeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'signMode');
    });
  }

  QueryBuilder<WalletProfileEntity, int, QQueryOperations> sortOrderProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sortOrder');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations> sourceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'source');
    });
  }

  QueryBuilder<WalletProfileEntity, int, QQueryOperations> ss58Property() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'ss58');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      walletIconProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletIcon');
    });
  }

  QueryBuilder<WalletProfileEntity, int, QQueryOperations>
      walletIndexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletIndex');
    });
  }

  QueryBuilder<WalletProfileEntity, String, QQueryOperations>
      walletNameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletName');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetWalletGroupEntityCollection on Isar {
  IsarCollection<WalletGroupEntity> get walletGroupEntitys => this.collection();
}

const WalletGroupEntitySchema = CollectionSchema(
  name: r'WalletGroupEntity',
  id: -5034756919696174632,
  properties: {
    r'isDefault': PropertySchema(
      id: 0,
      name: r'isDefault',
      type: IsarType.bool,
    ),
    r'name': PropertySchema(
      id: 1,
      name: r'name',
      type: IsarType.string,
    ),
    r'sortOrder': PropertySchema(
      id: 2,
      name: r'sortOrder',
      type: IsarType.long,
    )
  },
  estimateSize: _walletGroupEntityEstimateSize,
  serialize: _walletGroupEntitySerialize,
  deserialize: _walletGroupEntityDeserialize,
  deserializeProp: _walletGroupEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'name': IndexSchema(
      id: 879695947855722453,
      name: r'name',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'name',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _walletGroupEntityGetId,
  getLinks: _walletGroupEntityGetLinks,
  attach: _walletGroupEntityAttach,
  version: '3.1.0+1',
);

int _walletGroupEntityEstimateSize(
  WalletGroupEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.name.length * 3;
  return bytesCount;
}

void _walletGroupEntitySerialize(
  WalletGroupEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeBool(offsets[0], object.isDefault);
  writer.writeString(offsets[1], object.name);
  writer.writeLong(offsets[2], object.sortOrder);
}

WalletGroupEntity _walletGroupEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = WalletGroupEntity();
  object.id = id;
  object.isDefault = reader.readBool(offsets[0]);
  object.name = reader.readString(offsets[1]);
  object.sortOrder = reader.readLong(offsets[2]);
  return object;
}

P _walletGroupEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readBool(offset)) as P;
    case 1:
      return (reader.readString(offset)) as P;
    case 2:
      return (reader.readLong(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _walletGroupEntityGetId(WalletGroupEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _walletGroupEntityGetLinks(
    WalletGroupEntity object) {
  return [];
}

void _walletGroupEntityAttach(
    IsarCollection<dynamic> col, Id id, WalletGroupEntity object) {
  object.id = id;
}

extension WalletGroupEntityByIndex on IsarCollection<WalletGroupEntity> {
  Future<WalletGroupEntity?> getByName(String name) {
    return getByIndex(r'name', [name]);
  }

  WalletGroupEntity? getByNameSync(String name) {
    return getByIndexSync(r'name', [name]);
  }

  Future<bool> deleteByName(String name) {
    return deleteByIndex(r'name', [name]);
  }

  bool deleteByNameSync(String name) {
    return deleteByIndexSync(r'name', [name]);
  }

  Future<List<WalletGroupEntity?>> getAllByName(List<String> nameValues) {
    final values = nameValues.map((e) => [e]).toList();
    return getAllByIndex(r'name', values);
  }

  List<WalletGroupEntity?> getAllByNameSync(List<String> nameValues) {
    final values = nameValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'name', values);
  }

  Future<int> deleteAllByName(List<String> nameValues) {
    final values = nameValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'name', values);
  }

  int deleteAllByNameSync(List<String> nameValues) {
    final values = nameValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'name', values);
  }

  Future<Id> putByName(WalletGroupEntity object) {
    return putByIndex(r'name', object);
  }

  Id putByNameSync(WalletGroupEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'name', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByName(List<WalletGroupEntity> objects) {
    return putAllByIndex(r'name', objects);
  }

  List<Id> putAllByNameSync(List<WalletGroupEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'name', objects, saveLinks: saveLinks);
  }
}

extension WalletGroupEntityQueryWhereSort
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QWhere> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension WalletGroupEntityQueryWhere
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QWhereClause> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      idNotEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: lowerId,
        includeLower: includeLower,
        upper: upperId,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      nameEqualTo(String name) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'name',
        value: [name],
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterWhereClause>
      nameNotEqualTo(String name) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'name',
              lower: [],
              upper: [name],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'name',
              lower: [name],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'name',
              lower: [name],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'name',
              lower: [],
              upper: [name],
              includeUpper: false,
            ));
      }
    });
  }
}

extension WalletGroupEntityQueryFilter
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QFilterCondition> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      idGreaterThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      idLessThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'id',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      isDefaultEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'isDefault',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'name',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'name',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'name',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      nameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'name',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      sortOrderEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      sortOrderGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      sortOrderLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterFilterCondition>
      sortOrderBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'sortOrder',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }
}

extension WalletGroupEntityQueryObject
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QFilterCondition> {}

extension WalletGroupEntityQueryLinks
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QFilterCondition> {}

extension WalletGroupEntityQuerySortBy
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QSortBy> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortByIsDefault() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'isDefault', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortByIsDefaultDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'isDefault', Sort.desc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      sortBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }
}

extension WalletGroupEntityQuerySortThenBy
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QSortThenBy> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenByIsDefault() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'isDefault', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenByIsDefaultDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'isDefault', Sort.desc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QAfterSortBy>
      thenBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }
}

extension WalletGroupEntityQueryWhereDistinct
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QDistinct> {
  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QDistinct>
      distinctByIsDefault() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'isDefault');
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QDistinct> distinctByName(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'name', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletGroupEntity, WalletGroupEntity, QDistinct>
      distinctBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sortOrder');
    });
  }
}

extension WalletGroupEntityQueryProperty
    on QueryBuilder<WalletGroupEntity, WalletGroupEntity, QQueryProperty> {
  QueryBuilder<WalletGroupEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<WalletGroupEntity, bool, QQueryOperations> isDefaultProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'isDefault');
    });
  }

  QueryBuilder<WalletGroupEntity, String, QQueryOperations> nameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'name');
    });
  }

  QueryBuilder<WalletGroupEntity, int, QQueryOperations> sortOrderProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sortOrder');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetWalletSettingsEntityCollection on Isar {
  IsarCollection<WalletSettingsEntity> get walletSettingsEntitys =>
      this.collection();
}

const WalletSettingsEntitySchema = CollectionSchema(
  name: r'WalletSettingsEntity',
  id: 3556928265447228666,
  properties: {
    r'activeWalletIndex': PropertySchema(
      id: 0,
      name: r'activeWalletIndex',
      type: IsarType.long,
    ),
    r'updatedAtMillis': PropertySchema(
      id: 1,
      name: r'updatedAtMillis',
      type: IsarType.long,
    )
  },
  estimateSize: _walletSettingsEntityEstimateSize,
  serialize: _walletSettingsEntitySerialize,
  deserialize: _walletSettingsEntityDeserialize,
  deserializeProp: _walletSettingsEntityDeserializeProp,
  idName: r'id',
  indexes: {},
  links: {},
  embeddedSchemas: {},
  getId: _walletSettingsEntityGetId,
  getLinks: _walletSettingsEntityGetLinks,
  attach: _walletSettingsEntityAttach,
  version: '3.1.0+1',
);

int _walletSettingsEntityEstimateSize(
  WalletSettingsEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  return bytesCount;
}

void _walletSettingsEntitySerialize(
  WalletSettingsEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeLong(offsets[0], object.activeWalletIndex);
  writer.writeLong(offsets[1], object.updatedAtMillis);
}

WalletSettingsEntity _walletSettingsEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = WalletSettingsEntity();
  object.activeWalletIndex = reader.readLongOrNull(offsets[0]);
  object.id = id;
  object.updatedAtMillis = reader.readLong(offsets[1]);
  return object;
}

P _walletSettingsEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readLongOrNull(offset)) as P;
    case 1:
      return (reader.readLong(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _walletSettingsEntityGetId(WalletSettingsEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _walletSettingsEntityGetLinks(
    WalletSettingsEntity object) {
  return [];
}

void _walletSettingsEntityAttach(
    IsarCollection<dynamic> col, Id id, WalletSettingsEntity object) {
  object.id = id;
}

extension WalletSettingsEntityQueryWhereSort
    on QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QWhere> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhere>
      anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension WalletSettingsEntityQueryWhere
    on QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QWhereClause> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhereClause>
      idNotEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterWhereClause>
      idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: lowerId,
        includeLower: includeLower,
        upper: upperId,
        includeUpper: includeUpper,
      ));
    });
  }
}

extension WalletSettingsEntityQueryFilter on QueryBuilder<WalletSettingsEntity,
    WalletSettingsEntity, QFilterCondition> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'activeWalletIndex',
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'activeWalletIndex',
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'activeWalletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexGreaterThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'activeWalletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexLessThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'activeWalletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> activeWalletIndexBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'activeWalletIndex',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> idGreaterThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> idLessThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'id',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> updatedAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'updatedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> updatedAtMillisGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'updatedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> updatedAtMillisLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'updatedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity,
      QAfterFilterCondition> updatedAtMillisBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'updatedAtMillis',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }
}

extension WalletSettingsEntityQueryObject on QueryBuilder<WalletSettingsEntity,
    WalletSettingsEntity, QFilterCondition> {}

extension WalletSettingsEntityQueryLinks on QueryBuilder<WalletSettingsEntity,
    WalletSettingsEntity, QFilterCondition> {}

extension WalletSettingsEntityQuerySortBy
    on QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QSortBy> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      sortByActiveWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'activeWalletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      sortByActiveWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'activeWalletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      sortByUpdatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      sortByUpdatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAtMillis', Sort.desc);
    });
  }
}

extension WalletSettingsEntityQuerySortThenBy
    on QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QSortThenBy> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenByActiveWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'activeWalletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenByActiveWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'activeWalletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenByUpdatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QAfterSortBy>
      thenByUpdatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAtMillis', Sort.desc);
    });
  }
}

extension WalletSettingsEntityQueryWhereDistinct
    on QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QDistinct> {
  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QDistinct>
      distinctByActiveWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'activeWalletIndex');
    });
  }

  QueryBuilder<WalletSettingsEntity, WalletSettingsEntity, QDistinct>
      distinctByUpdatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'updatedAtMillis');
    });
  }
}

extension WalletSettingsEntityQueryProperty on QueryBuilder<
    WalletSettingsEntity, WalletSettingsEntity, QQueryProperty> {
  QueryBuilder<WalletSettingsEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<WalletSettingsEntity, int?, QQueryOperations>
      activeWalletIndexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'activeWalletIndex');
    });
  }

  QueryBuilder<WalletSettingsEntity, int, QQueryOperations>
      updatedAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'updatedAtMillis');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetAppKvEntityCollection on Isar {
  IsarCollection<AppKvEntity> get appKvEntitys => this.collection();
}

const AppKvEntitySchema = CollectionSchema(
  name: r'AppKvEntity',
  id: -4757328183228885293,
  properties: {
    r'boolValue': PropertySchema(
      id: 0,
      name: r'boolValue',
      type: IsarType.bool,
    ),
    r'intValue': PropertySchema(
      id: 1,
      name: r'intValue',
      type: IsarType.long,
    ),
    r'key': PropertySchema(
      id: 2,
      name: r'key',
      type: IsarType.string,
    ),
    r'stringValue': PropertySchema(
      id: 3,
      name: r'stringValue',
      type: IsarType.string,
    )
  },
  estimateSize: _appKvEntityEstimateSize,
  serialize: _appKvEntitySerialize,
  deserialize: _appKvEntityDeserialize,
  deserializeProp: _appKvEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'key': IndexSchema(
      id: -4906094122524121629,
      name: r'key',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'key',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _appKvEntityGetId,
  getLinks: _appKvEntityGetLinks,
  attach: _appKvEntityAttach,
  version: '3.1.0+1',
);

int _appKvEntityEstimateSize(
  AppKvEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.key.length * 3;
  {
    final value = object.stringValue;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  return bytesCount;
}

void _appKvEntitySerialize(
  AppKvEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeBool(offsets[0], object.boolValue);
  writer.writeLong(offsets[1], object.intValue);
  writer.writeString(offsets[2], object.key);
  writer.writeString(offsets[3], object.stringValue);
}

AppKvEntity _appKvEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = AppKvEntity();
  object.boolValue = reader.readBoolOrNull(offsets[0]);
  object.id = id;
  object.intValue = reader.readLongOrNull(offsets[1]);
  object.key = reader.readString(offsets[2]);
  object.stringValue = reader.readStringOrNull(offsets[3]);
  return object;
}

P _appKvEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readBoolOrNull(offset)) as P;
    case 1:
      return (reader.readLongOrNull(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    case 3:
      return (reader.readStringOrNull(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _appKvEntityGetId(AppKvEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _appKvEntityGetLinks(AppKvEntity object) {
  return [];
}

void _appKvEntityAttach(
    IsarCollection<dynamic> col, Id id, AppKvEntity object) {
  object.id = id;
}

extension AppKvEntityByIndex on IsarCollection<AppKvEntity> {
  Future<AppKvEntity?> getByKey(String key) {
    return getByIndex(r'key', [key]);
  }

  AppKvEntity? getByKeySync(String key) {
    return getByIndexSync(r'key', [key]);
  }

  Future<bool> deleteByKey(String key) {
    return deleteByIndex(r'key', [key]);
  }

  bool deleteByKeySync(String key) {
    return deleteByIndexSync(r'key', [key]);
  }

  Future<List<AppKvEntity?>> getAllByKey(List<String> keyValues) {
    final values = keyValues.map((e) => [e]).toList();
    return getAllByIndex(r'key', values);
  }

  List<AppKvEntity?> getAllByKeySync(List<String> keyValues) {
    final values = keyValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'key', values);
  }

  Future<int> deleteAllByKey(List<String> keyValues) {
    final values = keyValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'key', values);
  }

  int deleteAllByKeySync(List<String> keyValues) {
    final values = keyValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'key', values);
  }

  Future<Id> putByKey(AppKvEntity object) {
    return putByIndex(r'key', object);
  }

  Id putByKeySync(AppKvEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'key', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByKey(List<AppKvEntity> objects) {
    return putAllByIndex(r'key', objects);
  }

  List<Id> putAllByKeySync(List<AppKvEntity> objects, {bool saveLinks = true}) {
    return putAllByIndexSync(r'key', objects, saveLinks: saveLinks);
  }
}

extension AppKvEntityQueryWhereSort
    on QueryBuilder<AppKvEntity, AppKvEntity, QWhere> {
  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension AppKvEntityQueryWhere
    on QueryBuilder<AppKvEntity, AppKvEntity, QWhereClause> {
  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> idNotEqualTo(
      Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> idGreaterThan(Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> idLessThan(Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: lowerId,
        includeLower: includeLower,
        upper: upperId,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> keyEqualTo(
      String key) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'key',
        value: [key],
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterWhereClause> keyNotEqualTo(
      String key) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'key',
              lower: [],
              upper: [key],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'key',
              lower: [key],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'key',
              lower: [key],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'key',
              lower: [],
              upper: [key],
              includeUpper: false,
            ));
      }
    });
  }
}

extension AppKvEntityQueryFilter
    on QueryBuilder<AppKvEntity, AppKvEntity, QFilterCondition> {
  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      boolValueIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'boolValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      boolValueIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'boolValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      boolValueEqualTo(bool? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'boolValue',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> idEqualTo(
      Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> idGreaterThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> idLessThan(
    Id value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'id',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      intValueIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'intValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      intValueIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'intValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> intValueEqualTo(
      int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'intValue',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      intValueGreaterThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'intValue',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      intValueLessThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'intValue',
        value: value,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> intValueBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'intValue',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'key',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyContains(
      String value,
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'key',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyMatches(
      String pattern,
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'key',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition> keyIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'key',
        value: '',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      keyIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'key',
        value: '',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'stringValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'stringValue',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'stringValue',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'stringValue',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'stringValue',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'stringValue',
        value: '',
      ));
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterFilterCondition>
      stringValueIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'stringValue',
        value: '',
      ));
    });
  }
}

extension AppKvEntityQueryObject
    on QueryBuilder<AppKvEntity, AppKvEntity, QFilterCondition> {}

extension AppKvEntityQueryLinks
    on QueryBuilder<AppKvEntity, AppKvEntity, QFilterCondition> {}

extension AppKvEntityQuerySortBy
    on QueryBuilder<AppKvEntity, AppKvEntity, QSortBy> {
  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByBoolValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'boolValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByBoolValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'boolValue', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByIntValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'intValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByIntValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'intValue', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByKey() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'key', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByKeyDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'key', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByStringValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'stringValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> sortByStringValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'stringValue', Sort.desc);
    });
  }
}

extension AppKvEntityQuerySortThenBy
    on QueryBuilder<AppKvEntity, AppKvEntity, QSortThenBy> {
  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByBoolValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'boolValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByBoolValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'boolValue', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByIntValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'intValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByIntValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'intValue', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByKey() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'key', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByKeyDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'key', Sort.desc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByStringValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'stringValue', Sort.asc);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QAfterSortBy> thenByStringValueDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'stringValue', Sort.desc);
    });
  }
}

extension AppKvEntityQueryWhereDistinct
    on QueryBuilder<AppKvEntity, AppKvEntity, QDistinct> {
  QueryBuilder<AppKvEntity, AppKvEntity, QDistinct> distinctByBoolValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'boolValue');
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QDistinct> distinctByIntValue() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'intValue');
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QDistinct> distinctByKey(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'key', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppKvEntity, AppKvEntity, QDistinct> distinctByStringValue(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'stringValue', caseSensitive: caseSensitive);
    });
  }
}

extension AppKvEntityQueryProperty
    on QueryBuilder<AppKvEntity, AppKvEntity, QQueryProperty> {
  QueryBuilder<AppKvEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<AppKvEntity, bool?, QQueryOperations> boolValueProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'boolValue');
    });
  }

  QueryBuilder<AppKvEntity, int?, QQueryOperations> intValueProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'intValue');
    });
  }

  QueryBuilder<AppKvEntity, String, QQueryOperations> keyProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'key');
    });
  }

  QueryBuilder<AppKvEntity, String?, QQueryOperations> stringValueProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'stringValue');
    });
  }
}
