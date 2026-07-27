// model B //index 管理员公钥派生器(创世常量用)。
//
// 用途:给一套助记词 + 数量 N,输出该助记词 `//0..//(N-1)` 的 N 个 sr25519
//   account_id(=公钥,64 位 bare hex),用于写入 citizenchain 创世常量
//   (china_cb 省储会 admins / china_ch 省储行 admins / china_zf 联邦注册局
//    FEDERAL_REGISTRY_ADMINS)。**每省一套助记词,按 //index 顺序即数组顺序。**
//
// 与 App 派生逐字节同源:核心 = `Keyring.sr25519.fromUri('<助记词>//<index>')`,
//   与 citizenwallet/citizenapp、金标 `test/wallet/derivation_golden_test.dart`
//   完全一致。启动强制金标自检(dev 助记词 //0//1//2),不过即 exit(1) 拒绝输出,
//   从根上杜绝派生漂移。
//
// 运行(在 citizenwallet 目录):
//   dart run tool/derive_admin_pubkeys.dart            # 交互:逐次输入助记词+数量
//   dart run tool/derive_admin_pubkeys.dart --check    # 只跑金标自检后退出
//   dart run tool/derive_admin_pubkeys.dart --rust      # 输出 `hex!("..."),` 便于直接粘贴
//   printf '<助记词>\n6\n' | dart run tool/derive_admin_pubkeys.dart  # 管道
//
// 输出契约:**stdout 只有公钥**(每行一个,或 --rust 的 hex! 行);所有提示、
//   自检、统计一律走 stderr,便于 `> province.txt` 直接落文件。
//
// ⚠️ 助记词是机密:本工具只读不存、不落盘、不联网;请在你自己的机器本地运行。

import 'dart:io';

import 'package:polkadart_keyring/polkadart_keyring.dart';

/// 固定测试助记词(substrate dev 助记词,全网公开,仅自检用)。
const String _devPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

/// 金标:dev 助记词 //0 //1 //2 的 account_id(bare hex,与派生金标同源)。
const List<String> _goldenAccount0to2 = [
  '2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972',
  'b606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48',
  '46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a',
];

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

/// 派生 `<mnemonic>//<index>` 的 sr25519 account_id(64 位 bare hex)。
Future<String> _accountIdHex(String mnemonic, int index) async {
  final pair = await Keyring.sr25519.fromUri('$mnemonic//$index');
  return _hex(pair.bytes().toList(growable: false));
}

/// 金标自检:dev 助记词 //0//1//2 必须逐字节命中金标,否则拒绝一切输出。
Future<void> _selfCheck() async {
  for (var i = 0; i < _goldenAccount0to2.length; i++) {
    final got = await _accountIdHex(_devPhrase, i);
    if (got != _goldenAccount0to2[i]) {
      stderr.writeln('[FATAL] 金标自检失败,派生与 model B 金标不符,禁止使用输出。');
      stderr.writeln('  //$i 期望 ${_goldenAccount0to2[i]}');
      stderr.writeln('  //$i 实得 $got');
      exit(1);
    }
  }
  stderr.writeln('[OK] 金标自检通过(dev 助记词 //0//1//2 逐字节命中)。');
}

Future<void> main(List<String> args) async {
  final rustFormat = args.contains('--rust');
  await _selfCheck();
  if (args.contains('--check')) {
    exit(0);
  }

  stderr.writeln('—— model B 管理员公钥派生器 ——');
  stderr.writeln('逐次输入:助记词一行,数量 N 一行;派生 //0..//(N-1)。空行或 EOF 结束。');
  stderr.writeln('stdout 只输出公钥;提示与统计在 stderr。');

  var block = 0;
  while (true) {
    stderr.write('\n助记词(空行结束):');
    final mnemonic = stdin.readLineSync()?.trim();
    if (mnemonic == null || mnemonic.isEmpty) {
      stderr.writeln('结束。共 $block 组。');
      break;
    }
    stderr.write('数量 N:');
    final countLine = stdin.readLineSync()?.trim();
    final count = int.tryParse(countLine ?? '');
    if (count == null || count <= 0) {
      stderr.writeln('[跳过] 数量非法:"$countLine"');
      continue;
    }

    for (var i = 0; i < count; i++) {
      final id = await _accountIdHex(mnemonic, i);
      stdout.writeln(rustFormat ? '    hex!("$id"),' : id);
    }
    block++;
    stderr.writeln('[OK] 已输出 $count 个(//0..//${count - 1})。');
  }
}
