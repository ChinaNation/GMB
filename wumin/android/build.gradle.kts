allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)
}
subprojects {
    project.evaluationDependsOn(":app")
}

// Force Java 17 for all subprojects to suppress source/target 8 warnings.
subprojects {
    tasks.withType<JavaCompile>().configureEach {
        sourceCompatibility = JavaVersion.VERSION_17.toString()
        targetCompatibility = JavaVersion.VERSION_17.toString()
    }
}

// AGP 8+ requires namespace for every Android module.
// isar_flutter_libs 3.1.0 still omits it, so patch here until upstream catches up.
subprojects {
    if (name == "isar_flutter_libs") {
        afterEvaluate {
            val androidExt = extensions.findByName("android")
            // 兼容旧版 isar_flutter_libs：
            // 1. 补 namespace，满足 AGP 8+
            // 2. 覆盖其写死的 compileSdkVersion 30，避免 release 资源链接阶段缺少 android:attr/lStar
            val setNamespace = androidExt
                ?.javaClass
                ?.methods
                ?.firstOrNull { it.name == "setNamespace" && it.parameterCount == 1 }
            setNamespace?.invoke(androidExt, "dev.isar.isar_flutter_libs")
            val compileSdkMethods =
                androidExt
                    ?.javaClass
                    ?.methods
                    ?.filter {
                        it.parameterCount == 1 &&
                            it.name in
                                setOf("setCompileSdk", "setCompileSdkVersion", "compileSdkVersion")
                    }
                    .orEmpty()
            for (method in compileSdkMethods) {
                val applied =
                    runCatching {
                        method.invoke(androidExt, 36)
                        true
                    }.getOrElse {
                        runCatching {
                            method.invoke(androidExt, "android-36")
                            true
                        }.getOrDefault(false)
                    }
                if (applied) {
                    break
                }
            }
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
