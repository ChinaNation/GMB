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

// AGP 8+ requires namespace for every Android module.
// Some transitive plugins still omit it, so patch them here until upstream catches up.
// Also force compileSdk to 36 for all subprojects (isar_flutter_libs etc.).
subprojects {
    plugins.withId("com.android.library") {
        val androidExt = extensions.findByName("android")
        if (androidExt != null) {
            // Force compileSdk = 36 for all library modules
            androidExt.javaClass.methods
                .firstOrNull { it.name == "setCompileSdk" && it.parameterCount == 1 && it.parameterTypes[0] == Int::class.java }
                ?.invoke(androidExt, 36)

            // Patch namespace for isar_flutter_libs
            if (name == "isar_flutter_libs") {
                androidExt.javaClass.methods
                    .firstOrNull { it.name == "setNamespace" && it.parameterCount == 1 }
                    ?.invoke(androidExt, "dev.isar.isar_flutter_libs")
            }
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
