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

fun signingValue(propertyName: String, envName: String): String? {
    val propertyValue = releaseKeystoreProperties.getProperty(propertyName)
    if (!propertyValue.isNullOrBlank()) {
        return propertyValue.trim()
    }
    return System.getenv(envName)?.trim()?.takeIf { it.isNotEmpty() }
}

val releaseStoreFile = signingValue("storeFile", "WUMINAPP_ANDROID_STORE_FILE")
val releaseStorePassword = signingValue("storePassword", "WUMINAPP_ANDROID_STORE_PASSWORD")
val releaseKeyAlias = signingValue("keyAlias", "WUMINAPP_ANDROID_KEY_ALIAS")
val releaseKeyPassword = signingValue("keyPassword", "WUMINAPP_ANDROID_KEY_PASSWORD")
val hasReleaseSigningConfig = listOf(
    releaseStoreFile,
    releaseStorePassword,
    releaseKeyAlias,
    releaseKeyPassword,
).all { !it.isNullOrBlank() }

android {
    namespace = "org.chinanation.citizen"
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
        applicationId = "org.chinanation.citizen"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
        ndk {
            // 中文注释：wuminapp Android 正式支持真实手机常用 ARM ABI；
            // smoldot native 库也只为这两类 ABI 产出，避免 APK 混入未适配 x86。
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a"))
        }
    }

    signingConfigs {
        create("release") {
            if (hasReleaseSigningConfig) {
                // 中文注释：正式 APK 签名只接受固定 release keystore，保证后续 Android 更新能匹配同一签名证书。
                storeFile = rootProject.file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
        }
    }

    packaging {
        jniLibs {
            // 中文注释：第三方 Flutter 插件可能自带 x86/x86_64 native 库；
            // 当前 wuminapp 不支持 x86 Android，打包阶段直接排除，避免出现半适配 APK。
            excludes.addAll(listOf("lib/x86/**", "lib/x86_64/**"))
        }
    }
}

dependencies {
    implementation("androidx.core:core:1.13.1")
}

gradle.taskGraph.whenReady {
    val runsReleaseTask = allTasks.any { task ->
        task.path.contains("Release", ignoreCase = true)
    }
    if (runsReleaseTask && !hasReleaseSigningConfig) {
        throw GradleException(
            "wuminapp Android release 构建缺少正式签名配置。请在 wuminapp/android/key.properties " +
                "配置 storeFile/storePassword/keyAlias/keyPassword，或通过 CI 环境变量 " +
                "WUMINAPP_ANDROID_STORE_FILE/WUMINAPP_ANDROID_STORE_PASSWORD/" +
                "WUMINAPP_ANDROID_KEY_ALIAS/WUMINAPP_ANDROID_KEY_PASSWORD 注入。"
        )
    }
}

flutter {
    source = "../.."
}
