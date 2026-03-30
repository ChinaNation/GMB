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

// Force compileSdk = 36 for all library subprojects and patch isar_flutter_libs namespace.
subprojects {
    plugins.withId("com.android.library") {
        val android = extensions.getByName("android") as com.android.build.gradle.LibraryExtension
        android.compileSdk = 36
        if (name == "isar_flutter_libs" && android.namespace.isNullOrEmpty()) {
            android.namespace = "dev.isar.isar_flutter_libs"
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
