const fs = require("fs");

const filename = "./rust.wasm";
// const filename = "./as.wasm";

const module_bytes = fs.readFileSync(filename);

(async () => {
  const mod = new WebAssembly.Module(module_bytes);
  const instance = await WebAssembly.instantiate(mod, {});

  arraySum([1, 2, 3, 4, 5], instance);
  upper("this should be uppercase", instance);
})();

// Invoke the `array_sum` exported method and
// log the result to the console
function arraySum(array, instance) {
  // copy the contents of `array` into the
  // module's memory and get the offset
  var ptr = copyMemory(array, instance);
  // invoke the module's `array_sum` exported function
  // and log the result
  var res = instance.exports.array_sum(ptr, array.length);
  console.log(`Result running ${filename}: ${res}`);

  // if running the AssemblyScript module, this should also
  // be executed, particularly for long-running modules
  //instance.exports.__release(ptr);
}

// Copy `data` into the `instance` exported memory buffer.
function copyMemory(data, instance) {
  // the `alloc` function returns an offset in
  // the module's memory to the start of the block
  var ptr = instance.exports.alloc(data.length);
  // create a typed `ArrayBuffer` at `ptr` of proper size
  var mem = new Uint8Array(instance.exports.memory.buffer, ptr, data.length);
  // copy the content of `data` into the memory buffer
  mem.set(new Uint8Array(data));
  // return the pointer
  return ptr;
}

// Invoke the `upper` function from the module
// and log the result to the console.
function upper(input, instance) {
  // transform the input string into its UTF-8
  // representation
  var bytes = new TextEncoder("utf-8").encode(input);
  // copy the contents of the string into
  // the module's memory
  var ptr = copyMemory(bytes, instance);
  // call the module's `upper` function and
  // get the offset into the memory where the
  // module wrote the result string
  var res_ptr = instance.exports.upper(ptr, bytes.length);
  // read the string from the module's memory,
  // store it, and log it to the console
  var result = readString(res_ptr, bytes.length, instance);
  console.log(result);
  // the JavaScript runtime took ownership of the
  // data returned by the module, which did not
  // deallocate it - so we need to clean it up
  deallocGuestMemory(res_ptr, bytes.length, instance);
}

// Read a string from the instance's memory.
function readString(ptr, len, instance) {
  var m = new Uint8Array(instance.exports.memory.buffer, ptr, len);
  var decoder = new TextDecoder("utf-8");
  // return a slice of size `len` from the module's
  // memory, starting at offset `ptr`
  return decoder.decode(m.slice(0, len));
}

function deallocGuestMemory(ptr, len, instance) {
  // call the module's `dealloc` function
  instance.exports.dealloc(ptr, len);
}
