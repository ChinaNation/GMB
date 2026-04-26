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
    r'pubkeyHex': PropertySchema(
      id: 4,
      name: r'pubkeyHex',
      type: IsarType.string,
    ),
    r'signMode': PropertySchema(
      id: 5,
      name: r'signMode',
      type: IsarType.string,
    ),
    r'sortOrder': PropertySchema(
      id: 6,
      name: r'sortOrder',
      type: IsarType.long,
    ),
    r'source': PropertySchema(
      id: 7,
      name: r'source',
      type: IsarType.string,
    ),
    r'ss58': PropertySchema(
      id: 8,
      name: r'ss58',
      type: IsarType.long,
    ),
    r'walletIcon': PropertySchema(
      id: 9,
      name: r'walletIcon',
      type: IsarType.string,
    ),
    r'walletIndex': PropertySchema(
      id: 10,
      name: r'walletIndex',
      type: IsarType.long,
    ),
    r'walletName': PropertySchema(
      id: 11,
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
  writer.writeString(offsets[4], object.pubkeyHex);
  writer.writeString(offsets[5], object.signMode);
  writer.writeLong(offsets[6], object.sortOrder);
  writer.writeString(offsets[7], object.source);
  writer.writeLong(offsets[8], object.ss58);
  writer.writeString(offsets[9], object.walletIcon);
  writer.writeLong(offsets[10], object.walletIndex);
  writer.writeString(offsets[11], object.walletName);
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
  object.id = id;
  object.pubkeyHex = reader.readString(offsets[4]);
  object.signMode = reader.readString(offsets[5]);
  object.sortOrder = reader.readLong(offsets[6]);
  object.source = reader.readString(offsets[7]);
  object.ss58 = reader.readLong(offsets[8]);
  object.walletIcon = reader.readString(offsets[9]);
  object.walletIndex = reader.readLong(offsets[10]);
  object.walletName = reader.readString(offsets[11]);
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
      return (reader.readLong(offset)) as P;
    case 7:
      return (reader.readString(offset)) as P;
    case 8:
      return (reader.readLong(offset)) as P;
    case 9:
      return (reader.readString(offset)) as P;
    case 10:
      return (reader.readLong(offset)) as P;
    case 11:
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

extension GetAdminRoleCacheEntityCollection on Isar {
  IsarCollection<AdminRoleCacheEntity> get adminRoleCacheEntitys =>
      this.collection();
}

const AdminRoleCacheEntitySchema = CollectionSchema(
  name: r'AdminRoleCacheEntity',
  id: -7398263961586602634,
  properties: {
    r'pubkeyHex': PropertySchema(
      id: 0,
      name: r'pubkeyHex',
      type: IsarType.string,
    ),
    r'roleName': PropertySchema(
      id: 1,
      name: r'roleName',
      type: IsarType.string,
    ),
    r'updatedAt': PropertySchema(
      id: 2,
      name: r'updatedAt',
      type: IsarType.long,
    )
  },
  estimateSize: _adminRoleCacheEntityEstimateSize,
  serialize: _adminRoleCacheEntitySerialize,
  deserialize: _adminRoleCacheEntityDeserialize,
  deserializeProp: _adminRoleCacheEntityDeserializeProp,
  idName: r'id',
  indexes: {
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
    ),
    r'updatedAt': IndexSchema(
      id: -6238191080293565125,
      name: r'updatedAt',
      unique: false,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'updatedAt',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _adminRoleCacheEntityGetId,
  getLinks: _adminRoleCacheEntityGetLinks,
  attach: _adminRoleCacheEntityAttach,
  version: '3.1.0+1',
);

int _adminRoleCacheEntityEstimateSize(
  AdminRoleCacheEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.pubkeyHex.length * 3;
  bytesCount += 3 + object.roleName.length * 3;
  return bytesCount;
}

void _adminRoleCacheEntitySerialize(
  AdminRoleCacheEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeString(offsets[0], object.pubkeyHex);
  writer.writeString(offsets[1], object.roleName);
  writer.writeLong(offsets[2], object.updatedAt);
}

AdminRoleCacheEntity _adminRoleCacheEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = AdminRoleCacheEntity();
  object.id = id;
  object.pubkeyHex = reader.readString(offsets[0]);
  object.roleName = reader.readString(offsets[1]);
  object.updatedAt = reader.readLong(offsets[2]);
  return object;
}

P _adminRoleCacheEntityDeserializeProp<P>(
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
      return (reader.readLong(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _adminRoleCacheEntityGetId(AdminRoleCacheEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _adminRoleCacheEntityGetLinks(
    AdminRoleCacheEntity object) {
  return [];
}

void _adminRoleCacheEntityAttach(
    IsarCollection<dynamic> col, Id id, AdminRoleCacheEntity object) {
  object.id = id;
}

extension AdminRoleCacheEntityByIndex on IsarCollection<AdminRoleCacheEntity> {
  Future<AdminRoleCacheEntity?> getByPubkeyHex(String pubkeyHex) {
    return getByIndex(r'pubkeyHex', [pubkeyHex]);
  }

  AdminRoleCacheEntity? getByPubkeyHexSync(String pubkeyHex) {
    return getByIndexSync(r'pubkeyHex', [pubkeyHex]);
  }

  Future<bool> deleteByPubkeyHex(String pubkeyHex) {
    return deleteByIndex(r'pubkeyHex', [pubkeyHex]);
  }

  bool deleteByPubkeyHexSync(String pubkeyHex) {
    return deleteByIndexSync(r'pubkeyHex', [pubkeyHex]);
  }

  Future<List<AdminRoleCacheEntity?>> getAllByPubkeyHex(
      List<String> pubkeyHexValues) {
    final values = pubkeyHexValues.map((e) => [e]).toList();
    return getAllByIndex(r'pubkeyHex', values);
  }

  List<AdminRoleCacheEntity?> getAllByPubkeyHexSync(
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

  Future<Id> putByPubkeyHex(AdminRoleCacheEntity object) {
    return putByIndex(r'pubkeyHex', object);
  }

  Id putByPubkeyHexSync(AdminRoleCacheEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'pubkeyHex', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByPubkeyHex(List<AdminRoleCacheEntity> objects) {
    return putAllByIndex(r'pubkeyHex', objects);
  }

  List<Id> putAllByPubkeyHexSync(List<AdminRoleCacheEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'pubkeyHex', objects, saveLinks: saveLinks);
  }
}

extension AdminRoleCacheEntityQueryWhereSort
    on QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QWhere> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhere>
      anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhere>
      anyUpdatedAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'updatedAt'),
      );
    });
  }
}

extension AdminRoleCacheEntityQueryWhere
    on QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QWhereClause> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      pubkeyHexEqualTo(String pubkeyHex) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'pubkeyHex',
        value: [pubkeyHex],
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      updatedAtEqualTo(int updatedAt) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'updatedAt',
        value: [updatedAt],
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      updatedAtNotEqualTo(int updatedAt) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'updatedAt',
              lower: [],
              upper: [updatedAt],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'updatedAt',
              lower: [updatedAt],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'updatedAt',
              lower: [updatedAt],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'updatedAt',
              lower: [],
              upper: [updatedAt],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      updatedAtGreaterThan(
    int updatedAt, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'updatedAt',
        lower: [updatedAt],
        includeLower: include,
        upper: [],
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      updatedAtLessThan(
    int updatedAt, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'updatedAt',
        lower: [],
        upper: [updatedAt],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterWhereClause>
      updatedAtBetween(
    int lowerUpdatedAt,
    int upperUpdatedAt, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'updatedAt',
        lower: [lowerUpdatedAt],
        includeLower: includeLower,
        upper: [upperUpdatedAt],
        includeUpper: includeUpper,
      ));
    });
  }
}

extension AdminRoleCacheEntityQueryFilter on QueryBuilder<AdminRoleCacheEntity,
    AdminRoleCacheEntity, QFilterCondition> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexEqualTo(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexGreaterThan(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexLessThan(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexBetween(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexStartsWith(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexEndsWith(
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

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
          QAfterFilterCondition>
      pubkeyHexContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'pubkeyHex',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
          QAfterFilterCondition>
      pubkeyHexMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'pubkeyHex',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'pubkeyHex',
        value: '',
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> pubkeyHexIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'pubkeyHex',
        value: '',
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'roleName',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
          QAfterFilterCondition>
      roleNameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'roleName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
          QAfterFilterCondition>
      roleNameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'roleName',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'roleName',
        value: '',
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> roleNameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'roleName',
        value: '',
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> updatedAtEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'updatedAt',
        value: value,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> updatedAtGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'updatedAt',
        value: value,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> updatedAtLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'updatedAt',
        value: value,
      ));
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity,
      QAfterFilterCondition> updatedAtBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'updatedAt',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }
}

extension AdminRoleCacheEntityQueryObject on QueryBuilder<AdminRoleCacheEntity,
    AdminRoleCacheEntity, QFilterCondition> {}

extension AdminRoleCacheEntityQueryLinks on QueryBuilder<AdminRoleCacheEntity,
    AdminRoleCacheEntity, QFilterCondition> {}

extension AdminRoleCacheEntityQuerySortBy
    on QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QSortBy> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByPubkeyHex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByPubkeyHexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.desc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByRoleName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'roleName', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByRoleNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'roleName', Sort.desc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByUpdatedAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAt', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      sortByUpdatedAtDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAt', Sort.desc);
    });
  }
}

extension AdminRoleCacheEntityQuerySortThenBy
    on QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QSortThenBy> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByPubkeyHex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByPubkeyHexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pubkeyHex', Sort.desc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByRoleName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'roleName', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByRoleNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'roleName', Sort.desc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByUpdatedAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAt', Sort.asc);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QAfterSortBy>
      thenByUpdatedAtDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'updatedAt', Sort.desc);
    });
  }
}

extension AdminRoleCacheEntityQueryWhereDistinct
    on QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QDistinct> {
  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QDistinct>
      distinctByPubkeyHex({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'pubkeyHex', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QDistinct>
      distinctByRoleName({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'roleName', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AdminRoleCacheEntity, AdminRoleCacheEntity, QDistinct>
      distinctByUpdatedAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'updatedAt');
    });
  }
}

extension AdminRoleCacheEntityQueryProperty on QueryBuilder<
    AdminRoleCacheEntity, AdminRoleCacheEntity, QQueryProperty> {
  QueryBuilder<AdminRoleCacheEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<AdminRoleCacheEntity, String, QQueryOperations>
      pubkeyHexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'pubkeyHex');
    });
  }

  QueryBuilder<AdminRoleCacheEntity, String, QQueryOperations>
      roleNameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'roleName');
    });
  }

  QueryBuilder<AdminRoleCacheEntity, int, QQueryOperations>
      updatedAtProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'updatedAt');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetObservedAccountEntityCollection on Isar {
  IsarCollection<ObservedAccountEntity> get observedAccountEntitys =>
      this.collection();
}

const ObservedAccountEntitySchema = CollectionSchema(
  name: r'ObservedAccountEntity',
  id: -4712203032392534857,
  properties: {
    r'accountId': PropertySchema(
      id: 0,
      name: r'accountId',
      type: IsarType.string,
    ),
    r'address': PropertySchema(
      id: 1,
      name: r'address',
      type: IsarType.string,
    ),
    r'balance': PropertySchema(
      id: 2,
      name: r'balance',
      type: IsarType.double,
    ),
    r'orgName': PropertySchema(
      id: 3,
      name: r'orgName',
      type: IsarType.string,
    ),
    r'publicKey': PropertySchema(
      id: 4,
      name: r'publicKey',
      type: IsarType.string,
    ),
    r'source': PropertySchema(
      id: 5,
      name: r'source',
      type: IsarType.string,
    )
  },
  estimateSize: _observedAccountEntityEstimateSize,
  serialize: _observedAccountEntitySerialize,
  deserialize: _observedAccountEntityDeserialize,
  deserializeProp: _observedAccountEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'accountId': IndexSchema(
      id: -1591555361937770434,
      name: r'accountId',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'accountId',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _observedAccountEntityGetId,
  getLinks: _observedAccountEntityGetLinks,
  attach: _observedAccountEntityAttach,
  version: '3.1.0+1',
);

int _observedAccountEntityEstimateSize(
  ObservedAccountEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.accountId.length * 3;
  bytesCount += 3 + object.address.length * 3;
  bytesCount += 3 + object.orgName.length * 3;
  bytesCount += 3 + object.publicKey.length * 3;
  bytesCount += 3 + object.source.length * 3;
  return bytesCount;
}

void _observedAccountEntitySerialize(
  ObservedAccountEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeString(offsets[0], object.accountId);
  writer.writeString(offsets[1], object.address);
  writer.writeDouble(offsets[2], object.balance);
  writer.writeString(offsets[3], object.orgName);
  writer.writeString(offsets[4], object.publicKey);
  writer.writeString(offsets[5], object.source);
}

ObservedAccountEntity _observedAccountEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = ObservedAccountEntity();
  object.accountId = reader.readString(offsets[0]);
  object.address = reader.readString(offsets[1]);
  object.balance = reader.readDoubleOrNull(offsets[2]);
  object.id = id;
  object.orgName = reader.readString(offsets[3]);
  object.publicKey = reader.readString(offsets[4]);
  object.source = reader.readString(offsets[5]);
  return object;
}

P _observedAccountEntityDeserializeProp<P>(
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
      return (reader.readDoubleOrNull(offset)) as P;
    case 3:
      return (reader.readString(offset)) as P;
    case 4:
      return (reader.readString(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _observedAccountEntityGetId(ObservedAccountEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _observedAccountEntityGetLinks(
    ObservedAccountEntity object) {
  return [];
}

void _observedAccountEntityAttach(
    IsarCollection<dynamic> col, Id id, ObservedAccountEntity object) {
  object.id = id;
}

extension ObservedAccountEntityByIndex
    on IsarCollection<ObservedAccountEntity> {
  Future<ObservedAccountEntity?> getByAccountId(String accountId) {
    return getByIndex(r'accountId', [accountId]);
  }

  ObservedAccountEntity? getByAccountIdSync(String accountId) {
    return getByIndexSync(r'accountId', [accountId]);
  }

  Future<bool> deleteByAccountId(String accountId) {
    return deleteByIndex(r'accountId', [accountId]);
  }

  bool deleteByAccountIdSync(String accountId) {
    return deleteByIndexSync(r'accountId', [accountId]);
  }

  Future<List<ObservedAccountEntity?>> getAllByAccountId(
      List<String> accountIdValues) {
    final values = accountIdValues.map((e) => [e]).toList();
    return getAllByIndex(r'accountId', values);
  }

  List<ObservedAccountEntity?> getAllByAccountIdSync(
      List<String> accountIdValues) {
    final values = accountIdValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'accountId', values);
  }

  Future<int> deleteAllByAccountId(List<String> accountIdValues) {
    final values = accountIdValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'accountId', values);
  }

  int deleteAllByAccountIdSync(List<String> accountIdValues) {
    final values = accountIdValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'accountId', values);
  }

  Future<Id> putByAccountId(ObservedAccountEntity object) {
    return putByIndex(r'accountId', object);
  }

  Id putByAccountIdSync(ObservedAccountEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'accountId', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByAccountId(List<ObservedAccountEntity> objects) {
    return putAllByIndex(r'accountId', objects);
  }

  List<Id> putAllByAccountIdSync(List<ObservedAccountEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'accountId', objects, saveLinks: saveLinks);
  }
}

extension ObservedAccountEntityQueryWhereSort
    on QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QWhere> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhere>
      anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension ObservedAccountEntityQueryWhere on QueryBuilder<ObservedAccountEntity,
    ObservedAccountEntity, QWhereClause> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
      accountIdEqualTo(String accountId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'accountId',
        value: [accountId],
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterWhereClause>
      accountIdNotEqualTo(String accountId) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'accountId',
              lower: [],
              upper: [accountId],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'accountId',
              lower: [accountId],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'accountId',
              lower: [accountId],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'accountId',
              lower: [],
              upper: [accountId],
              includeUpper: false,
            ));
      }
    });
  }
}

extension ObservedAccountEntityQueryFilter on QueryBuilder<
    ObservedAccountEntity, ObservedAccountEntity, QFilterCondition> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'accountId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      accountIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      accountIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'accountId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountId',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> accountIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'accountId',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressEqualTo(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressGreaterThan(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressLessThan(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressBetween(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressStartsWith(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressEndsWith(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      addressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      addressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'address',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'address',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> addressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'address',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'balance',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'balance',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceEqualTo(
    double? value, {
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceGreaterThan(
    double? value, {
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceLessThan(
    double? value, {
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> balanceBetween(
    double? lower,
    double? upper, {
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'orgName',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      orgNameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'orgName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      orgNameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'orgName',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'orgName',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> orgNameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'orgName',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'publicKey',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      publicKeyContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'publicKey',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      publicKeyMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'publicKey',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'publicKey',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> publicKeyIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'publicKey',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceEqualTo(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceGreaterThan(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceLessThan(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceBetween(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceStartsWith(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceEndsWith(
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

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      sourceContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
          QAfterFilterCondition>
      sourceMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'source',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'source',
        value: '',
      ));
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity,
      QAfterFilterCondition> sourceIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'source',
        value: '',
      ));
    });
  }
}

extension ObservedAccountEntityQueryObject on QueryBuilder<
    ObservedAccountEntity, ObservedAccountEntity, QFilterCondition> {}

extension ObservedAccountEntityQueryLinks on QueryBuilder<ObservedAccountEntity,
    ObservedAccountEntity, QFilterCondition> {}

extension ObservedAccountEntityQuerySortBy
    on QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QSortBy> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByAccountId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByAccountIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByBalanceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByOrgName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'orgName', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByOrgNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'orgName', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByPublicKey() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'publicKey', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortByPublicKeyDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'publicKey', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      sortBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }
}

extension ObservedAccountEntityQuerySortThenBy
    on QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QSortThenBy> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByAccountId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByAccountIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'address', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByBalanceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'balance', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByOrgName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'orgName', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByOrgNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'orgName', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByPublicKey() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'publicKey', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenByPublicKeyDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'publicKey', Sort.desc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QAfterSortBy>
      thenBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }
}

extension ObservedAccountEntityQueryWhereDistinct
    on QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct> {
  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctByAccountId({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'accountId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctByAddress({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'address', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctByBalance() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'balance');
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctByOrgName({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'orgName', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctByPublicKey({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'publicKey', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<ObservedAccountEntity, ObservedAccountEntity, QDistinct>
      distinctBySource({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'source', caseSensitive: caseSensitive);
    });
  }
}

extension ObservedAccountEntityQueryProperty on QueryBuilder<
    ObservedAccountEntity, ObservedAccountEntity, QQueryProperty> {
  QueryBuilder<ObservedAccountEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<ObservedAccountEntity, String, QQueryOperations>
      accountIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'accountId');
    });
  }

  QueryBuilder<ObservedAccountEntity, String, QQueryOperations>
      addressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'address');
    });
  }

  QueryBuilder<ObservedAccountEntity, double?, QQueryOperations>
      balanceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'balance');
    });
  }

  QueryBuilder<ObservedAccountEntity, String, QQueryOperations>
      orgNameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'orgName');
    });
  }

  QueryBuilder<ObservedAccountEntity, String, QQueryOperations>
      publicKeyProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'publicKey');
    });
  }

  QueryBuilder<ObservedAccountEntity, String, QQueryOperations>
      sourceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'source');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetLoginReplayEntityCollection on Isar {
  IsarCollection<LoginReplayEntity> get loginReplayEntitys => this.collection();
}

const LoginReplayEntitySchema = CollectionSchema(
  name: r'LoginReplayEntity',
  id: -6077716445208142447,
  properties: {
    r'expiresAt': PropertySchema(
      id: 0,
      name: r'expiresAt',
      type: IsarType.long,
    ),
    r'requestId': PropertySchema(
      id: 1,
      name: r'requestId',
      type: IsarType.string,
    )
  },
  estimateSize: _loginReplayEntityEstimateSize,
  serialize: _loginReplayEntitySerialize,
  deserialize: _loginReplayEntityDeserialize,
  deserializeProp: _loginReplayEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'requestId': IndexSchema(
      id: 938047444593699237,
      name: r'requestId',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'requestId',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _loginReplayEntityGetId,
  getLinks: _loginReplayEntityGetLinks,
  attach: _loginReplayEntityAttach,
  version: '3.1.0+1',
);

int _loginReplayEntityEstimateSize(
  LoginReplayEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.requestId.length * 3;
  return bytesCount;
}

void _loginReplayEntitySerialize(
  LoginReplayEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeLong(offsets[0], object.expiresAt);
  writer.writeString(offsets[1], object.requestId);
}

LoginReplayEntity _loginReplayEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = LoginReplayEntity();
  object.expiresAt = reader.readLong(offsets[0]);
  object.id = id;
  object.requestId = reader.readString(offsets[1]);
  return object;
}

P _loginReplayEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readLong(offset)) as P;
    case 1:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _loginReplayEntityGetId(LoginReplayEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _loginReplayEntityGetLinks(
    LoginReplayEntity object) {
  return [];
}

void _loginReplayEntityAttach(
    IsarCollection<dynamic> col, Id id, LoginReplayEntity object) {
  object.id = id;
}

extension LoginReplayEntityByIndex on IsarCollection<LoginReplayEntity> {
  Future<LoginReplayEntity?> getByRequestId(String requestId) {
    return getByIndex(r'requestId', [requestId]);
  }

  LoginReplayEntity? getByRequestIdSync(String requestId) {
    return getByIndexSync(r'requestId', [requestId]);
  }

  Future<bool> deleteByRequestId(String requestId) {
    return deleteByIndex(r'requestId', [requestId]);
  }

  bool deleteByRequestIdSync(String requestId) {
    return deleteByIndexSync(r'requestId', [requestId]);
  }

  Future<List<LoginReplayEntity?>> getAllByRequestId(
      List<String> requestIdValues) {
    final values = requestIdValues.map((e) => [e]).toList();
    return getAllByIndex(r'requestId', values);
  }

  List<LoginReplayEntity?> getAllByRequestIdSync(List<String> requestIdValues) {
    final values = requestIdValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'requestId', values);
  }

  Future<int> deleteAllByRequestId(List<String> requestIdValues) {
    final values = requestIdValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'requestId', values);
  }

  int deleteAllByRequestIdSync(List<String> requestIdValues) {
    final values = requestIdValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'requestId', values);
  }

  Future<Id> putByRequestId(LoginReplayEntity object) {
    return putByIndex(r'requestId', object);
  }

  Id putByRequestIdSync(LoginReplayEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'requestId', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByRequestId(List<LoginReplayEntity> objects) {
    return putAllByIndex(r'requestId', objects);
  }

  List<Id> putAllByRequestIdSync(List<LoginReplayEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'requestId', objects, saveLinks: saveLinks);
  }
}

extension LoginReplayEntityQueryWhereSort
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QWhere> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension LoginReplayEntityQueryWhere
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QWhereClause> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
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

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
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

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
      requestIdEqualTo(String requestId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'requestId',
        value: [requestId],
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterWhereClause>
      requestIdNotEqualTo(String requestId) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'requestId',
              lower: [],
              upper: [requestId],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'requestId',
              lower: [requestId],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'requestId',
              lower: [requestId],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'requestId',
              lower: [],
              upper: [requestId],
              includeUpper: false,
            ));
      }
    });
  }
}

extension LoginReplayEntityQueryFilter
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QFilterCondition> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      expiresAtEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'expiresAt',
        value: value,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      expiresAtGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'expiresAt',
        value: value,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      expiresAtLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'expiresAt',
        value: value,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      expiresAtBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'expiresAt',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
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

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
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

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
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

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'requestId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'requestId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'requestId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'requestId',
        value: '',
      ));
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterFilterCondition>
      requestIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'requestId',
        value: '',
      ));
    });
  }
}

extension LoginReplayEntityQueryObject
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QFilterCondition> {}

extension LoginReplayEntityQueryLinks
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QFilterCondition> {}

extension LoginReplayEntityQuerySortBy
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QSortBy> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      sortByExpiresAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'expiresAt', Sort.asc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      sortByExpiresAtDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'expiresAt', Sort.desc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      sortByRequestId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'requestId', Sort.asc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      sortByRequestIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'requestId', Sort.desc);
    });
  }
}

extension LoginReplayEntityQuerySortThenBy
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QSortThenBy> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      thenByExpiresAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'expiresAt', Sort.asc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      thenByExpiresAtDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'expiresAt', Sort.desc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      thenByRequestId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'requestId', Sort.asc);
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QAfterSortBy>
      thenByRequestIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'requestId', Sort.desc);
    });
  }
}

extension LoginReplayEntityQueryWhereDistinct
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QDistinct> {
  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QDistinct>
      distinctByExpiresAt() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'expiresAt');
    });
  }

  QueryBuilder<LoginReplayEntity, LoginReplayEntity, QDistinct>
      distinctByRequestId({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'requestId', caseSensitive: caseSensitive);
    });
  }
}

extension LoginReplayEntityQueryProperty
    on QueryBuilder<LoginReplayEntity, LoginReplayEntity, QQueryProperty> {
  QueryBuilder<LoginReplayEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<LoginReplayEntity, int, QQueryOperations> expiresAtProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'expiresAt');
    });
  }

  QueryBuilder<LoginReplayEntity, String, QQueryOperations>
      requestIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'requestId');
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

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetPersonalDuoqianEntityCollection on Isar {
  IsarCollection<PersonalDuoqianEntity> get personalDuoqianEntitys =>
      this.collection();
}

const PersonalDuoqianEntitySchema = CollectionSchema(
  name: r'PersonalDuoqianEntity',
  id: -8490877180663063815,
  properties: {
    r'addedAtMillis': PropertySchema(
      id: 0,
      name: r'addedAtMillis',
      type: IsarType.long,
    ),
    r'creatorAddress': PropertySchema(
      id: 1,
      name: r'creatorAddress',
      type: IsarType.string,
    ),
    r'duoqianAddress': PropertySchema(
      id: 2,
      name: r'duoqianAddress',
      type: IsarType.string,
    ),
    r'name': PropertySchema(
      id: 3,
      name: r'name',
      type: IsarType.string,
    )
  },
  estimateSize: _personalDuoqianEntityEstimateSize,
  serialize: _personalDuoqianEntitySerialize,
  deserialize: _personalDuoqianEntityDeserialize,
  deserializeProp: _personalDuoqianEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'duoqianAddress': IndexSchema(
      id: 3822211026917775041,
      name: r'duoqianAddress',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'duoqianAddress',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'addedAtMillis': IndexSchema(
      id: -1059979261930735929,
      name: r'addedAtMillis',
      unique: false,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'addedAtMillis',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _personalDuoqianEntityGetId,
  getLinks: _personalDuoqianEntityGetLinks,
  attach: _personalDuoqianEntityAttach,
  version: '3.1.0+1',
);

int _personalDuoqianEntityEstimateSize(
  PersonalDuoqianEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.creatorAddress.length * 3;
  bytesCount += 3 + object.duoqianAddress.length * 3;
  bytesCount += 3 + object.name.length * 3;
  return bytesCount;
}

void _personalDuoqianEntitySerialize(
  PersonalDuoqianEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeLong(offsets[0], object.addedAtMillis);
  writer.writeString(offsets[1], object.creatorAddress);
  writer.writeString(offsets[2], object.duoqianAddress);
  writer.writeString(offsets[3], object.name);
}

PersonalDuoqianEntity _personalDuoqianEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = PersonalDuoqianEntity();
  object.addedAtMillis = reader.readLong(offsets[0]);
  object.creatorAddress = reader.readString(offsets[1]);
  object.duoqianAddress = reader.readString(offsets[2]);
  object.id = id;
  object.name = reader.readString(offsets[3]);
  return object;
}

P _personalDuoqianEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readLong(offset)) as P;
    case 1:
      return (reader.readString(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    case 3:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _personalDuoqianEntityGetId(PersonalDuoqianEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _personalDuoqianEntityGetLinks(
    PersonalDuoqianEntity object) {
  return [];
}

void _personalDuoqianEntityAttach(
    IsarCollection<dynamic> col, Id id, PersonalDuoqianEntity object) {
  object.id = id;
}

extension PersonalDuoqianEntityByIndex
    on IsarCollection<PersonalDuoqianEntity> {
  Future<PersonalDuoqianEntity?> getByDuoqianAddress(String duoqianAddress) {
    return getByIndex(r'duoqianAddress', [duoqianAddress]);
  }

  PersonalDuoqianEntity? getByDuoqianAddressSync(String duoqianAddress) {
    return getByIndexSync(r'duoqianAddress', [duoqianAddress]);
  }

  Future<bool> deleteByDuoqianAddress(String duoqianAddress) {
    return deleteByIndex(r'duoqianAddress', [duoqianAddress]);
  }

  bool deleteByDuoqianAddressSync(String duoqianAddress) {
    return deleteByIndexSync(r'duoqianAddress', [duoqianAddress]);
  }

  Future<List<PersonalDuoqianEntity?>> getAllByDuoqianAddress(
      List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return getAllByIndex(r'duoqianAddress', values);
  }

  List<PersonalDuoqianEntity?> getAllByDuoqianAddressSync(
      List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'duoqianAddress', values);
  }

  Future<int> deleteAllByDuoqianAddress(List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'duoqianAddress', values);
  }

  int deleteAllByDuoqianAddressSync(List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'duoqianAddress', values);
  }

  Future<Id> putByDuoqianAddress(PersonalDuoqianEntity object) {
    return putByIndex(r'duoqianAddress', object);
  }

  Id putByDuoqianAddressSync(PersonalDuoqianEntity object,
      {bool saveLinks = true}) {
    return putByIndexSync(r'duoqianAddress', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByDuoqianAddress(List<PersonalDuoqianEntity> objects) {
    return putAllByIndex(r'duoqianAddress', objects);
  }

  List<Id> putAllByDuoqianAddressSync(List<PersonalDuoqianEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'duoqianAddress', objects, saveLinks: saveLinks);
  }
}

extension PersonalDuoqianEntityQueryWhereSort
    on QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QWhere> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhere>
      anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhere>
      anyAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'addedAtMillis'),
      );
    });
  }
}

extension PersonalDuoqianEntityQueryWhere on QueryBuilder<PersonalDuoqianEntity,
    PersonalDuoqianEntity, QWhereClause> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      duoqianAddressEqualTo(String duoqianAddress) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'duoqianAddress',
        value: [duoqianAddress],
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      duoqianAddressNotEqualTo(String duoqianAddress) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [],
              upper: [duoqianAddress],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [duoqianAddress],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [duoqianAddress],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [],
              upper: [duoqianAddress],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      addedAtMillisEqualTo(int addedAtMillis) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'addedAtMillis',
        value: [addedAtMillis],
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      addedAtMillisNotEqualTo(int addedAtMillis) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [],
              upper: [addedAtMillis],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [addedAtMillis],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [addedAtMillis],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [],
              upper: [addedAtMillis],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      addedAtMillisGreaterThan(
    int addedAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [addedAtMillis],
        includeLower: include,
        upper: [],
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      addedAtMillisLessThan(
    int addedAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [],
        upper: [addedAtMillis],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterWhereClause>
      addedAtMillisBetween(
    int lowerAddedAtMillis,
    int upperAddedAtMillis, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [lowerAddedAtMillis],
        includeLower: includeLower,
        upper: [upperAddedAtMillis],
        includeUpper: includeUpper,
      ));
    });
  }
}

extension PersonalDuoqianEntityQueryFilter on QueryBuilder<
    PersonalDuoqianEntity, PersonalDuoqianEntity, QFilterCondition> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> addedAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> addedAtMillisGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> addedAtMillisLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> addedAtMillisBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'addedAtMillis',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'creatorAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      creatorAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'creatorAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      creatorAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'creatorAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'creatorAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> creatorAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'creatorAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'duoqianAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      duoqianAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      duoqianAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'duoqianAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'duoqianAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> duoqianAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'duoqianAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameEqualTo(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameGreaterThan(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameLessThan(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameBetween(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameStartsWith(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameEndsWith(
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

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      nameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
          QAfterFilterCondition>
      nameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'name',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'name',
        value: '',
      ));
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity,
      QAfterFilterCondition> nameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'name',
        value: '',
      ));
    });
  }
}

extension PersonalDuoqianEntityQueryObject on QueryBuilder<
    PersonalDuoqianEntity, PersonalDuoqianEntity, QFilterCondition> {}

extension PersonalDuoqianEntityQueryLinks on QueryBuilder<PersonalDuoqianEntity,
    PersonalDuoqianEntity, QFilterCondition> {}

extension PersonalDuoqianEntityQuerySortBy
    on QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QSortBy> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByAddedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByCreatorAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'creatorAddress', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByCreatorAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'creatorAddress', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByDuoqianAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByDuoqianAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      sortByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }
}

extension PersonalDuoqianEntityQuerySortThenBy
    on QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QSortThenBy> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByAddedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByCreatorAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'creatorAddress', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByCreatorAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'creatorAddress', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByDuoqianAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByDuoqianAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QAfterSortBy>
      thenByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }
}

extension PersonalDuoqianEntityQueryWhereDistinct
    on QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QDistinct> {
  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QDistinct>
      distinctByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'addedAtMillis');
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QDistinct>
      distinctByCreatorAddress({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'creatorAddress',
          caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QDistinct>
      distinctByDuoqianAddress({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'duoqianAddress',
          caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<PersonalDuoqianEntity, PersonalDuoqianEntity, QDistinct>
      distinctByName({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'name', caseSensitive: caseSensitive);
    });
  }
}

extension PersonalDuoqianEntityQueryProperty on QueryBuilder<
    PersonalDuoqianEntity, PersonalDuoqianEntity, QQueryProperty> {
  QueryBuilder<PersonalDuoqianEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<PersonalDuoqianEntity, int, QQueryOperations>
      addedAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'addedAtMillis');
    });
  }

  QueryBuilder<PersonalDuoqianEntity, String, QQueryOperations>
      creatorAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'creatorAddress');
    });
  }

  QueryBuilder<PersonalDuoqianEntity, String, QQueryOperations>
      duoqianAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'duoqianAddress');
    });
  }

  QueryBuilder<PersonalDuoqianEntity, String, QQueryOperations> nameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'name');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetDuoqianInstitutionEntityCollection on Isar {
  IsarCollection<DuoqianInstitutionEntity> get duoqianInstitutionEntitys =>
      this.collection();
}

const DuoqianInstitutionEntitySchema = CollectionSchema(
  name: r'DuoqianInstitutionEntity',
  id: -2269869662941769306,
  properties: {
    r'addedAtMillis': PropertySchema(
      id: 0,
      name: r'addedAtMillis',
      type: IsarType.long,
    ),
    r'duoqianAddress': PropertySchema(
      id: 1,
      name: r'duoqianAddress',
      type: IsarType.string,
    ),
    r'name': PropertySchema(
      id: 2,
      name: r'name',
      type: IsarType.string,
    ),
    r'sfidId': PropertySchema(
      id: 3,
      name: r'sfidId',
      type: IsarType.string,
    )
  },
  estimateSize: _duoqianInstitutionEntityEstimateSize,
  serialize: _duoqianInstitutionEntitySerialize,
  deserialize: _duoqianInstitutionEntityDeserialize,
  deserializeProp: _duoqianInstitutionEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'duoqianAddress': IndexSchema(
      id: 3822211026917775041,
      name: r'duoqianAddress',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'duoqianAddress',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'addedAtMillis': IndexSchema(
      id: -1059979261930735929,
      name: r'addedAtMillis',
      unique: false,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'addedAtMillis',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _duoqianInstitutionEntityGetId,
  getLinks: _duoqianInstitutionEntityGetLinks,
  attach: _duoqianInstitutionEntityAttach,
  version: '3.1.0+1',
);

int _duoqianInstitutionEntityEstimateSize(
  DuoqianInstitutionEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.duoqianAddress.length * 3;
  bytesCount += 3 + object.name.length * 3;
  bytesCount += 3 + object.sfidId.length * 3;
  return bytesCount;
}

void _duoqianInstitutionEntitySerialize(
  DuoqianInstitutionEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeLong(offsets[0], object.addedAtMillis);
  writer.writeString(offsets[1], object.duoqianAddress);
  writer.writeString(offsets[2], object.name);
  writer.writeString(offsets[3], object.sfidId);
}

DuoqianInstitutionEntity _duoqianInstitutionEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = DuoqianInstitutionEntity();
  object.addedAtMillis = reader.readLong(offsets[0]);
  object.duoqianAddress = reader.readString(offsets[1]);
  object.id = id;
  object.name = reader.readString(offsets[2]);
  object.sfidId = reader.readString(offsets[3]);
  return object;
}

P _duoqianInstitutionEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readLong(offset)) as P;
    case 1:
      return (reader.readString(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    case 3:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _duoqianInstitutionEntityGetId(DuoqianInstitutionEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _duoqianInstitutionEntityGetLinks(
    DuoqianInstitutionEntity object) {
  return [];
}

void _duoqianInstitutionEntityAttach(
    IsarCollection<dynamic> col, Id id, DuoqianInstitutionEntity object) {
  object.id = id;
}

extension DuoqianInstitutionEntityByIndex
    on IsarCollection<DuoqianInstitutionEntity> {
  Future<DuoqianInstitutionEntity?> getByDuoqianAddress(String duoqianAddress) {
    return getByIndex(r'duoqianAddress', [duoqianAddress]);
  }

  DuoqianInstitutionEntity? getByDuoqianAddressSync(String duoqianAddress) {
    return getByIndexSync(r'duoqianAddress', [duoqianAddress]);
  }

  Future<bool> deleteByDuoqianAddress(String duoqianAddress) {
    return deleteByIndex(r'duoqianAddress', [duoqianAddress]);
  }

  bool deleteByDuoqianAddressSync(String duoqianAddress) {
    return deleteByIndexSync(r'duoqianAddress', [duoqianAddress]);
  }

  Future<List<DuoqianInstitutionEntity?>> getAllByDuoqianAddress(
      List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return getAllByIndex(r'duoqianAddress', values);
  }

  List<DuoqianInstitutionEntity?> getAllByDuoqianAddressSync(
      List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'duoqianAddress', values);
  }

  Future<int> deleteAllByDuoqianAddress(List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'duoqianAddress', values);
  }

  int deleteAllByDuoqianAddressSync(List<String> duoqianAddressValues) {
    final values = duoqianAddressValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'duoqianAddress', values);
  }

  Future<Id> putByDuoqianAddress(DuoqianInstitutionEntity object) {
    return putByIndex(r'duoqianAddress', object);
  }

  Id putByDuoqianAddressSync(DuoqianInstitutionEntity object,
      {bool saveLinks = true}) {
    return putByIndexSync(r'duoqianAddress', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByDuoqianAddress(
      List<DuoqianInstitutionEntity> objects) {
    return putAllByIndex(r'duoqianAddress', objects);
  }

  List<Id> putAllByDuoqianAddressSync(List<DuoqianInstitutionEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'duoqianAddress', objects, saveLinks: saveLinks);
  }
}

extension DuoqianInstitutionEntityQueryWhereSort on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QWhere> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterWhere>
      anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterWhere>
      anyAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'addedAtMillis'),
      );
    });
  }
}

extension DuoqianInstitutionEntityQueryWhere on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QWhereClause> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> idNotEqualTo(Id id) {
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> idBetween(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> duoqianAddressEqualTo(String duoqianAddress) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'duoqianAddress',
        value: [duoqianAddress],
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> duoqianAddressNotEqualTo(String duoqianAddress) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [],
              upper: [duoqianAddress],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [duoqianAddress],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [duoqianAddress],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'duoqianAddress',
              lower: [],
              upper: [duoqianAddress],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> addedAtMillisEqualTo(int addedAtMillis) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'addedAtMillis',
        value: [addedAtMillis],
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> addedAtMillisNotEqualTo(int addedAtMillis) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [],
              upper: [addedAtMillis],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [addedAtMillis],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [addedAtMillis],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'addedAtMillis',
              lower: [],
              upper: [addedAtMillis],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> addedAtMillisGreaterThan(
    int addedAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [addedAtMillis],
        includeLower: include,
        upper: [],
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> addedAtMillisLessThan(
    int addedAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [],
        upper: [addedAtMillis],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterWhereClause> addedAtMillisBetween(
    int lowerAddedAtMillis,
    int upperAddedAtMillis, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'addedAtMillis',
        lower: [lowerAddedAtMillis],
        includeLower: includeLower,
        upper: [upperAddedAtMillis],
        includeUpper: includeUpper,
      ));
    });
  }
}

extension DuoqianInstitutionEntityQueryFilter on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QFilterCondition> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> addedAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> addedAtMillisGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> addedAtMillisLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'addedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> addedAtMillisBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'addedAtMillis',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'duoqianAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      duoqianAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'duoqianAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      duoqianAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'duoqianAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'duoqianAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> duoqianAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'duoqianAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameEqualTo(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameGreaterThan(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameLessThan(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameBetween(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameStartsWith(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameEndsWith(
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

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      nameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'name',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      nameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'name',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'name',
        value: '',
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> nameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'name',
        value: '',
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'sfidId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      sfidIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'sfidId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
          QAfterFilterCondition>
      sfidIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'sfidId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'sfidId',
        value: '',
      ));
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity,
      QAfterFilterCondition> sfidIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'sfidId',
        value: '',
      ));
    });
  }
}

extension DuoqianInstitutionEntityQueryObject on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QFilterCondition> {}

extension DuoqianInstitutionEntityQueryLinks on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QFilterCondition> {}

extension DuoqianInstitutionEntityQuerySortBy on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QSortBy> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByAddedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByDuoqianAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByDuoqianAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortBySfidId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sfidId', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      sortBySfidIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sfidId', Sort.desc);
    });
  }
}

extension DuoqianInstitutionEntityQuerySortThenBy on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QSortThenBy> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByAddedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'addedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByDuoqianAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByDuoqianAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'duoqianAddress', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenByNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'name', Sort.desc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenBySfidId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sfidId', Sort.asc);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QAfterSortBy>
      thenBySfidIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sfidId', Sort.desc);
    });
  }
}

extension DuoqianInstitutionEntityQueryWhereDistinct on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QDistinct> {
  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QDistinct>
      distinctByAddedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'addedAtMillis');
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QDistinct>
      distinctByDuoqianAddress({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'duoqianAddress',
          caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QDistinct>
      distinctByName({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'name', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, DuoqianInstitutionEntity, QDistinct>
      distinctBySfidId({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sfidId', caseSensitive: caseSensitive);
    });
  }
}

extension DuoqianInstitutionEntityQueryProperty on QueryBuilder<
    DuoqianInstitutionEntity, DuoqianInstitutionEntity, QQueryProperty> {
  QueryBuilder<DuoqianInstitutionEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, int, QQueryOperations>
      addedAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'addedAtMillis');
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, String, QQueryOperations>
      duoqianAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'duoqianAddress');
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, String, QQueryOperations>
      nameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'name');
    });
  }

  QueryBuilder<DuoqianInstitutionEntity, String, QQueryOperations>
      sfidIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sfidId');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetLocalTxEntityCollection on Isar {
  IsarCollection<LocalTxEntity> get localTxEntitys => this.collection();
}

const LocalTxEntitySchema = CollectionSchema(
  name: r'LocalTxEntity',
  id: 3324518130997293643,
  properties: {
    r'amountYuan': PropertySchema(
      id: 0,
      name: r'amountYuan',
      type: IsarType.double,
    ),
    r'bankShenfenId': PropertySchema(
      id: 1,
      name: r'bankShenfenId',
      type: IsarType.string,
    ),
    r'blockNumber': PropertySchema(
      id: 2,
      name: r'blockNumber',
      type: IsarType.long,
    ),
    r'confirmedAtMillis': PropertySchema(
      id: 3,
      name: r'confirmedAtMillis',
      type: IsarType.long,
    ),
    r'createdAtMillis': PropertySchema(
      id: 4,
      name: r'createdAtMillis',
      type: IsarType.long,
    ),
    r'direction': PropertySchema(
      id: 5,
      name: r'direction',
      type: IsarType.string,
    ),
    r'feeYuan': PropertySchema(
      id: 6,
      name: r'feeYuan',
      type: IsarType.double,
    ),
    r'fromAddress': PropertySchema(
      id: 7,
      name: r'fromAddress',
      type: IsarType.string,
    ),
    r'status': PropertySchema(
      id: 8,
      name: r'status',
      type: IsarType.string,
    ),
    r'toAddress': PropertySchema(
      id: 9,
      name: r'toAddress',
      type: IsarType.string,
    ),
    r'txHash': PropertySchema(
      id: 10,
      name: r'txHash',
      type: IsarType.string,
    ),
    r'txId': PropertySchema(
      id: 11,
      name: r'txId',
      type: IsarType.string,
    ),
    r'txType': PropertySchema(
      id: 12,
      name: r'txType',
      type: IsarType.string,
    ),
    r'usedNonce': PropertySchema(
      id: 13,
      name: r'usedNonce',
      type: IsarType.long,
    ),
    r'walletAddress': PropertySchema(
      id: 14,
      name: r'walletAddress',
      type: IsarType.string,
    )
  },
  estimateSize: _localTxEntityEstimateSize,
  serialize: _localTxEntitySerialize,
  deserialize: _localTxEntityDeserialize,
  deserializeProp: _localTxEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'txId': IndexSchema(
      id: 1771378982912115290,
      name: r'txId',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'txId',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'walletAddress': IndexSchema(
      id: -6656497977715402550,
      name: r'walletAddress',
      unique: false,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'walletAddress',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'createdAtMillis': IndexSchema(
      id: -2739706252225730577,
      name: r'createdAtMillis',
      unique: false,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'createdAtMillis',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _localTxEntityGetId,
  getLinks: _localTxEntityGetLinks,
  attach: _localTxEntityAttach,
  version: '3.1.0+1',
);

int _localTxEntityEstimateSize(
  LocalTxEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  {
    final value = object.bankShenfenId;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.direction.length * 3;
  {
    final value = object.fromAddress;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.status.length * 3;
  {
    final value = object.toAddress;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  {
    final value = object.txHash;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.txId.length * 3;
  bytesCount += 3 + object.txType.length * 3;
  bytesCount += 3 + object.walletAddress.length * 3;
  return bytesCount;
}

void _localTxEntitySerialize(
  LocalTxEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeDouble(offsets[0], object.amountYuan);
  writer.writeString(offsets[1], object.bankShenfenId);
  writer.writeLong(offsets[2], object.blockNumber);
  writer.writeLong(offsets[3], object.confirmedAtMillis);
  writer.writeLong(offsets[4], object.createdAtMillis);
  writer.writeString(offsets[5], object.direction);
  writer.writeDouble(offsets[6], object.feeYuan);
  writer.writeString(offsets[7], object.fromAddress);
  writer.writeString(offsets[8], object.status);
  writer.writeString(offsets[9], object.toAddress);
  writer.writeString(offsets[10], object.txHash);
  writer.writeString(offsets[11], object.txId);
  writer.writeString(offsets[12], object.txType);
  writer.writeLong(offsets[13], object.usedNonce);
  writer.writeString(offsets[14], object.walletAddress);
}

LocalTxEntity _localTxEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = LocalTxEntity();
  object.amountYuan = reader.readDouble(offsets[0]);
  object.bankShenfenId = reader.readStringOrNull(offsets[1]);
  object.blockNumber = reader.readLongOrNull(offsets[2]);
  object.confirmedAtMillis = reader.readLongOrNull(offsets[3]);
  object.createdAtMillis = reader.readLong(offsets[4]);
  object.direction = reader.readString(offsets[5]);
  object.feeYuan = reader.readDoubleOrNull(offsets[6]);
  object.fromAddress = reader.readStringOrNull(offsets[7]);
  object.id = id;
  object.status = reader.readString(offsets[8]);
  object.toAddress = reader.readStringOrNull(offsets[9]);
  object.txHash = reader.readStringOrNull(offsets[10]);
  object.txId = reader.readString(offsets[11]);
  object.txType = reader.readString(offsets[12]);
  object.usedNonce = reader.readLongOrNull(offsets[13]);
  object.walletAddress = reader.readString(offsets[14]);
  return object;
}

P _localTxEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readDouble(offset)) as P;
    case 1:
      return (reader.readStringOrNull(offset)) as P;
    case 2:
      return (reader.readLongOrNull(offset)) as P;
    case 3:
      return (reader.readLongOrNull(offset)) as P;
    case 4:
      return (reader.readLong(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    case 6:
      return (reader.readDoubleOrNull(offset)) as P;
    case 7:
      return (reader.readStringOrNull(offset)) as P;
    case 8:
      return (reader.readString(offset)) as P;
    case 9:
      return (reader.readStringOrNull(offset)) as P;
    case 10:
      return (reader.readStringOrNull(offset)) as P;
    case 11:
      return (reader.readString(offset)) as P;
    case 12:
      return (reader.readString(offset)) as P;
    case 13:
      return (reader.readLongOrNull(offset)) as P;
    case 14:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _localTxEntityGetId(LocalTxEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _localTxEntityGetLinks(LocalTxEntity object) {
  return [];
}

void _localTxEntityAttach(
    IsarCollection<dynamic> col, Id id, LocalTxEntity object) {
  object.id = id;
}

extension LocalTxEntityByIndex on IsarCollection<LocalTxEntity> {
  Future<LocalTxEntity?> getByTxId(String txId) {
    return getByIndex(r'txId', [txId]);
  }

  LocalTxEntity? getByTxIdSync(String txId) {
    return getByIndexSync(r'txId', [txId]);
  }

  Future<bool> deleteByTxId(String txId) {
    return deleteByIndex(r'txId', [txId]);
  }

  bool deleteByTxIdSync(String txId) {
    return deleteByIndexSync(r'txId', [txId]);
  }

  Future<List<LocalTxEntity?>> getAllByTxId(List<String> txIdValues) {
    final values = txIdValues.map((e) => [e]).toList();
    return getAllByIndex(r'txId', values);
  }

  List<LocalTxEntity?> getAllByTxIdSync(List<String> txIdValues) {
    final values = txIdValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'txId', values);
  }

  Future<int> deleteAllByTxId(List<String> txIdValues) {
    final values = txIdValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'txId', values);
  }

  int deleteAllByTxIdSync(List<String> txIdValues) {
    final values = txIdValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'txId', values);
  }

  Future<Id> putByTxId(LocalTxEntity object) {
    return putByIndex(r'txId', object);
  }

  Id putByTxIdSync(LocalTxEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'txId', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByTxId(List<LocalTxEntity> objects) {
    return putAllByIndex(r'txId', objects);
  }

  List<Id> putAllByTxIdSync(List<LocalTxEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'txId', objects, saveLinks: saveLinks);
  }
}

extension LocalTxEntityQueryWhereSort
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QWhere> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhere> anyCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'createdAtMillis'),
      );
    });
  }
}

extension LocalTxEntityQueryWhere
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QWhereClause> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> idEqualTo(
      Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> idNotEqualTo(
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> idGreaterThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> idLessThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> idBetween(
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> txIdEqualTo(
      String txId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'txId',
        value: [txId],
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause> txIdNotEqualTo(
      String txId) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txId',
              lower: [],
              upper: [txId],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txId',
              lower: [txId],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txId',
              lower: [txId],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txId',
              lower: [],
              upper: [txId],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      walletAddressEqualTo(String walletAddress) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'walletAddress',
        value: [walletAddress],
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      walletAddressNotEqualTo(String walletAddress) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletAddress',
              lower: [],
              upper: [walletAddress],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletAddress',
              lower: [walletAddress],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletAddress',
              lower: [walletAddress],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'walletAddress',
              lower: [],
              upper: [walletAddress],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      createdAtMillisEqualTo(int createdAtMillis) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'createdAtMillis',
        value: [createdAtMillis],
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      createdAtMillisNotEqualTo(int createdAtMillis) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'createdAtMillis',
              lower: [],
              upper: [createdAtMillis],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'createdAtMillis',
              lower: [createdAtMillis],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'createdAtMillis',
              lower: [createdAtMillis],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'createdAtMillis',
              lower: [],
              upper: [createdAtMillis],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      createdAtMillisGreaterThan(
    int createdAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'createdAtMillis',
        lower: [createdAtMillis],
        includeLower: include,
        upper: [],
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      createdAtMillisLessThan(
    int createdAtMillis, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'createdAtMillis',
        lower: [],
        upper: [createdAtMillis],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterWhereClause>
      createdAtMillisBetween(
    int lowerCreatedAtMillis,
    int upperCreatedAtMillis, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'createdAtMillis',
        lower: [lowerCreatedAtMillis],
        includeLower: includeLower,
        upper: [upperCreatedAtMillis],
        includeUpper: includeUpper,
      ));
    });
  }
}

extension LocalTxEntityQueryFilter
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QFilterCondition> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      amountYuanEqualTo(
    double value, {
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'amountYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      amountYuanGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'amountYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      amountYuanLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'amountYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      amountYuanBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'amountYuan',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'bankShenfenId',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'bankShenfenId',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'bankShenfenId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'bankShenfenId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'bankShenfenId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'bankShenfenId',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      bankShenfenIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'bankShenfenId',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'blockNumber',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'blockNumber',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'blockNumber',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberGreaterThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'blockNumber',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberLessThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'blockNumber',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      blockNumberBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'blockNumber',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'confirmedAtMillis',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'confirmedAtMillis',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'confirmedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisGreaterThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'confirmedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisLessThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'confirmedAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      confirmedAtMillisBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'confirmedAtMillis',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      createdAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'direction',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'direction',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'direction',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'direction',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      directionIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'direction',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'feeYuan',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'feeYuan',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanEqualTo(
    double? value, {
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'feeYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanGreaterThan(
    double? value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'feeYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanLessThan(
    double? value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'feeYuan',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      feeYuanBetween(
    double? lower,
    double? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'feeYuan',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'fromAddress',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'fromAddress',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'fromAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'fromAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'fromAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      fromAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'fromAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> idEqualTo(
      Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> idLessThan(
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> idBetween(
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

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'status',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'status',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'status',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      statusIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'status',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'toAddress',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'toAddress',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'toAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'toAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'toAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      toAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'toAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'txHash',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'txHash',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'txHash',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'txHash',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txHash',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txHashIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'txHash',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> txIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> txIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'txId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'txId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition> txIdMatches(
      String pattern,
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'txId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txId',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'txId',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'txType',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'txType',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'txType',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txType',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      txTypeIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'txType',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'usedNonce',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'usedNonce',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'usedNonce',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceGreaterThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'usedNonce',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceLessThan(
    int? value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'usedNonce',
        value: value,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      usedNonceBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'usedNonce',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'walletAddress',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'walletAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'walletAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterFilterCondition>
      walletAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'walletAddress',
        value: '',
      ));
    });
  }
}

extension LocalTxEntityQueryObject
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QFilterCondition> {}

extension LocalTxEntityQueryLinks
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QFilterCondition> {}

extension LocalTxEntityQuerySortBy
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QSortBy> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByAmountYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amountYuan', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByAmountYuanDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amountYuan', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByBankShenfenId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'bankShenfenId', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByBankShenfenIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'bankShenfenId', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByBlockNumber() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'blockNumber', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByBlockNumberDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'blockNumber', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByConfirmedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'confirmedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByConfirmedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'confirmedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByDirection() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'direction', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByDirectionDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'direction', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByFeeYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'feeYuan', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByFeeYuanDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'feeYuan', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByFromAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByFromAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByStatus() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByStatusDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByToAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByToAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxHash() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxHashDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txId', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txId', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxType() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txType', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByTxTypeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txType', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> sortByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByUsedNonceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByWalletAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      sortByWalletAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletAddress', Sort.desc);
    });
  }
}

extension LocalTxEntityQuerySortThenBy
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QSortThenBy> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByAmountYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amountYuan', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByAmountYuanDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amountYuan', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByBankShenfenId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'bankShenfenId', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByBankShenfenIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'bankShenfenId', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByBlockNumber() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'blockNumber', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByBlockNumberDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'blockNumber', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByConfirmedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'confirmedAtMillis', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByConfirmedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'confirmedAtMillis', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByDirection() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'direction', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByDirectionDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'direction', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByFeeYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'feeYuan', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByFeeYuanDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'feeYuan', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByFromAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByFromAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByStatus() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByStatusDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByToAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByToAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxHash() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxHashDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txId', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txId', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxType() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txType', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByTxTypeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txType', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy> thenByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByUsedNonceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.desc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByWalletAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletAddress', Sort.asc);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QAfterSortBy>
      thenByWalletAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletAddress', Sort.desc);
    });
  }
}

extension LocalTxEntityQueryWhereDistinct
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> {
  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByAmountYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'amountYuan');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByBankShenfenId(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'bankShenfenId',
          caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct>
      distinctByBlockNumber() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'blockNumber');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct>
      distinctByConfirmedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'confirmedAtMillis');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct>
      distinctByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'createdAtMillis');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByDirection(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'direction', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByFeeYuan() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'feeYuan');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByFromAddress(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'fromAddress', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByStatus(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'status', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByToAddress(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'toAddress', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByTxHash(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'txHash', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByTxId(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'txId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByTxType(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'txType', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'usedNonce');
    });
  }

  QueryBuilder<LocalTxEntity, LocalTxEntity, QDistinct> distinctByWalletAddress(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletAddress',
          caseSensitive: caseSensitive);
    });
  }
}

extension LocalTxEntityQueryProperty
    on QueryBuilder<LocalTxEntity, LocalTxEntity, QQueryProperty> {
  QueryBuilder<LocalTxEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<LocalTxEntity, double, QQueryOperations> amountYuanProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'amountYuan');
    });
  }

  QueryBuilder<LocalTxEntity, String?, QQueryOperations>
      bankShenfenIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'bankShenfenId');
    });
  }

  QueryBuilder<LocalTxEntity, int?, QQueryOperations> blockNumberProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'blockNumber');
    });
  }

  QueryBuilder<LocalTxEntity, int?, QQueryOperations>
      confirmedAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'confirmedAtMillis');
    });
  }

  QueryBuilder<LocalTxEntity, int, QQueryOperations> createdAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'createdAtMillis');
    });
  }

  QueryBuilder<LocalTxEntity, String, QQueryOperations> directionProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'direction');
    });
  }

  QueryBuilder<LocalTxEntity, double?, QQueryOperations> feeYuanProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'feeYuan');
    });
  }

  QueryBuilder<LocalTxEntity, String?, QQueryOperations> fromAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'fromAddress');
    });
  }

  QueryBuilder<LocalTxEntity, String, QQueryOperations> statusProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'status');
    });
  }

  QueryBuilder<LocalTxEntity, String?, QQueryOperations> toAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toAddress');
    });
  }

  QueryBuilder<LocalTxEntity, String?, QQueryOperations> txHashProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'txHash');
    });
  }

  QueryBuilder<LocalTxEntity, String, QQueryOperations> txIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'txId');
    });
  }

  QueryBuilder<LocalTxEntity, String, QQueryOperations> txTypeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'txType');
    });
  }

  QueryBuilder<LocalTxEntity, int?, QQueryOperations> usedNonceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'usedNonce');
    });
  }

  QueryBuilder<LocalTxEntity, String, QQueryOperations>
      walletAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletAddress');
    });
  }
}
