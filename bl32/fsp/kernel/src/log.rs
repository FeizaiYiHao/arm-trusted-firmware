//! These macros print output to the console. They are very simple at this point and just
//! accept a single string literal and uses libc's printf.

#![macro_use]

///! This is used when there is no dynamic memory.
#[macro_export]
macro_rules! static_debug {
    ( $x:literal ) => {
        #[cfg(feature = "debug")]
        {
            #[allow(unused_unsafe)] // to avoid nested unsafe warnings
            unsafe {
                use core::fmt::Write;
                writeln!(&mut crate::FSP_CONSOLE, "FSP DEBUG: {}", $x).unwrap();
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
            assert!(crate::FSP_ALLOC.is_initialized(), "Global Allocator is not initialized");
            #[allow(unused_unsafe)] // to avoid nested unsafe warnings
            unsafe {
                use core::fmt::Write;
                writeln!(&mut crate::FSP_CONSOLE, "FSP DEBUG: {}", alloc::format!($($x),+)).unwrap();
            }
        }
    };
}
