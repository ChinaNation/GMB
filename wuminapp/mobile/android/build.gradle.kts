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
// Some transitive plugins (e.g. isar_flutter_libs 3.1.0+1) still omit it.
subprojects {
    if (name == "isar_flutter_libs") {
        plugins.withId("com.android.library") {
            val androidExt = extensions.findByName("android")
            val setNamespace = androidExt
                ?.javaClass
                ?.methods
                ?.firstOrNull { it.name == "setNamespace" && it.parameterCount == 1 }
            setNamespace?.invoke(androidExt, "dev.isar.isar_flutter_libs")
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
