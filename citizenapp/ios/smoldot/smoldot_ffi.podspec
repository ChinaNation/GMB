#
# CitizenApp 热端原生静态库(smoldot 轻节点 + citizen-signer sr25519 签名 + OpenMLS 聊天)。
#
# 由 scripts/build-smoldot-native.sh ios 交叉编译产出 libsmoldot.a,并从产物实抽
# exported_symbols.txt;本 podspec 逐符号生成 -Wl,-u,<符号>,清单绝不手维护——
# 手写必漂移,漏一个符号 = Release 被 -dead_strip 静默剔除(Debug 正常、Release 崩)。
#
# 为什么用静态库而不是 dylib:裸 .dylib 需要嵌入 App 并单独代码签名,且 App Store
# 要求动态库必须包在 .framework 里;静态库直接链进 App 二进制,无这些坑。
# `-force_load` 保证 `#[no_mangle]` 的 FFI 符号不被链接器当未引用剔除,
# Dart 侧因此可用 DynamicLibrary.process() 直接取到,与冷端(ios/signer)同一套做法。
#
# 符号检查要查对文件(查错了会误判成"没链接进去"):
#   Debug   → Runner.app/Runner.debug.dylib (Runner 只是 ~70KB 启动壳,
#                                            新版 Xcode 的 debug 构建把代码放
#                                            进 debug dylib)
#   Release → Runner.app/Runner
#   命令:llvm-nm -g <binary> | grep -cE 'smoldot_|citizen_'   应等于清单行数
#
symbols_path = File.expand_path('exported_symbols.txt', File.dirname(__FILE__))
unless File.exist?(symbols_path)
  raise "缺少 #{symbols_path},先运行 scripts/build-smoldot-native.sh ios 生成静态库与符号清单"
end
ffi_symbols = File.readlines(symbols_path).map(&:strip).reject(&:empty?)
raise "exported_symbols.txt 为空,静态库构建异常" if ffi_symbols.empty?

Pod::Spec.new do |s|
  s.name             = 'smoldot_ffi'
  s.version          = '1.0.0'
  s.summary          = 'CitizenApp 热端原生静态库(smoldot + sr25519 + OpenMLS)'
  s.description      = <<-DESC
CitizenApp 热端原生库:smoldot 轻节点、sr25519 原生签名(与 CitizenWallet 冷端
共用 citizenchain/crates/citizen-signer 同一份源码)、OpenMLS 端到端加密聊天。
                       DESC
  s.homepage         = 'https://github.com/ChinaNation/GMB'
  s.license          = { :type => 'Apache-2.0' }
  s.author           = { 'voyager_rhett' => 'chinanation@icloud.com' }
  s.source           = { :path => '.' }
  s.platform         = :ios, '16.0'

  # CocoaPods 要求至少有一个源文件;用一个空的占位 .m,真正的实现全在 .a 里。
  s.source_files     = 'placeholder.m'
  s.vendored_libraries = 'libsmoldot.a'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386 x86_64',
  }
  # 两件事缺一不可,否则运行时 DynamicLibrary.process().lookup 找不到符号:
  # 1) -force_load:整库加载。App 侧没有任何 ObjC/Swift 代码引用这些 Rust 符号,
  #    普通链接会把整个 .a 当无用直接跳过。
  # 2) -u(force undefined):把 FFI 符号声明为"必需"。**Release 开启 -dead_strip**,
  #    即使 force_load 进来了,没被引用的符号仍会被剔除——表现为 Debug 正常、
  #    Release 静默失效。-u 让链接器必须保留它们。
  # 必须写成 `-Wl,-u,<符号>` 而不是 `-u <符号>`:CocoaPods 会把重复的 `-u` 前缀
  # 去重合并成 `-u a b c d`,后面的被链接器当成文件名报 No such file or directory。
  s.user_target_xcconfig = {
    'OTHER_LDFLAGS' => '-force_load ${PODS_ROOT}/../smoldot/libsmoldot.a ' +
                       ffi_symbols.map { |sym| "-Wl,-u,#{sym}" }.join(' '),
  }
end
