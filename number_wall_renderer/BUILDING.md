# Building

You will need the `number_wall_generator` library from https://github.com/313ctric/number_wall_generator.

You should run `emsdk_env.ps1` to set up the emscripten environment, but I had issues, so I just activate every time with `emsdk.ps1 activate`

Then build with:
```
cargo build --target wasm32-unknown-emscripten --release
```