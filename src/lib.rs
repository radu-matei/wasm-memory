#[allow(dead_code)]
fn main() {
    test_sum();
    test_upper();
}

fn test_sum() {
    let input = vec![1 as u8, 2, 3, 4, 5];
    let ptr = alloc(input.len());
    let res: u8;
    unsafe {
        std::ptr::copy(input.as_ptr(), ptr, input.len());
        res = array_sum(ptr, input.len());
        // no need to call dealloc, since the array_sum
        // function already cleaned up the array data
        // dealloc(ptr, input.len());
    }
    println!("Result: {:#?}", res);
}

fn test_upper() {
    let input = "this should be uppercase";
    let ptr = alloc(input.as_bytes().len());
    unsafe {
        std::ptr::copy(input.as_bytes().as_ptr(), ptr, input.as_bytes().len());
        let res_ptr = upper(ptr, input.as_bytes().len());
        let data = Vec::from_raw_parts(res_ptr, input.as_bytes().len(), input.as_bytes().len());
        let output = String::from_utf8(data).unwrap();
        println!("{}", output);
        // no need to call dealloc, since `Vec::from_raw_parts`
        // takes ownership of the underlying data, which goes
        // out of scope when this unsafe block returns
        // dealloc(res_ptr, input.as_bytes().len());
    }
}

/// Allocate memory into the module's linear memory
/// and return the offset to the start of the block.
#[no_mangle]
pub fn alloc(len: usize) -> *mut u8 {
    // create a new mutable buffer with capacity `len`
    let mut buf = Vec::with_capacity(len);
    // take a mutable pointer to the buffer
    let ptr = buf.as_mut_ptr();
    // take ownership of the memory block and
    // ensure the its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(buf);
    // return the pointer so the runtime
    // can write data at this offset
    return ptr;
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let data = Vec::from_raw_parts(ptr, size, size);

    std::mem::drop(data);
}

/// Given a pointer to the start of a byte array and
/// its length, return the sum of its elements.
#[no_mangle]
pub unsafe fn array_sum(ptr: *mut u8, len: usize) -> u8 {
    // create a `Vec<u8>` from the pointer to the
    // linear memory and length
    let data = Vec::from_raw_parts(ptr, len, len);
    // actually compute the sum and return it
    data.iter().sum()
}

/// Given a pointer to the start of a byte array and
/// its length, read a string, create its uppercase
/// representation, then return the pointer in
/// memory to it.
#[no_mangle]
pub unsafe fn upper(ptr: *mut u8, len: usize) -> *mut u8 {
    // create a `Vec<u8>` from the pointer and length
    // here we could also use Rust's excellent FFI
    // libraries to read a string, but for simplicity,
    // we are using the same method as for plain byte arrays
    let data = Vec::from_raw_parts(ptr, len, len);
    // read a Rust `String` from the byte array,
    let input_str = String::from_utf8(data).unwrap();
    // transform the string to uppercase, then turn it into owned bytes
    let mut upper = input_str.to_ascii_uppercase().as_bytes().to_owned();
    let ptr = upper.as_mut_ptr();
    // take ownership of the memory block where the result string
    // is written and ensure its destructor is not
    // called whe the object goes out of scope
    // at the end of the function
    std::mem::forget(upper);
    // return the pointer to the uppercase string
    // so the runtime can read data from this offset
    ptr
}

/// The Node.js WASI runtime requires a `_start` function
/// for instantiating the module.
#[no_mangle]
pub fn _start() {}
