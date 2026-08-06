import java.util.Properties

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val keystoreProperties = Properties()
val keystorePropertiesFile = rootProject.file("key.properties")
if (keystorePropertiesFile.exists()) {
    keystorePropertiesFile.inputStream().use { keystoreProperties.load(it) }
}

android {
    namespace = "org.citizenwallet"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = "org.citizenwallet"
        // local_auth 3.x 与新 SecureStorage 加固配置统一要求 API ≥ 24。
        minSdk = maxOf(24, flutter.minSdkVersion)
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
        ndk {
            // CitizenWallet Android 唯一支持 64 位 ARM；禁止恢复其他 ABI。
            abiFilters.add("arm64-v8a")
        }
    }

    signingConfigs {
        if (keystorePropertiesFile.exists()) {
            create("release") {
                // 正式版签名从本地 key.properties 读取，避免把私钥路径和密码写进仓库。
                val storeFilePath = keystoreProperties.getProperty("storeFile")
                check(!storeFilePath.isNullOrBlank()) { "key.properties 缺少 storeFile" }
                storeFile = file(storeFilePath)
                storePassword = keystoreProperties.getProperty("storePassword")
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        debug {
            packaging {
                jniLibs {
                    // 开发期保留原生符号：签名库崩溃栈才能用 llvm-symbolizer 反解到
                    // Rust 源码行，否则只有偏移量、无从定位。release 仍剥离，见下。
                    keepDebugSymbols.add("**/libcitizenwallet_signer.so")
                }
            }
        }
        release {
            // 存在 key.properties 时使用正式签名；否则回退到 debug，方便本地测试构建。
            signingConfig =
                if (keystorePropertiesFile.exists()) {
                    signingConfigs.getByName("release")
                } else {
                    signingConfigs.getByName("debug")
                }
            // release 不加 keepDebugSymbols：APK 保持精简，也不把内部符号随包发出。
            // 线上崩溃的反解依赖构建时留档的未剥离产物
            // android/app/src/main/jniLibs/arm64-v8a/libcitizenwallet_signer.so
            // （Cargo 侧 strip=false 保证它始终带符号），剥离只发生在打包阶段。
        }
    }

    packaging {
        jniLibs {
            // 第三方插件可能携带非 ARM64 预编译库；打包阶段统一排除，确保 APK
            // 物理上只保留 defaultConfig 声明的 arm64-v8a。
            excludes.addAll(listOf("lib/armeabi*/**", "lib/x86/**", "lib/x86_64/**"))
        }
    }
}

flutter {
    source = "../.."
}
