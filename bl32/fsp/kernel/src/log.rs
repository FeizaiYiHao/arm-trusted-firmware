/*
 * The debug macro prints output to the console. It's very simple at this point and just
 * accepts a single string literal and uses libc's printf.
 */

extern "C" {
    pub fn printf(fmt: *const u8, ...);
}

#[macro_export]
macro_rules! debug {
    ( $x:expr ) => {
        #[cfg(feature = "debug")]
        {
            unsafe {
                crate::log::printf(
                    concat!("FSP: ", $x, "\n", '\0').as_bytes().as_ptr() as *const u8
                );
            }
        }
    };
}
