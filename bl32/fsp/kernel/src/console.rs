// TODO: need to check these sizes
// A nullable fn pointer is an Option, since it's supposed to be represented the same way.
#[repr(C)]
struct Console {
    next: *const Console,
    flags: u32,
    putc: Option<extern "C" fn(i32, *const Console) -> i32>,
    getc: Option<extern "C" fn(*const Console) -> i32>,
    flush: Option<extern "C" fn(*const Console) -> i32>,
}
unsafe impl Sync for Console {} // TODO: I think this means that we don't support threads.

// PL011 console from drivers/arm/pl011.h
// TODO: need to check these sizes
#[repr(C)]
struct ConsolePl011 {
    console: Console,
    base: u32,
}
unsafe impl Sync for ConsolePl011 {} // TODO: I think this means that we don't support threads.

pub struct FspConsole {
    console: ConsolePl011,
    registered: bool,
}

impl FspConsole {
    pub const fn new() -> Self {
        Self {
            console: ConsolePl011 {
                console: Console {
                    next: core::ptr::null(),
                    flags: 0,
                    putc: None,
                    getc: None,
                    flush: None,
                },
                base: 0,
            },
            registered: false,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            use crate::qemu_constants;
            crate::extern_c_fns::console_pl011_register(
                qemu_constants::PLAT_QEMU_BOOT_UART_BASE as *const u8,
                qemu_constants::PLAT_QEMU_BOOT_UART_CLK_IN_HZ,
                qemu_constants::PLAT_QEMU_CONSOLE_BAUDRATE,
                &self.console as *const ConsolePl011 as *const u8,
            );
        }
        self.registered = true;
        self.console_set_scope(
            crate::qemu_constants::CONSOLE_FLAG_BOOT | crate::qemu_constants::CONSOLE_FLAG_RUNTIME,
        );
    }

    fn console_set_scope(&mut self, scope: u32) {
        assert!(self.registered);
        self.console.console.flags =
            (self.console.console.flags & !crate::qemu_constants::CONSOLE_FLAG_SCOPE_MASK) | scope;
    }
}

impl core::fmt::Write for FspConsole {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if let Some(putc) = self.console.console.putc {
            for c in s.bytes() {
                putc(c as i32, &self.console.console as *const Console);
            }
        } else {
            panic!("No putc initialized");
        }

        Ok(())
    }
}
