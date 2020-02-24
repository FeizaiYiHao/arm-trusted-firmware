///! Currently some definitions are in qemu_defs.h

///! Secure DRAM memory
pub const BL32_MEM_BASE: usize = 0x0e100000;
pub const BL32_MEM_SIZE: usize = 0x00f00000; // This is 15MB.

///! QEMU PL011 console related constants
pub const UART0_BASE: usize = 0x09000000;
//pub const UART1_BASE: usize = 0x09040000;
pub const UART0_CLK_IN_HZ: u32 = 1;
//pub const UART1_CLK_IN_HZ: u32 = 1;

pub const PLAT_QEMU_BOOT_UART_BASE: usize = UART0_BASE;
pub const PLAT_QEMU_BOOT_UART_CLK_IN_HZ: u32 = UART0_CLK_IN_HZ;

//pub const PLAT_QEMU_CRASH_UART_BASE: usize = UART1_BASE;
//pub const PLAT_QEMU_CRASH_UART_CLK_IN_HZ: u32 = UART1_CLK_IN_HZ;

pub const PLAT_QEMU_CONSOLE_BAUDRATE: u32 = 115200;
pub const CONSOLE_FLAG_BOOT: u32 = 1 << 0;
pub const CONSOLE_FLAG_RUNTIME: u32 = 1 << 1;
pub const CONSOLE_FLAG_SCOPE_MASK: u32 = (1 << 8) - 1;

pub const PLATFORM_MAX_CPUS_PER_CLUSTER: usize = 4;
//pub const PLATFORM_CLUSTER_COUNT: usize = 2;
pub const PLATFORM_CLUSTER0_CORE_COUNT: usize = PLATFORM_MAX_CPUS_PER_CLUSTER;
pub const PLATFORM_CLUSTER1_CORE_COUNT: usize = PLATFORM_MAX_CPUS_PER_CLUSTER;
pub const PLATFORM_CORE_COUNT: usize = PLATFORM_CLUSTER0_CORE_COUNT + PLATFORM_CLUSTER1_CORE_COUNT;
