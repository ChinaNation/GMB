import java.io.FileInputStream
import java.util.Properties

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val releaseKeystoreProperties = Properties()
val releaseKeystorePropertiesFile = rootProject.file("key.properties")
if (releaseKeystorePropertiesFile.exists()) {
    releaseKeystoreProperties.load(FileInputStream(releaseKeystorePropertiesFile))
}

fun signingValue(propertyName: String): String? {
    val propertyValue = releaseKeystoreProperties.getProperty(propertyName)
    if (!propertyValue.isNullOrBlank()) {
        return propertyValue.trim()
    }
    return null
}

val releaseStoreFile = signingValue("storeFile")
val releaseStorePassword = signingValue("storePassword")
val releaseKeyAlias = signingValue("keyAlias")
val releaseKeyPassword = signingValue("keyPassword")
val hasReleaseSigningConfig = listOf(
    releaseStoreFile,
    releaseStorePassword,
    releaseKeyAlias,
    releaseKeyPassword,
).all { !it.isNullOrBlank() }

android {
    namespace = "org.citizenapp"
    compileSdk = 36
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = "org.citizenapp"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk {
            // CitizenApp Android 唯一支持 64 位 ARM；禁止恢复其他 ABI。
            abiFilters.add("arm64-v8a")
        }
    }

    signingConfigs {
        create("release") {
            if (hasReleaseSigningConfig) {
                // 正式 APK 签名只接受固定 release keystore，保证后续 Android 更新能匹配同一签名证书。
                storeFile = rootProject.file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        debug {
            packaging {
                jniLibs {
                    // 开发期保留原生符号:smoldot 原生崩溃(SIGABRT/panic)的栈帧才能
                    // 用 llvm-symbolizer 反解到 Rust 源码行,否则只有偏移量、无从定位。
                    // 只作用于 debug 包(约 +55M);release 仍剥离,见下方说明。
                    keepDebugSymbols.add("**/libsmoldot.so")
                }
            }
        }
        release {
            signingConfig = signingConfigs.getByName("release")
            // release 不加 keepDebugSymbols:APK 保持精简,且不把 2.4 万个内部函数名
            // (含密码学/密钥存储符号)随包发出。线上崩溃的反解依赖构建时留档的未剥离
            // 产物 android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so(Cargo 侧
            // strip=false 保证它始终带符号),剥离只发生在打包阶段。
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

dependencies {
    implementation("androidx.core:core:1.13.1")
    // ⚠️ Step0 SPIKE：硬件 auth-bound 金库需要 AndroidX BiometricPrompt + CryptoObject。
    implementation("androidx.biometric:biometric:1.1.0")
    androidTestImplementation("androidx.test:runner:1.6.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
}

gradle.taskGraph.whenReady {
    val runsReleaseTask = allTasks.any { task ->
        task.path.contains("Release", ignoreCase = true)
    }
    if (runsReleaseTask && !hasReleaseSigningConfig) {
        throw GradleException(
            "citizenapp Android release 构建缺少正式签名配置。请在 citizenapp/android/key.properties " +
                "配置 storeFile/storePassword/keyAlias/keyPassword；CI 手动发布由 GMB_APP_KEY 自动生成该文件。"
        )
    }
}

flutter {
    source = "../.."
}
