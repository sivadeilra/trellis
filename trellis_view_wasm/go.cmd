@echo off
rem https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#bundlers
wasm-pack -v build --target web --out-dir web\pkg

