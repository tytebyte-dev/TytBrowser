plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "dev.tytebyte.tytbrowser"
    compileSdk = 34

    defaultConfig {
        applicationId = "dev.tytebyte.tytbrowser"
        minSdk = 21
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
        // Нативное ядро собирается только под эти ABI (см. задачу cargoNdk ниже).
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64") }
    }

    buildFeatures { buildConfig = true }

    signingConfigs {
        // Постоянный отладочный ключ из репозитория: APK с любой машины и из CI подписан одинаково,
        // поэтому приложение можно ставить поверх предыдущей версии без удаления.
        getByName("debug") {
            storeFile = file("debug.keystore")
            storePassword = "android"
            keyAlias = "tytdebug"
            keyPassword = "android"
        }
        // Подпись релиза берётся из переменных окружения (CI) — иначе используется постоянный debug-ключ (app/debug.keystore).
        val ks = System.getenv("TYT_KEYSTORE")
        if (ks != null) {
            create("release") {
                storeFile = file(ks)
                storePassword = System.getenv("TYT_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("TYT_KEY_ALIAS")
                keyPassword = System.getenv("TYT_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            signingConfig = signingConfigs.findByName("release") ?: signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }

    packaging {
        // Без этого .so сжимаются внутри APK и распаковываются при установке — лишние мегабайты.
        jniLibs { useLegacyPackaging = false }
    }
}

// ── Rust-ядро ──────────────────────────────────────────────────────────────
// cargo-ndk кладёт libtytcore.so в app/src/main/jniLibs/<abi>/.
// Пропустить (например, если .so уже собраны): ./gradlew assembleDebug -PskipRust
val cargoNdk = tasks.register<Exec>("cargoNdk") {
    group = "build"
    description = "Собирает libtytcore.so из core/ через cargo-ndk"
    workingDir = rootProject.file("core")
    commandLine(
        "cargo", "ndk",
        "--platform", "21",
        "-t", "arm64-v8a", "-t", "armeabi-v7a", "-t", "x86_64",
        "-o", file("src/main/jniLibs").absolutePath,
        "build", "--release",
    )
}
// ── Xray-core (Go) ─────────────────────────────────────────────────────────
// scripts/build-xray.sh собирает libtytxray.so (оригинальное ядро Xray) под все ABI.
val goXray = tasks.register<Exec>("goXray") {
    group = "build"
    description = "Собирает libtytxray.so из xraycore/ через Go + NDK"
    workingDir = rootProject.projectDir
    commandLine("bash", "scripts/build-xray.sh")
}
if (!project.hasProperty("skipGo")) {
    tasks.matching { it.name == "preBuild" }.configureEach { dependsOn(goXray) }
}

if (!project.hasProperty("skipRust")) {
    tasks.matching { it.name == "preBuild" }.configureEach { dependsOn(cargoNdk) }
}

dependencies {
    // ProxyController: направляет трафик WebView во встроенный прокси (Xray).
    implementation("androidx.webkit:webkit:1.11.0")
}
