import 'package:wuminapp_mobile/Isar/wallet_isar.dart';

class UserIdentificationSettings {
  Future<bool> isFaceAuthEnabled() async {
    final isar = await WalletIsar.instance.db();
    final settings = await isar.walletSettingsEntitys.get(0);
    return settings?.faceAuthEnabled ?? true;
  }

  Future<void> setFaceAuthEnabled(bool enabled) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      final settings = await isar.walletSettingsEntitys.get(0) ??
          (WalletSettingsEntity()..id = 0);
      settings.faceAuthEnabled = enabled;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }
}
