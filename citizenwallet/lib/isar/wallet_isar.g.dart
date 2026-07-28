// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_isar.dart';

// **************************************************************************
// IsarCollectionGenerator
// **************************************************************************

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetWalletEntityCollection on Isar {
  IsarCollection<WalletEntity> get walletEntitys => this.collection();
}

const WalletEntitySchema = CollectionSchema(
  name: r'WalletEntity',
  id: 495311719639707741,
  properties: {
    r'createdAtMillis': PropertySchema(
      id: 0,
      name: r'createdAtMillis',
      type: IsarType.long,
    ),
    r'masterId': PropertySchema(
      id: 1,
      name: r'masterId',
      type: IsarType.string,
    ),
    r'sortOrder': PropertySchema(
      id: 2,
      name: r'sortOrder',
      type: IsarType.long,
    ),
    r'source': PropertySchema(
      id: 3,
      name: r'source',
      type: IsarType.string,
    ),
    r'walletIndex': PropertySchema(
      id: 4,
      name: r'walletIndex',
      type: IsarType.long,
    ),
    r'walletName': PropertySchema(
      id: 5,
      name: r'walletName',
      type: IsarType.string,
    )
  },
  estimateSize: _walletEntityEstimateSize,
  serialize: _walletEntitySerialize,
  deserialize: _walletEntityDeserialize,
  deserializeProp: _walletEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'walletIndex': IndexSchema(
      id: 3929031194099616871,
      name: r'walletIndex',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'walletIndex',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    ),
    r'masterId': IndexSchema(
      id: 8318582791188363777,
      name: r'masterId',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'masterId',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _walletEntityGetId,
  getLinks: _walletEntityGetLinks,
  attach: _walletEntityAttach,
  version: '3.1.0+1',
);

int _walletEntityEstimateSize(
  WalletEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.masterId.length * 3;
  bytesCount += 3 + object.source.length * 3;
  bytesCount += 3 + object.walletName.length * 3;
  return bytesCount;
}

void _walletEntitySerialize(
  WalletEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeLong(offsets[0], object.createdAtMillis);
  writer.writeString(offsets[1], object.masterId);
  writer.writeLong(offsets[2], object.sortOrder);
  writer.writeString(offsets[3], object.source);
  writer.writeLong(offsets[4], object.walletIndex);
  writer.writeString(offsets[5], object.walletName);
}

WalletEntity _walletEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = WalletEntity();
  object.createdAtMillis = reader.readLong(offsets[0]);
  object.id = id;
  object.masterId = reader.readString(offsets[1]);
  object.sortOrder = reader.readLong(offsets[2]);
  object.source = reader.readString(offsets[3]);
  object.walletIndex = reader.readLong(offsets[4]);
  object.walletName = reader.readString(offsets[5]);
  return object;
}

P _walletEntityDeserializeProp<P>(
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
      return (reader.readLong(offset)) as P;
    case 3:
      return (reader.readString(offset)) as P;
    case 4:
      return (reader.readLong(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _walletEntityGetId(WalletEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _walletEntityGetLinks(WalletEntity object) {
  return [];
}

void _walletEntityAttach(
    IsarCollection<dynamic> col, Id id, WalletEntity object) {
  object.id = id;
}

extension WalletEntityByIndex on IsarCollection<WalletEntity> {
  Future<WalletEntity?> getByWalletIndex(int walletIndex) {
    return getByIndex(r'walletIndex', [walletIndex]);
  }

  WalletEntity? getByWalletIndexSync(int walletIndex) {
    return getByIndexSync(r'walletIndex', [walletIndex]);
  }

  Future<bool> deleteByWalletIndex(int walletIndex) {
    return deleteByIndex(r'walletIndex', [walletIndex]);
  }

  bool deleteByWalletIndexSync(int walletIndex) {
    return deleteByIndexSync(r'walletIndex', [walletIndex]);
  }

  Future<List<WalletEntity?>> getAllByWalletIndex(List<int> walletIndexValues) {
    final values = walletIndexValues.map((e) => [e]).toList();
    return getAllByIndex(r'walletIndex', values);
  }

  List<WalletEntity?> getAllByWalletIndexSync(List<int> walletIndexValues) {
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

  Future<Id> putByWalletIndex(WalletEntity object) {
    return putByIndex(r'walletIndex', object);
  }

  Id putByWalletIndexSync(WalletEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'walletIndex', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByWalletIndex(List<WalletEntity> objects) {
    return putAllByIndex(r'walletIndex', objects);
  }

  List<Id> putAllByWalletIndexSync(List<WalletEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'walletIndex', objects, saveLinks: saveLinks);
  }

  Future<WalletEntity?> getByMasterId(String masterId) {
    return getByIndex(r'masterId', [masterId]);
  }

  WalletEntity? getByMasterIdSync(String masterId) {
    return getByIndexSync(r'masterId', [masterId]);
  }

  Future<bool> deleteByMasterId(String masterId) {
    return deleteByIndex(r'masterId', [masterId]);
  }

  bool deleteByMasterIdSync(String masterId) {
    return deleteByIndexSync(r'masterId', [masterId]);
  }

  Future<List<WalletEntity?>> getAllByMasterId(List<String> masterIdValues) {
    final values = masterIdValues.map((e) => [e]).toList();
    return getAllByIndex(r'masterId', values);
  }

  List<WalletEntity?> getAllByMasterIdSync(List<String> masterIdValues) {
    final values = masterIdValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'masterId', values);
  }

  Future<int> deleteAllByMasterId(List<String> masterIdValues) {
    final values = masterIdValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'masterId', values);
  }

  int deleteAllByMasterIdSync(List<String> masterIdValues) {
    final values = masterIdValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'masterId', values);
  }

  Future<Id> putByMasterId(WalletEntity object) {
    return putByIndex(r'masterId', object);
  }

  Id putByMasterIdSync(WalletEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'masterId', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByMasterId(List<WalletEntity> objects) {
    return putAllByIndex(r'masterId', objects);
  }

  List<Id> putAllByMasterIdSync(List<WalletEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'masterId', objects, saveLinks: saveLinks);
  }
}

extension WalletEntityQueryWhereSort
    on QueryBuilder<WalletEntity, WalletEntity, QWhere> {
  QueryBuilder<WalletEntity, WalletEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhere> anyWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        const IndexWhereClause.any(indexName: r'walletIndex'),
      );
    });
  }
}

extension WalletEntityQueryWhere
    on QueryBuilder<WalletEntity, WalletEntity, QWhereClause> {
  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> idNotEqualTo(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> idGreaterThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> idLessThan(Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> idBetween(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
      walletIndexEqualTo(int walletIndex) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'walletIndex',
        value: [walletIndex],
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause> masterIdEqualTo(
      String masterId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'masterId',
        value: [masterId],
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterWhereClause>
      masterIdNotEqualTo(String masterId) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId',
              lower: [],
              upper: [masterId],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId',
              lower: [masterId],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId',
              lower: [masterId],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId',
              lower: [],
              upper: [masterId],
              includeUpper: false,
            ));
      }
    });
  }
}

extension WalletEntityQueryFilter
    on QueryBuilder<WalletEntity, WalletEntity, QFilterCondition> {
  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      createdAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> idEqualTo(
      Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> idGreaterThan(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> idLessThan(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> idBetween(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'masterId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'masterId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'masterId',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      masterIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'masterId',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      sortOrderEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'sortOrder',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> sourceEqualTo(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> sourceBetween(
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      sourceContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'source',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition> sourceMatches(
      String pattern,
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'source',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      sourceIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'source',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      sourceIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'source',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      walletIndexEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
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

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      walletNameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'walletName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      walletNameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'walletName',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      walletNameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'walletName',
        value: '',
      ));
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterFilterCondition>
      walletNameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'walletName',
        value: '',
      ));
    });
  }
}

extension WalletEntityQueryObject
    on QueryBuilder<WalletEntity, WalletEntity, QFilterCondition> {}

extension WalletEntityQueryLinks
    on QueryBuilder<WalletEntity, WalletEntity, QFilterCondition> {}

extension WalletEntityQuerySortBy
    on QueryBuilder<WalletEntity, WalletEntity, QSortBy> {
  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      sortByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      sortByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortByMasterId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortByMasterIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      sortByWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> sortByWalletName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      sortByWalletNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.desc);
    });
  }
}

extension WalletEntityQuerySortThenBy
    on QueryBuilder<WalletEntity, WalletEntity, QSortThenBy> {
  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      thenByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      thenByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenByMasterId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenByMasterIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenBySortOrderDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sortOrder', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenBySource() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenBySourceDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'source', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      thenByWalletIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletIndex', Sort.desc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy> thenByWalletName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.asc);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QAfterSortBy>
      thenByWalletNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'walletName', Sort.desc);
    });
  }
}

extension WalletEntityQueryWhereDistinct
    on QueryBuilder<WalletEntity, WalletEntity, QDistinct> {
  QueryBuilder<WalletEntity, WalletEntity, QDistinct>
      distinctByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'createdAtMillis');
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QDistinct> distinctByMasterId(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'masterId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QDistinct> distinctBySortOrder() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sortOrder');
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QDistinct> distinctBySource(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'source', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QDistinct> distinctByWalletIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletIndex');
    });
  }

  QueryBuilder<WalletEntity, WalletEntity, QDistinct> distinctByWalletName(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'walletName', caseSensitive: caseSensitive);
    });
  }
}

extension WalletEntityQueryProperty
    on QueryBuilder<WalletEntity, WalletEntity, QQueryProperty> {
  QueryBuilder<WalletEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<WalletEntity, int, QQueryOperations> createdAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'createdAtMillis');
    });
  }

  QueryBuilder<WalletEntity, String, QQueryOperations> masterIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'masterId');
    });
  }

  QueryBuilder<WalletEntity, int, QQueryOperations> sortOrderProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sortOrder');
    });
  }

  QueryBuilder<WalletEntity, String, QQueryOperations> sourceProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'source');
    });
  }

  QueryBuilder<WalletEntity, int, QQueryOperations> walletIndexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletIndex');
    });
  }

  QueryBuilder<WalletEntity, String, QQueryOperations> walletNameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'walletName');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetAccountEntityCollection on Isar {
  IsarCollection<AccountEntity> get accountEntitys => this.collection();
}

const AccountEntitySchema = CollectionSchema(
  name: r'AccountEntity',
  id: -996322080142432925,
  properties: {
    r'accountId': PropertySchema(
      id: 0,
      name: r'accountId',
      type: IsarType.string,
    ),
    r'accountIndex': PropertySchema(
      id: 1,
      name: r'accountIndex',
      type: IsarType.long,
    ),
    r'accountName': PropertySchema(
      id: 2,
      name: r'accountName',
      type: IsarType.string,
    ),
    r'createdAtMillis': PropertySchema(
      id: 3,
      name: r'createdAtMillis',
      type: IsarType.long,
    ),
    r'masterId': PropertySchema(
      id: 4,
      name: r'masterId',
      type: IsarType.string,
    ),
    r'ss58Address': PropertySchema(
      id: 5,
      name: r'ss58Address',
      type: IsarType.string,
    )
  },
  estimateSize: _accountEntityEstimateSize,
  serialize: _accountEntitySerialize,
  deserialize: _accountEntityDeserialize,
  deserializeProp: _accountEntityDeserializeProp,
  idName: r'id',
  indexes: {
    r'masterId_accountIndex': IndexSchema(
      id: -4526954551786447048,
      name: r'masterId_accountIndex',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'masterId',
          type: IndexType.hash,
          caseSensitive: true,
        ),
        IndexPropertySchema(
          name: r'accountIndex',
          type: IndexType.value,
          caseSensitive: false,
        )
      ],
    ),
    r'accountId': IndexSchema(
      id: -1591555361937770434,
      name: r'accountId',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'accountId',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    ),
    r'ss58Address': IndexSchema(
      id: 5333651859904869202,
      name: r'ss58Address',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'ss58Address',
          type: IndexType.hash,
          caseSensitive: true,
        )
      ],
    )
  },
  links: {},
  embeddedSchemas: {},
  getId: _accountEntityGetId,
  getLinks: _accountEntityGetLinks,
  attach: _accountEntityAttach,
  version: '3.1.0+1',
);

int _accountEntityEstimateSize(
  AccountEntity object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.accountId.length * 3;
  bytesCount += 3 + object.accountName.length * 3;
  bytesCount += 3 + object.masterId.length * 3;
  bytesCount += 3 + object.ss58Address.length * 3;
  return bytesCount;
}

void _accountEntitySerialize(
  AccountEntity object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeString(offsets[0], object.accountId);
  writer.writeLong(offsets[1], object.accountIndex);
  writer.writeString(offsets[2], object.accountName);
  writer.writeLong(offsets[3], object.createdAtMillis);
  writer.writeString(offsets[4], object.masterId);
  writer.writeString(offsets[5], object.ss58Address);
}

AccountEntity _accountEntityDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = AccountEntity();
  object.accountId = reader.readString(offsets[0]);
  object.accountIndex = reader.readLong(offsets[1]);
  object.accountName = reader.readString(offsets[2]);
  object.createdAtMillis = reader.readLong(offsets[3]);
  object.id = id;
  object.masterId = reader.readString(offsets[4]);
  object.ss58Address = reader.readString(offsets[5]);
  return object;
}

P _accountEntityDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readString(offset)) as P;
    case 1:
      return (reader.readLong(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    case 3:
      return (reader.readLong(offset)) as P;
    case 4:
      return (reader.readString(offset)) as P;
    case 5:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _accountEntityGetId(AccountEntity object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _accountEntityGetLinks(AccountEntity object) {
  return [];
}

void _accountEntityAttach(
    IsarCollection<dynamic> col, Id id, AccountEntity object) {
  object.id = id;
}

extension AccountEntityByIndex on IsarCollection<AccountEntity> {
  Future<AccountEntity?> getByMasterIdAccountIndex(
      String masterId, int accountIndex) {
    return getByIndex(r'masterId_accountIndex', [masterId, accountIndex]);
  }

  AccountEntity? getByMasterIdAccountIndexSync(
      String masterId, int accountIndex) {
    return getByIndexSync(r'masterId_accountIndex', [masterId, accountIndex]);
  }

  Future<bool> deleteByMasterIdAccountIndex(String masterId, int accountIndex) {
    return deleteByIndex(r'masterId_accountIndex', [masterId, accountIndex]);
  }

  bool deleteByMasterIdAccountIndexSync(String masterId, int accountIndex) {
    return deleteByIndexSync(
        r'masterId_accountIndex', [masterId, accountIndex]);
  }

  Future<List<AccountEntity?>> getAllByMasterIdAccountIndex(
      List<String> masterIdValues, List<int> accountIndexValues) {
    final len = masterIdValues.length;
    assert(accountIndexValues.length == len,
        'All index values must have the same length');
    final values = <List<dynamic>>[];
    for (var i = 0; i < len; i++) {
      values.add([masterIdValues[i], accountIndexValues[i]]);
    }

    return getAllByIndex(r'masterId_accountIndex', values);
  }

  List<AccountEntity?> getAllByMasterIdAccountIndexSync(
      List<String> masterIdValues, List<int> accountIndexValues) {
    final len = masterIdValues.length;
    assert(accountIndexValues.length == len,
        'All index values must have the same length');
    final values = <List<dynamic>>[];
    for (var i = 0; i < len; i++) {
      values.add([masterIdValues[i], accountIndexValues[i]]);
    }

    return getAllByIndexSync(r'masterId_accountIndex', values);
  }

  Future<int> deleteAllByMasterIdAccountIndex(
      List<String> masterIdValues, List<int> accountIndexValues) {
    final len = masterIdValues.length;
    assert(accountIndexValues.length == len,
        'All index values must have the same length');
    final values = <List<dynamic>>[];
    for (var i = 0; i < len; i++) {
      values.add([masterIdValues[i], accountIndexValues[i]]);
    }

    return deleteAllByIndex(r'masterId_accountIndex', values);
  }

  int deleteAllByMasterIdAccountIndexSync(
      List<String> masterIdValues, List<int> accountIndexValues) {
    final len = masterIdValues.length;
    assert(accountIndexValues.length == len,
        'All index values must have the same length');
    final values = <List<dynamic>>[];
    for (var i = 0; i < len; i++) {
      values.add([masterIdValues[i], accountIndexValues[i]]);
    }

    return deleteAllByIndexSync(r'masterId_accountIndex', values);
  }

  Future<Id> putByMasterIdAccountIndex(AccountEntity object) {
    return putByIndex(r'masterId_accountIndex', object);
  }

  Id putByMasterIdAccountIndexSync(AccountEntity object,
      {bool saveLinks = true}) {
    return putByIndexSync(r'masterId_accountIndex', object,
        saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByMasterIdAccountIndex(List<AccountEntity> objects) {
    return putAllByIndex(r'masterId_accountIndex', objects);
  }

  List<Id> putAllByMasterIdAccountIndexSync(List<AccountEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'masterId_accountIndex', objects,
        saveLinks: saveLinks);
  }

  Future<AccountEntity?> getByAccountId(String accountId) {
    return getByIndex(r'accountId', [accountId]);
  }

  AccountEntity? getByAccountIdSync(String accountId) {
    return getByIndexSync(r'accountId', [accountId]);
  }

  Future<bool> deleteByAccountId(String accountId) {
    return deleteByIndex(r'accountId', [accountId]);
  }

  bool deleteByAccountIdSync(String accountId) {
    return deleteByIndexSync(r'accountId', [accountId]);
  }

  Future<List<AccountEntity?>> getAllByAccountId(List<String> accountIdValues) {
    final values = accountIdValues.map((e) => [e]).toList();
    return getAllByIndex(r'accountId', values);
  }

  List<AccountEntity?> getAllByAccountIdSync(List<String> accountIdValues) {
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

  Future<Id> putByAccountId(AccountEntity object) {
    return putByIndex(r'accountId', object);
  }

  Id putByAccountIdSync(AccountEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'accountId', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByAccountId(List<AccountEntity> objects) {
    return putAllByIndex(r'accountId', objects);
  }

  List<Id> putAllByAccountIdSync(List<AccountEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'accountId', objects, saveLinks: saveLinks);
  }

  Future<AccountEntity?> getBySs58Address(String ss58Address) {
    return getByIndex(r'ss58Address', [ss58Address]);
  }

  AccountEntity? getBySs58AddressSync(String ss58Address) {
    return getByIndexSync(r'ss58Address', [ss58Address]);
  }

  Future<bool> deleteBySs58Address(String ss58Address) {
    return deleteByIndex(r'ss58Address', [ss58Address]);
  }

  bool deleteBySs58AddressSync(String ss58Address) {
    return deleteByIndexSync(r'ss58Address', [ss58Address]);
  }

  Future<List<AccountEntity?>> getAllBySs58Address(
      List<String> ss58AddressValues) {
    final values = ss58AddressValues.map((e) => [e]).toList();
    return getAllByIndex(r'ss58Address', values);
  }

  List<AccountEntity?> getAllBySs58AddressSync(List<String> ss58AddressValues) {
    final values = ss58AddressValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'ss58Address', values);
  }

  Future<int> deleteAllBySs58Address(List<String> ss58AddressValues) {
    final values = ss58AddressValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'ss58Address', values);
  }

  int deleteAllBySs58AddressSync(List<String> ss58AddressValues) {
    final values = ss58AddressValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'ss58Address', values);
  }

  Future<Id> putBySs58Address(AccountEntity object) {
    return putByIndex(r'ss58Address', object);
  }

  Id putBySs58AddressSync(AccountEntity object, {bool saveLinks = true}) {
    return putByIndexSync(r'ss58Address', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllBySs58Address(List<AccountEntity> objects) {
    return putAllByIndex(r'ss58Address', objects);
  }

  List<Id> putAllBySs58AddressSync(List<AccountEntity> objects,
      {bool saveLinks = true}) {
    return putAllByIndexSync(r'ss58Address', objects, saveLinks: saveLinks);
  }
}

extension AccountEntityQueryWhereSort
    on QueryBuilder<AccountEntity, AccountEntity, QWhere> {
  QueryBuilder<AccountEntity, AccountEntity, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension AccountEntityQueryWhere
    on QueryBuilder<AccountEntity, AccountEntity, QWhereClause> {
  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause> idEqualTo(
      Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(
        lower: id,
        upper: id,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause> idNotEqualTo(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause> idGreaterThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause> idLessThan(
      Id id,
      {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause> idBetween(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdEqualToAnyAccountIndex(String masterId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'masterId_accountIndex',
        value: [masterId],
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdNotEqualToAnyAccountIndex(String masterId) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [],
              upper: [masterId],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [],
              upper: [masterId],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdAccountIndexEqualTo(String masterId, int accountIndex) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'masterId_accountIndex',
        value: [masterId, accountIndex],
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdEqualToAccountIndexNotEqualTo(String masterId, int accountIndex) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId],
              upper: [masterId, accountIndex],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId, accountIndex],
              includeLower: false,
              upper: [masterId],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId, accountIndex],
              includeLower: false,
              upper: [masterId],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'masterId_accountIndex',
              lower: [masterId],
              upper: [masterId, accountIndex],
              includeUpper: false,
            ));
      }
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdEqualToAccountIndexGreaterThan(
    String masterId,
    int accountIndex, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'masterId_accountIndex',
        lower: [masterId, accountIndex],
        includeLower: include,
        upper: [masterId],
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdEqualToAccountIndexLessThan(
    String masterId,
    int accountIndex, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'masterId_accountIndex',
        lower: [masterId],
        upper: [masterId, accountIndex],
        includeUpper: include,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      masterIdEqualToAccountIndexBetween(
    String masterId,
    int lowerAccountIndex,
    int upperAccountIndex, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.between(
        indexName: r'masterId_accountIndex',
        lower: [masterId, lowerAccountIndex],
        includeLower: includeLower,
        upper: [masterId, upperAccountIndex],
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      accountIdEqualTo(String accountId) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'accountId',
        value: [accountId],
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      ss58AddressEqualTo(String ss58Address) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IndexWhereClause.equalTo(
        indexName: r'ss58Address',
        value: [ss58Address],
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterWhereClause>
      ss58AddressNotEqualTo(String ss58Address) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'ss58Address',
              lower: [],
              upper: [ss58Address],
              includeUpper: false,
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'ss58Address',
              lower: [ss58Address],
              includeLower: false,
              upper: [],
            ));
      } else {
        return query
            .addWhereClause(IndexWhereClause.between(
              indexName: r'ss58Address',
              lower: [ss58Address],
              includeLower: false,
              upper: [],
            ))
            .addWhereClause(IndexWhereClause.between(
              indexName: r'ss58Address',
              lower: [],
              upper: [ss58Address],
              includeUpper: false,
            ));
      }
    });
  }
}

extension AccountEntityQueryFilter
    on QueryBuilder<AccountEntity, AccountEntity, QFilterCondition> {
  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdEqualTo(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdGreaterThan(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdLessThan(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdBetween(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdStartsWith(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdEndsWith(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'accountId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'accountId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountId',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'accountId',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIndexEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIndexGreaterThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'accountIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIndexLessThan(
    int value, {
    bool include = false,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'accountIndex',
        value: value,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountIndexBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'accountIndex',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'accountName',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'accountName',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'accountName',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'accountName',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      accountNameIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'accountName',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      createdAtMillisEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'createdAtMillis',
        value: value,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition> idEqualTo(
      Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'id',
        value: value,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition> idLessThan(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition> idBetween(
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

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'masterId',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'masterId',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'masterId',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'masterId',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      masterIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'masterId',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressEqualTo(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        include: include,
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.lessThan(
        include: include,
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.between(
        property: r'ss58Address',
        lower: lower,
        includeLower: includeLower,
        upper: upper,
        includeUpper: includeUpper,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressStartsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.startsWith(
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressEndsWith(
    String value, {
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.endsWith(
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.contains(
        property: r'ss58Address',
        value: value,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.matches(
        property: r'ss58Address',
        wildcard: pattern,
        caseSensitive: caseSensitive,
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.equalTo(
        property: r'ss58Address',
        value: '',
      ));
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterFilterCondition>
      ss58AddressIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(FilterCondition.greaterThan(
        property: r'ss58Address',
        value: '',
      ));
    });
  }
}

extension AccountEntityQueryObject
    on QueryBuilder<AccountEntity, AccountEntity, QFilterCondition> {}

extension AccountEntityQueryLinks
    on QueryBuilder<AccountEntity, AccountEntity, QFilterCondition> {}

extension AccountEntityQuerySortBy
    on QueryBuilder<AccountEntity, AccountEntity, QSortBy> {
  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> sortByAccountId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByAccountIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByAccountIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountIndex', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByAccountIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountIndex', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> sortByAccountName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountName', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByAccountNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountName', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> sortByMasterId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortByMasterIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> sortBySs58Address() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58Address', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      sortBySs58AddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58Address', Sort.desc);
    });
  }
}

extension AccountEntityQuerySortThenBy
    on QueryBuilder<AccountEntity, AccountEntity, QSortThenBy> {
  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenByAccountId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByAccountIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountId', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByAccountIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountIndex', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByAccountIndexDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountIndex', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenByAccountName() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountName', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByAccountNameDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'accountName', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByCreatedAtMillisDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'createdAtMillis', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenByMasterId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenByMasterIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'masterId', Sort.desc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy> thenBySs58Address() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58Address', Sort.asc);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QAfterSortBy>
      thenBySs58AddressDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'ss58Address', Sort.desc);
    });
  }
}

extension AccountEntityQueryWhereDistinct
    on QueryBuilder<AccountEntity, AccountEntity, QDistinct> {
  QueryBuilder<AccountEntity, AccountEntity, QDistinct> distinctByAccountId(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'accountId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QDistinct>
      distinctByAccountIndex() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'accountIndex');
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QDistinct> distinctByAccountName(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'accountName', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QDistinct>
      distinctByCreatedAtMillis() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'createdAtMillis');
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QDistinct> distinctByMasterId(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'masterId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AccountEntity, AccountEntity, QDistinct> distinctBySs58Address(
      {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'ss58Address', caseSensitive: caseSensitive);
    });
  }
}

extension AccountEntityQueryProperty
    on QueryBuilder<AccountEntity, AccountEntity, QQueryProperty> {
  QueryBuilder<AccountEntity, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<AccountEntity, String, QQueryOperations> accountIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'accountId');
    });
  }

  QueryBuilder<AccountEntity, int, QQueryOperations> accountIndexProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'accountIndex');
    });
  }

  QueryBuilder<AccountEntity, String, QQueryOperations> accountNameProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'accountName');
    });
  }

  QueryBuilder<AccountEntity, int, QQueryOperations> createdAtMillisProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'createdAtMillis');
    });
  }

  QueryBuilder<AccountEntity, String, QQueryOperations> masterIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'masterId');
    });
  }

  QueryBuilder<AccountEntity, String, QQueryOperations> ss58AddressProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'ss58Address');
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
