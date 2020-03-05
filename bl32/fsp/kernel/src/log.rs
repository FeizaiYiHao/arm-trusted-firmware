//! These macros print output to the console. They are very simple at this point and just
//! accept a single string literal and uses libc's printf.

#![macro_use]

/// This can only be used when there is dynamic memory.
#[macro_export]
macro_rules! debug {
    ( $x:expr ) => {
        #[cfg(feature = "debug")]
        {
            #[allow(unused_unsafe)] // to avoid nested unsafe warnings
            unsafe {
                use core::fmt::Write;
                writeln!(&mut crate::entrypoints::FSP_CONSOLE, "FSP DEBUG: {}", $x).unwrap();
            }
        }
    };

    ( $x:literal, $($y:expr),+ ) => {
        {
            // TODO: verify if we need a global allocator with alloc::format!
            //assert!(crate::entrypoints::FSP_ALLOC.is_initialized(), "Global allocator is not initialized");
            debug!(alloc::format!($x, $($y),+));
        }
    };
}
