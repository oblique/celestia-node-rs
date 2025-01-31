plugins {
    alias(libs.plugins.kotlin.jvm)
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(libs.jna)
    implementation(libs.kotlinx.coroutines)
    testImplementation(libs.kotlin.test)
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()
}

tasks.register<Copy>("copyNativeLibs") {
    from("src/main/resources") // or your libs directory
    include("liblumina_node_uniffi.so")
    into("build/libs")
}

tasks.withType<AbstractCompile> {
    dependsOn("copyNativeLibs")
}
