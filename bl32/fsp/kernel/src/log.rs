//! These macros print output to the console. They are very simple at this point and just
//! accept a single string literal and uses libc's printf.

#![macro_use]

#[cfg(feature = "debug")]
#[allow(dead_code)] // since this won't be used if debug! is not called anywhere
extern "C" {
    pub fn printf(fmt: *const u8, ...);
}

#[macro_export]
macro_rules! debug {
    ( $x:expr ) => {
        #[cfg(feature = "debug")]
        {
            #[allow(unused_unsafe)] // to avoid nested unsafe warnings
            unsafe {
                crate::log::printf(
                    concat!("FSP DEBUG: ", $x, '\n', '\0').as_bytes().as_ptr() as *const u8
                );
            }
        }
    };
}
