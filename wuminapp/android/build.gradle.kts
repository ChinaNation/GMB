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

// Force compileSdk = 36 for all subprojects and patch isar_flutter_libs namespace.
// isar_flutter_libs ships with a low compileSdk causing android:attr/lStar not found.
subprojects {
    afterEvaluate {
        if (extensions.findByName("android") != null) {
            val android = extensions.getByName("android") as com.android.build.gradle.BaseExtension
            if (android.compileSdkVersion == null || android.compileSdkVersion!!.substringAfter("-").toIntOrNull()?.let { it < 36 } == true) {
                android.compileSdkVersion(36)
            }
            // AGP 8+ requires namespace for every Android module.
            if (name == "isar_flutter_libs" && android.namespace.isNullOrEmpty()) {
                android.namespace = "dev.isar.isar_flutter_libs"
            }
        }
    }
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
