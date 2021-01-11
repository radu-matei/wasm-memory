## [A practical guide to WebAssembly memory][article]

### Building the code

- building the Rust Wasm module:

```
$ rustc --target wasm32-unknown-unknown --crate-type=cdylib src/lib.rs -o rust.wasm
```

- building the AssemblyScript module:

```
$ npm install
$ npm run asbuild
```

- executing the Node.js runtime:

```
$ node test.js
```

- executing the Wasmtime runtime :

```
$ cargo run
```

- building and running a local Rust executable that uses the same API as the
  Wasm module and running a Valgrind:

```
$ rustc src/lib.rs -o mem
$ ./mem
$ valgrind ./mem
```

[article]: https://radu-matei.com/blog/practical-guide-to-wasm-memory
