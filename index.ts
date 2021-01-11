export function alloc(len: i32): usize {
    let buf = new Array<u8>(len);
    let buf_ptr = memory.data(8);
    store<Array<u8>>(buf_ptr, buf);
    return buf_ptr;
  }
  
  export function array_sum(buf_ptr: usize, len: i32): u8 {
      let result: u8 = 0;
      for(let i = 0; i < len; i++) {
        result += load<u8>(buf_ptr + i) as u8;
      }
      return result as u8;
  }
  
  export function abort(
    message: string | null,
    fileName: string | null,
    lineNumber: u32,
    columnNumber: u32
  ): void {}