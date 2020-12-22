plugins {
    // Apply the org.jetbrains.kotlin.jvm Plugin to add support for Kotlin.
    kotlin("jvm") version "1.4.21"
    kotlin("plugin.serialization") version "1.4.21"
    // Apply the application plugin to add support for building a CLI application in Java.
    application
    id("com.github.johnrengelman.shadow") version "6.1.0"
    id("org.mikeneck.graalvm-native-image") version "1.0.0"
}

group = "marais"
version = "0.1.0"

repositories {
    mavenLocal()
    mavenCentral()
    jcenter()
}

dependencies {
    val kotlinVersion: String by project
    val vlcjVersion: String by project

    implementation(platform(kotlin("bom", kotlinVersion)))
    implementation(kotlin("stdlib-jdk8"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.0.1")
    implementation("uk.co.caprica:vlcj:$vlcjVersion")
}

application {
    // Define the main class for the application.
    mainClass.set("marais.vibin.AppKt")
    mainClassName = mainClass.get()
}

java {
    targetCompatibility = JavaVersion.VERSION_11
}

tasks {
    withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
        kotlinOptions {
            jvmTarget = JavaVersion.VERSION_11.toString()
            //javaParameters = true
            //freeCompilerArgs = listOf("-Xemit-jvm-type-annotations")
        }
    }

    withType(JavaCompile::class).configureEach {
        options.encoding = "UTF-8"
    }

    shadowJar {
        mergeServiceFiles()
        //minimize()
    }
}

nativeImage {
    graalVmHome = System.getenv("GRAALVM_HOME")
    mainClass = application.mainClass.get()
    executableName = rootProject.name
    outputDirectory = file("$buildDir/bin")
    arguments(
        "--no-fallback",
        "--enable-all-security-services",
        options.traceClassInitialization("com.example.MyDataProvider,com.example.MyDataConsumer").get(),
        "--initialize-at-run-time=com.example.runtime",
        "--report-unsupported-elements-at-runtime"
    )
}

generateNativeImageConfig {
    enabled = true
    byRunningApplicationWithoutArguments()
}
