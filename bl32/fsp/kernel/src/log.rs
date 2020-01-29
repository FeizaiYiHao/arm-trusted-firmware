/*
 * These macros print output to the console. They are very simple at this point and just
 * accept a single string literal and uses libc's printf.
 *
 */

#[cfg(feature = "debug")]
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
                    concat!("FSP DEBUG: ", $x, "\n", '\0').as_bytes().as_ptr() as *const u8
                );
            }
        }
    };
}
