docker run --rm -it -v $(pwd):/root/src -w /root/src tomaka/cargo-apk cargo apk build &&
adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk
