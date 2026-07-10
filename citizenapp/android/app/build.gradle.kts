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
        ndk {
            // citizenapp Android 正式支持真实手机常用 ARM ABI；
            // smoldot native 库也只为这两类 ABI 产出，避免 APK 混入未适配 x86。
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a"))
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
        release {
            signingConfig = signingConfigs.getByName("release")
        }
    }

    packaging {
        jniLibs {
            // 第三方 Flutter 插件可能自带 x86/x86_64 native 库；
            // 当前 citizenapp 不支持 x86 Android，打包阶段直接排除，避免出现半适配 APK。
            excludes.addAll(listOf("lib/x86/**", "lib/x86_64/**"))
        }
    }
}

dependencies {
    implementation("androidx.core:core:1.13.1")
    // ⚠️ Step0 SPIKE：硬件 auth-bound 金库需要 AndroidX BiometricPrompt + CryptoObject。
    implementation("androidx.biometric:biometric:1.1.0")
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
