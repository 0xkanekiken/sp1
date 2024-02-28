#![no_main]
sp1_zkvm::entrypoint!(main);

extern "C" {
    fn syscall_blake2b_compress_inner(p: *mut u32, q: *const u32);
}

pub fn main() {
    // The input message and state are simply 0, 1, ..., 128.
    let input_message: [u8; 128] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
    ];

    let mut input_state: [u8; 128] = [
        127, 126, 125, 124, 123, 122, 121, 120, 119, 118, 117, 116, 115, 114, 113, 112, 111, 110,
        109, 108, 107, 106, 105, 104, 103, 102, 101, 100, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90,
        89, 88, 87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 69, 68, 67,
        66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44,
        43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21,
        20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    ];

    unsafe {
        syscall_blake2b_compress_inner(
            input_state.as_mut_ptr() as *mut u32,
            input_message.as_ptr() as *const u32,
        );
    }

    // The expected output state is the result of compress_inner.
    let output_state: [u8; 128] = [
        142, 97, 21, 48, 181, 38, 102, 120, 181, 181, 132, 2, 142, 34, 222, 200, 91, 59, 143, 244,
        192, 97, 180, 164, 129, 45, 52, 152, 137, 242, 140, 229, 60, 67, 137, 84, 69, 146, 94, 4,
        63, 200, 66, 9, 251, 190, 228, 189, 246, 163, 176, 213, 232, 193, 63, 40, 215, 186, 99, 34,
        210, 63, 47, 34, 195, 223, 240, 69, 9, 223, 197, 237, 92, 137, 53, 107, 116, 51, 96, 126,
        16, 216, 41, 178, 160, 235, 153, 202, 173, 222, 5, 69, 113, 47, 242, 201, 244, 241, 27,
        158, 243, 255, 164, 92, 137, 119, 128, 242, 236, 187, 87, 137, 92, 128, 158, 82, 183, 213,
        237, 49, 150, 67, 160, 164, 47, 13, 34, 200,
    ];

    assert_eq!(input_state, output_state);

    println!("done");
}
