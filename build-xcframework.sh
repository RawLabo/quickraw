rm -rf libquickraw.xcframework
cargo build -r --target aarch64-apple-ios 
cargo build -r --target aarch64-apple-ios-sim
xcodebuild -create-xcframework -library target/aarch64-apple-ios/release/*.a -library target/aarch64-apple-ios-sim/release/*.a -output libquickraw.xcframework
cbindgen --lang c --crate quickraw --output quickraw-header.h