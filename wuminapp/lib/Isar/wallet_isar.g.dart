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
    r'source': PropertySchema(
      id: 5,
      name: r'source',
      type: IsarType.string,
    ),
    r'ss58': PropertySchema(
      id: 6,
      name: r'ss58',
      type: IsarType.long,
    ),
    r'walletIcon': PropertySchema(
      id: 7,
      name: r'walletIcon',
      type: IsarType.string,
    ),
    r'walletIndex': PropertySchema(
      id: 8,
      name: r'walletIndex',
      type: IsarType.long,
    ),
    r'walletName': PropertySchema(
      id: 9,
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
  writer.writeString(offsets[5], object.source);
  writer.writeLong(offsets[6], object.ss58);
  writer.writeString(offsets[7], object.walletIcon);
  writer.writeLong(offsets[8], object.walletIndex);
  writer.writeString(offsets[9], object.walletName);
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
  object.source = reader.readString(offsets[5]);
  object.ss58 = reader.readLong(offsets[6]);
  object.walletIcon = reader.readString(offsets[7]);
  object.walletIndex = reader.readLong(offsets[8]);
  object.walletName = reader.readString(offsets[9]);
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

extension GetTxRecordEntityCollection on Isar {
  IsarCollection<TxRecordEntity> get txRecordEntitys => this.collection();
}

const TxRecordEntitySchema = CollectionSchema(
  name: r'TxRecordEntity',
  id: -8720012383598844330,
  properties: {
    r'amount': PropertySchema(
      id: 0,
      name: r'amount',
      type: IsarType.double,
    ),
    r'createdAtMillis': PropertySchema(
      id: 1,
      name: r'createdAtMillis',
      type: IsarType.long,
    ),
    r'failureReason': PropertySchema(
      id: 2,
      name: r'failureReason',
      type: IsarType.string,
    ),
    r'fromAddress': PropertySchema(
      id: 3,
      name: r'fromAddress',
      type: IsarType.string,
    ),
    r'status': PropertySchema(
      id: 4,
      name: r'status',
      type: IsarType.string,
    ),
    r'symbol': PropertySchema(
      id: 5,
      name: r'symbol',
      type: IsarType.string,
    ),
    r'toAddress': PropertySchema(
      id: 6,
      name: r'toAddress',
      type: IsarType.string,
    ),
    r'txHash': PropertySchema(
      id: 7,
      name: r'txHash',
      type: IsarType.string,
    ),
    r'usedNonce': PropertySchema(
      id: 8,
      name: r'usedNonce',
      type: IsarType.long,
    )
  },
  estimateSize: _txRecordEntityEstimateSize,
  serialize: _txRecordEntitySerialize,
  deserialize: _txRecordEntityDeserialize,
  deserializeProp: _txRecordEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'txHash': IndexSchema(
      id: -7817790489977483783,
      name: r'txHash',
      unique: true,
      replace: true,
      properties: [
        IndexPropertySchema(
          name: r'txHash',
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
  getId: _txRecordEntityGetId,
  getLinks: _txRecordEntityGetLinks,
  attach: _txRecordEntityAttach,
  version: '3.1.0+1',
);

int _txRecordEntityEstimateSize(
  TxRecordEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  {
    final value = object.failureReason;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.fromAddress.length * 3;
  bytesCount += 3 + object.status.length * 3;
  bytesCount += 3 + object.symbol.length * 3;
  bytesCount += 3 + object.toAddress.length * 3;
  bytesCount += 3 + object.txHash.length * 3;
  return bytesCount;
}

void _txRecordEntitySerialize(
  TxRecordEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeDouble(offsets[0], object.amount);
  writer.writeLong(offsets[1], object.createdAtMillis);
  writer.writeString(offsets[2], object.failureReason);
  writer.writeString(offsets[3], object.fromAddress);
  writer.writeString(offsets[4], object.status);
  writer.writeString(offsets[5], object.symbol);
  writer.writeString(offsets[6], object.toAddress);
  writer.writeString(offsets[7], object.txHash);
  writer.writeLong(offsets[8], object.usedNonce);
}

TxRecordEntity _txRecordEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = TxRecordEntity();
  object.amount = reader.readDouble(offsets[0]);
  object.createdAtMillis = reader.readLong(offsets[1]);
  object.failureReason = reader.readStringOrNull(offsets[2]);
  object.fromAddress = reader.readString(offsets[3]);
  object.id = id;
  object.status = reader.readString(offsets[4]);
  object.symbol = reader.readString(offsets[5]);
  object.toAddress = reader.readString(offsets[6]);
  object.txHash = reader.readString(offsets[7]);
  object.usedNonce = reader.readLongOrNull(offsets[8]);
  return object;
}

P _txRecordEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readDouble(offset)) as P;
    case 1:
      return (reader.readLong(offset)) as P;
    case 2:
      return (reader.readStringOrNull(offset)) as P;
    case 3:
      return (reader.readString(offset)) as P;
    case 4:
      return (reader.readString(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    case 6:
      return (reader.readString(offset)) as P;
    case 7:
      return (reader.readString(offset)) as P;
    case 8:
      return (reader.readLongOrNull(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _txRecordEntityGetId(TxRecordEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _txRecordEntityGetLinks(TxRecordEntity object) {
  return [];
}

void _txRecordEntityAttach(
    IsarCollection<dynamic> col, Id id, TxRecordEntity object) {
  object.id = id;
}

extension TxRecordEntityByIndex on IsarCollection<TxRecordEntity> {
  Future<TxRecordEntity?> getByTxHash(String txHash) {
    return getByIndex(r'txHash', [txHash]);
  }

  TxRecordEntity? getByTxHashSync(String txHash) {
    return getByIndexSync(r'txHash', [txHash]);
  }

  Future<bool> deleteByTxHash(String txHash) {
    return deleteByIndex(r'txHash', [txHash]);
  }

  bool deleteByTxHashSync(String txHash) {
    return deleteByIndexSync(r'txHash', [txHash]);
  }

  Future<List<TxRecordEntity?>> getAllByTxHash(List<String> txHashValues) {
    final values = txHashValues.map((e) => [e]).toList();
    return getAllByIndex(r'txHash', values);
  }

  List<TxRecordEntity?> getAllByTxHashSync(List<String> txHashValues) {
    final values = txHashValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'txHash', values);
  }

  Future<int> deleteAllByTxHash(List<String> txHashValues) {
    final values = txHashValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'txHash', values);
  }

  int deleteAllByTxHashSync(List<String> txHashValues) {
    final values = txHashValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'txHash', values);
  }

  Future<Id> putByTxHash(TxRecordEntity object) {
    return putByIndex(r'txHash', object);
  }

  Id putByTxHashSync(TxRecordEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'txHash', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByTxHash(List<TxRecordEntity> objects) {
    return putAllByIndex(r'txHash', objects);
  }

  List<Id> putAllByTxHashSync(List<TxRecordEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'txHash', objects, saveLinks: saveLinks);
  }
}

extension TxRecordEntityQueryWhereSort
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QWhere> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhere>
      anyCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'createdAtMillis'),
      );
    });
  }
}

extension TxRecordEntityQueryWhere
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QWhereClause> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> idEqualTo(
      Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> idNotEqualTo(
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> idGreaterThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> idLessThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> idBetween(
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause> txHashEqualTo(
      String txHash) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'txHash',
        value: [txHash],
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
      txHashNotEqualTo(String txHash) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txHash',
              lower: [],
              upper: [txHash],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txHash',
              lower: [txHash],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txHash',
              lower: [txHash],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'txHash',
              lower: [],
              upper: [txHash],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
      createdAtMillisEqualTo(int createdAtMillis) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'createdAtMillis',
        value: [createdAtMillis],
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterWhereClause>
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

extension TxRecordEntityQueryFilter
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QFilterCondition> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      amountEqualTo(
    double value, {
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'amount',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      amountGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'amount',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      amountLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'amount',
        value: value,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      amountBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'amount',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        epsilon: epsilon,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      createdAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'failureReason',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'failureReason',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonEqualTo(
    String? value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'failureReason',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'failureReason',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'failureReason',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'failureReason',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      failureReasonIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'failureReason',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressEqualTo(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressGreaterThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressLessThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressBetween(
    String lower,
    String upper, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'fromAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'fromAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'fromAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      fromAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'fromAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition> idEqualTo(
      Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition> idBetween(
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      statusContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'status',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      statusMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'status',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      statusIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'status',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      statusIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'status',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'symbol',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'symbol',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'symbol',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'symbol',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      symbolIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'symbol',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressEqualTo(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressGreaterThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressLessThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressBetween(
    String lower,
    String upper, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'toAddress',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'toAddress',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'toAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      toAddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'toAddress',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashEqualTo(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashGreaterThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashLessThan(
    String value, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashBetween(
    String lower,
    String upper, {
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'txHash',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'txHash',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'txHash',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      txHashIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'txHash',
        value: '',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      usedNonceIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNull(
        property: r'usedNonce',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      usedNonceIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(const FilterCondition.isNotNull(
        property: r'usedNonce',
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
      usedNonceEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'usedNonce',
        value: value,
      ));
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterFilterCondition>
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
}

extension TxRecordEntityQueryObject
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QFilterCondition> {}

extension TxRecordEntityQueryLinks
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QFilterCondition> {}

extension TxRecordEntityQuerySortBy
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QSortBy> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortByAmount() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amount', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByAmountDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amount', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByFailureReason() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'failureReason', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByFailureReasonDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'failureReason', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByFromAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByFromAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortByStatus() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByStatusDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortBySymbol() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'symbol', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortBySymbolDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'symbol', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortByToAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByToAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortByTxHash() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByTxHashDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> sortByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      sortByUsedNonceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.desc);
    });
  }
}

extension TxRecordEntityQuerySortThenBy
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QSortThenBy> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByAmount() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amount', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByAmountDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'amount', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByFailureReason() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'failureReason', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByFailureReasonDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'failureReason', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByFromAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByFromAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fromAddress', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByStatus() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByStatusDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'status', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenBySymbol() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'symbol', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenBySymbolDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'symbol', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByToAddress() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByToAddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toAddress', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByTxHash() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByTxHashDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'txHash', Sort.desc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy> thenByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.asc);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QAfterSortBy>
      thenByUsedNonceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'usedNonce', Sort.desc);
    });
  }
}

extension TxRecordEntityQueryWhereDistinct
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> {
  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctByAmount() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'amount');
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct>
      distinctByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'createdAtMillis');
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct>
      distinctByFailureReason({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'failureReason',
          caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctByFromAddress(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'fromAddress', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctByStatus(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'status', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctBySymbol(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'symbol', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctByToAddress(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'toAddress', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct> distinctByTxHash(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'txHash', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<TxRecordEntity, TxRecordEntity, QDistinct>
      distinctByUsedNonce() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'usedNonce');
    });
  }
}

extension TxRecordEntityQueryProperty
    on QueryBuilder<TxRecordEntity, TxRecordEntity, QQueryProperty> {
  QueryBuilder<TxRecordEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<TxRecordEntity, double, QQueryOperations> amountProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'amount');
    });
  }

  QueryBuilder<TxRecordEntity, int, QQueryOperations>
      createdAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'createdAtMillis');
    });
  }

  QueryBuilder<TxRecordEntity, String?, QQueryOperations>
      failureReasonProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'failureReason');
    });
  }

  QueryBuilder<TxRecordEntity, String, QQueryOperations> fromAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'fromAddress');
    });
  }

  QueryBuilder<TxRecordEntity, String, QQueryOperations> statusProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'status');
    });
  }

  QueryBuilder<TxRecordEntity, String, QQueryOperations> symbolProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'symbol');
    });
  }

  QueryBuilder<TxRecordEntity, String, QQueryOperations> toAddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toAddress');
    });
  }

  QueryBuilder<TxRecordEntity, String, QQueryOperations> txHashProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'txHash');
    });
  }

  QueryBuilder<TxRecordEntity, int?, QQueryOperations> usedNonceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'usedNonce');
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
