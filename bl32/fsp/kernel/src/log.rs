/*
 * The log output macros print output to the console. It's very simple at this point and just
 * accepts a single string literal and uses libc's printf.
 */

extern "C" {
    pub fn printf(fmt: *const u8, ...);
}

#[macro_export]
macro_rules! debug {
    ( $x:expr ) => {{
        unsafe {
            crate::log::printf(concat!("DEBUG: ", $x).as_bytes().as_ptr() as *const u8);
        }
    }};
}
