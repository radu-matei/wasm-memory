use wasmtime::*;
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

const WASM: &str = "rust.wasm";
// const WASM: &str = "as.wasm";
const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";
const ARRAY_SUM_FN: &str = "array_sum";
const UPPER_FN: &str = "upper";
const DEALLOC_FN: &str = "dealloc";

pub fn main() {
    test_array_sum();
    test_upper();
}

fn test_array_sum() {
    let input = vec![1 as u8, 2, 3, 4, 5];
    let res = array_sum(input).unwrap();
    println!("Result from running {}: {:#?}", WASM, res);
}

fn test_upper() {
    let input = "this should be uppercase";
    let res = upper(input.to_string()).unwrap();
    println!("Result from running {}: {:#?}", WASM, res);
}

fn upper(input: String) -> Result<String, anyhow::Error> {
    // create a new Wasmtime instance
    let instance = create_instance(WASM.to_string())?;
    // write the input array to the module's linear memory
    let ptr = copy_memory(&input.as_bytes().to_vec(), &instance)?;
    // get the module's exported `upper` function
    let upper = instance
        .get_func(UPPER_FN)
        .expect("expected upper function not found");

    // call the `upper` function with the pointer to the
    // string and length
    let results = upper.call(&vec![
        Val::from(ptr as i32),
        Val::from(input.as_bytes().len() as i32),
    ])?;
    let res_ptr = match results
        .get(0)
        .expect("expected the result of upper to have one value")
    {
        Val::I32(val) => *val,

        _ => return Err(anyhow::Error::msg("cannot get result")),
    };

    // read the result string from the module's memory,
    // which is located at `res_ptr`
    let memory = instance
        .get_memory(MEMORY)
        .expect("expected memory not found");

    let res: String;
    unsafe { res = read_string(&memory, res_ptr as u32, input.as_bytes().len() as u32).unwrap() }

    // call the module's dealloc function for the result string
    let dealloc = instance
        .get_func(DEALLOC_FN)
        .expect("expected upper function not found");

    dealloc.call(&vec![
        Val::from(res_ptr as i32),
        Val::from(input.as_bytes().len() as i32),
    ])?;

    // return the string
    Ok(res)
}

/// Invoke the module's `array_sum` exported method
/// and print the result to the console.
fn array_sum(input: Vec<u8>) -> Result<i32, anyhow::Error> {
    // create a new Wasmtime instance
    let instance = create_instance(WASM.to_string())?;
    // write the input array to the module's linear memory
    let ptr = copy_memory(&input, &instance)?;
    // get the module's exported `array_sum` function
    let array_sum = instance
        .get_func(ARRAY_SUM_FN)
        .expect("expected array_sum function not found");

    // call the `array_sum` function with the pointer to the
    // array and length
    let results = array_sum.call(&vec![Val::from(ptr as i32), Val::from(input.len() as i32)])?;
    // return the result
    match results
        .get(0)
        .expect("expected the result of array_sum to have one value")
    {
        Val::I32(val) => Ok(*val),

        _ => return Err(anyhow::Error::msg("cannot get result")),
    }
}

/// Copy a byte array into an instance's linear memory
/// and return the offset relative to the module's memory.
fn copy_memory(bytes: &Vec<u8>, instance: &Instance) -> Result<isize, anyhow::Error> {
    // Get the "memory" export of the module.
    // If the module does not export it, just panic,
    // since we are not going to be able to copy array data.
    let memory = instance
        .get_memory(MEMORY)
        .expect("expected memory not found");

    // The module is not using any bindgen libraries, so it should export
    // its own alloc function.
    //
    // Get the guest's exported alloc function, and call it with the
    // length of the byte array we are trying to copy.
    // The result is an offset relative to the module's linear memory, which is
    // used to copy the bytes into the module's memory.
    // Then, return the offset.
    let alloc = instance
        .get_func(ALLOC_FN)
        .expect("expected alloc function not found");
    let alloc_result = alloc.call(&vec![Val::from(bytes.len() as i32)])?;

    let guest_ptr_offset = match alloc_result
        .get(0)
        .expect("expected the result of the allocation to have one value")
    {
        Val::I32(val) => *val as isize,
        _ => return Err(anyhow::Error::msg("guest pointer must be Val::I32")),
    };
    unsafe {
        let raw = memory.data_ptr().offset(guest_ptr_offset);
        raw.copy_from(bytes.as_ptr(), bytes.len());
    }
    return Ok(guest_ptr_offset);
}

/// Create a Wasmtime::Instance from a compiled module and
/// link the WASI imports.
fn create_instance(filename: String) -> Result<Instance, anyhow::Error> {
    let store = Store::default();
    let mut linker = Linker::new(&store);

    let ctx = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .build()?;

    let wasi = Wasi::new(&store, ctx);
    wasi.add_to_linker(&mut linker)?;
    let module = wasmtime::Module::from_file(store.engine(), filename)?;

    let instance = linker.instantiate(&module)?;
    return Ok(instance);
}

/// Read a Rust `String` from a module's memory, given an offset and length.
pub unsafe fn read_string(
    memory: &Memory,
    data_ptr: u32,
    len: u32,
) -> Result<String, anyhow::Error> {
    // get a raw byte array from the module's linear memory
    // at offset `data_ptr` and length `len`.
    let data = memory
        .data_unchecked()
        .get(data_ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));
    // attempt to read a UTF-8 string from the memory
    let str = match data {
        Some(data) => match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return Err(anyhow::Error::msg("invalid utf-8")),
        },
        None => return Err(anyhow::Error::msg("pointer/length out of bounds")),
    };

    Ok(String::from(str))
}
