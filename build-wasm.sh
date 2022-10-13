cargo build -r --lib --features wasm --target wasm32-unknown-unknown
echo "generating wasm bindgen"
wasm-bindgen --no-typescript --target web --out-dir pkg ./target/wasm32-unknown-unknown/release/quickraw.wasm
cd pkg
echo "optimizing wasm"
wasm-opt -O4 quickraw_bg.wasm -o quickraw_bg.wasm
echo "* wasm and js file are built under the ./pkg folder"
