#![no_main]
sp1_zkvm::entrypoint!(main);

extern "C" {
    fn syscall_uint256_mul(x: *mut u32, y: *mut u32);
}

#[sp1_derive::cycle_tracker]
pub fn main() {
    // 20393249858788985237231628593243673548167146579814268721945474994541877372611
    let mut x: [u8; 32] = [
        195, 166, 157, 207, 218, 220, 175, 197, 111, 65, 104, 252, 174, 190, 58, 35, 202, 211, 126,
        116, 143, 161, 250, 182, 149, 86, 86, 22, 158, 43, 22, 45,
    ];

    // 31717728572175158701898635111983295176935961585742968051419350619945173564862
    let y: [u8; 32] = [
        190, 189, 200, 77, 201, 212, 57, 105, 191, 85, 188, 85, 211, 64, 17, 192, 184, 233, 140,
        100, 211, 197, 111, 120, 221, 222, 181, 14, 35, 153, 31, 70,
    ];

    println!("cycle-tracker-start: uint256_mul");
    unsafe {
        syscall_uint256_mul(x.as_mut_ptr() as *mut u32, y.as_ptr() as *mut u32);
    }
    println!("cycle-tracker-end: uint256_mul");

    // 51322802423430141345283427329623147305270980883777033508307825758397730896826
    let c: [u8; 32] = [
        186, 187, 119, 106, 27, 127, 68, 151, 114, 22, 132, 70, 158, 202, 195, 162, 53, 206, 229,
        192, 68, 94, 65, 134, 205, 19, 237, 49, 64, 173, 119, 113,
    ];

    assert_eq!(x, c);
    println!("done");
}
