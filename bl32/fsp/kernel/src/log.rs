//! These macros print output to the console. They are very simple at this point and just
//! accept a single string literal and uses libc's printf.

#![macro_use]

#[cfg(feature = "debug")]
#[allow(dead_code)] // since this won't be used if debug! is not called anywhere
extern "C" {
    pub fn printf(fmt: *const u8, ...);
}

///! This is used when there is no dynamic memory.
#[macro_export]
macro_rules! static_debug {
    ( $x:literal ) => {
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

///! This can only be used when there is dynamic memory.
#[macro_export]
macro_rules! debug {
    ( $($x:expr),+ ) => {
        #[cfg(feature = "debug")]
        {
            let s = alloc::format!($($x),+);
            let s = alloc::format!("FSP DEBUG: {}{}{}", s, '\n', '\0');
            #[allow(unused_unsafe)] // to avoid nested unsafe warnings
            unsafe {
                crate::log::printf(s.as_bytes().as_ptr() as *const u8);
            }
        }
    };
}
