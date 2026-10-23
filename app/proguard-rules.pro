# JNI: имена методов должны совпадать с символами Rust (core/src/jni_api.rs).
-keep class dev.tytebyte.tytbrowser.core.NativeCore { *; }
-keepclasseswithmembernames class * { native <methods>; }
-keep class dev.tytebyte.tytbrowser.core.XrayNative { *; }
