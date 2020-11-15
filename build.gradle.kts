plugins {
    // Apply the org.jetbrains.kotlin.jvm Plugin to add support for Kotlin.
    kotlin("jvm") version "1.4.10"
    // Apply the application plugin to add support for building a CLI application in Java.
    application
    id("com.github.johnrengelman.shadow") version "6.1.0"
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
    implementation("uk.co.caprica:vlcj:$vlcjVersion")
}

application {
    // Define the main class for the application.
    mainClass.set("marais.vibin.AppKt")
    mainClassName = mainClass.get()
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
