#!/usr/bin/env bash
# Builds the pdfcer Android APK.
#
# Usage: bash tools/android/build-apk.sh [--emulator]
#   default     arm64-v8a, for phones
#   --emulator  x86_64 as well, for the Android emulator
#
# Output: android/app/build/outputs/apk/release/app-release.apk
# Needs: the Android SDK with NDK 27.2.12479018, cargo-ndk, the Rust targets
# aarch64-linux-android (and x86_64-linux-android for --emulator), and the JDK
# bundled with Android Studio. Exit 0 on success; any failing step stops the
# script with that step's code.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export ANDROID_HOME="${ANDROID_HOME:-$LOCALAPPDATA/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
export JAVA_HOME="${JAVA_HOME_ANDROID:-/c/Program Files/Android/Android Studio/jbr}"

TARGETS=(-t arm64-v8a)
[[ "${1:-}" == "--emulator" ]] && TARGETS+=(-t x86_64)

cd "$ROOT"
# An ABI this run does not build must not ship its previous library: the APK
# would carry a stale .so, and on that ABI a newly added native method is an
# UnsatisfiedLinkError.
rm -rf android/app/src/main/jniLibs
# No debug info in the shipped library: unstripped it is near 300 MB per ABI,
# and Gradle's packager runs out of memory on it.
export CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_STRIP=debuginfo
# --locked: the engine is pinned by branch, and this build must not move it.
cargo ndk "${TARGETS[@]}" --platform 28 -o android/app/src/main/jniLibs \
    build --release --locked -p pdfcer-android

GRADLE="$(ls -d "$HOME"/.gradle/wrapper/dists/gradle-8.1.1-bin/*/gradle-8.1.1/bin/gradle 2>/dev/null | head -1)"
[[ -x "$GRADLE" ]] || { echo "build-apk: Gradle 8.1.1 not found under ~/.gradle/wrapper/dists" >&2; exit 2; }
cd android
printf 'sdk.dir=%s\n' "$(cygpath -m "$ANDROID_HOME")" > local.properties
"$GRADLE" --no-daemon -q assembleRelease
ls -l app/build/outputs/apk/release/app-release.apk
