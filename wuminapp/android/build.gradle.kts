allprojects {
    repositories {
        google()
        mavenCentral()
    }

    // Force compileSdk = 36 for ALL Android modules (app + library).
    // isar_flutter_libs ships with a low compileSdk causing android:attr/lStar not found.
    // Must be in allprojects so it fires when each module applies the Android plugin,
    // BEFORE evaluationDependsOn triggers evaluation.
    plugins.withType<com.android.build.gradle.BasePlugin> {
        extensions.configure<com.android.build.gradle.BaseExtension> {
            compileSdkVersion(36)
        }
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

// Patch namespace for isar_flutter_libs (AGP 8+ requires it).
subprojects {
    if (name == "isar_flutter_libs") {
        plugins.withId("com.android.library") {
            extensions.configure<com.android.build.gradle.LibraryExtension> {
                if (namespace.isNullOrEmpty()) {
                    namespace = "dev.isar.isar_flutter_libs"
                }
            }
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
