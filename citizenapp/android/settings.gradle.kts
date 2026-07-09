pluginManagement {
    val flutterSdkPath =
        run {
            val properties = java.util.Properties()
            file("local.properties").inputStream().use { properties.load(it) }
            val flutterSdkPath = properties.getProperty("flutter.sdk")
            require(flutterSdkPath != null) { "flutter.sdk not set in local.properties" }
            flutterSdkPath
        }

    includeBuild("$flutterSdkPath/packages/flutter_tools/gradle")

    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("dev.flutter.flutter-plugin-loader") version "1.0.0"
    id("com.android.application") version "8.11.1" apply false
    id("org.jetbrains.kotlin.android") version "2.2.20" apply false
    // biometric_storage 5.x 用 jvmToolchain(17) 编译；本机 JDK 为 21，
    // Foojay 解析器让 Gradle 按需自动下载匹配的 JDK 17 工具链（CI/任意机器可复现）。
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.9.0"
}

include(":app")
