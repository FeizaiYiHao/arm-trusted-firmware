/*
 * Copyright (c) 2015-2019, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * QEMU-specific constants. Pulled mostly from TF-A's plat/qemu/qemu/include/platform_def.h
 * but also from other header files.
 */

#ifndef QEMU_DEFS_H
#define QEMU_DEFS_H

#define PLATFORM_STACK_SIZE 0x1000

/*
 * Some data must be aligned on the biggest cache line size in the platform.
 * This is known only to the platform as it might have a combination of
 * integrated and external caches.
 */
#define CACHE_WRITEBACK_SHIFT       6
#define CACHE_WRITEBACK_GRANULE     (1 << CACHE_WRITEBACK_SHIFT)

/*
 * Partition memory into secure ROM, non-secure DRAM, secure "SRAM",
 * and secure DRAM.
 */
#define SEC_ROM_BASE            0x00000000
#define SEC_ROM_SIZE            0x00020000

#define NS_DRAM0_BASE           0x40000000
#define NS_DRAM0_SIZE           0x3de00000

#define SEC_SRAM_BASE           0x0e000000
#define SEC_SRAM_SIZE           0x00060000

#define SEC_DRAM_BASE           0x0e100000
#define SEC_DRAM_SIZE           0x00f00000

/*
 * BL3-2 specific defines.
 *
 * BL3-2 can execute from Secure SRAM, or Secure DRAM.
 */
#define BL32_SRAM_BASE          BL_RAM_BASE
#define BL32_SRAM_LIMIT         BL31_BASE
#define BL32_DRAM_BASE          SEC_DRAM_BASE
#define BL32_DRAM_LIMIT         (SEC_DRAM_BASE + SEC_DRAM_SIZE)

#define SEC_SRAM_ID         0
#define SEC_DRAM_ID         1

#if BL32_RAM_LOCATION_ID == SEC_SRAM_ID
# define BL32_MEM_BASE          BL_RAM_BASE
# define BL32_MEM_SIZE          BL_RAM_SIZE
# define BL32_BASE              BL32_SRAM_BASE
# define BL32_LIMIT             BL32_SRAM_LIMIT
#elif BL32_RAM_LOCATION_ID == SEC_DRAM_ID
# define BL32_MEM_BASE          SEC_DRAM_BASE
# define BL32_MEM_SIZE          SEC_DRAM_SIZE
# define BL32_BASE              BL32_DRAM_BASE
# define BL32_LIMIT             BL32_DRAM_LIMIT
#else
# error "Unsupported BL32_RAM_LOCATION_ID value"
#endif

#endif /* PLATFORM_DEF_H */
