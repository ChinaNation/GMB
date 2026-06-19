// 内存 fake 版本游标 store —— 测 reconcile 增量逻辑,不依赖 Isar 真库。

import 'package:wuminapp_mobile/citizen/public/data/data_version_kv.dart';

class FakeDataVersionKv implements DataVersionKv {
  String? globalVersion;
  Map<String, String> provinceVersions = {};

  int writeGlobalCalls = 0;
  int writeProvinceCalls = 0;

  @override
  Future<String?> readGlobalVersion() async => globalVersion;

  @override
  Future<void> writeGlobalVersion(String version) async {
    writeGlobalCalls++;
    globalVersion = version;
  }

  @override
  Future<Map<String, String>> readProvinceVersions() async =>
      Map<String, String>.of(provinceVersions);

  @override
  Future<void> writeProvinceVersions(Map<String, String> versions) async {
    writeProvinceCalls++;
    provinceVersions = Map<String, String>.of(versions);
  }
}
