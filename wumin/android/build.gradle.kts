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
