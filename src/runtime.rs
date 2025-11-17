/// Runtime support functions for Edust programs

/// Print an integer value (called from generated code)
#[no_mangle]
pub extern "C" fn print_int(value: i64) -> i64 {
    println!("{}", value);
    value
}