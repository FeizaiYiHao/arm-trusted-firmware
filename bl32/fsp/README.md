FSP only targets AArch64 on QEMU. It is a stripped-down version of TSP and binds Rust code. The TSP
part is not cleanly separated yet and it is not self-contained either. It needs to be compiled
together with TF-A. The Rust code so far simply calls a C function that prints out a log message.

The following is the call sequence.

* The entry point is fsp_entrypoint, which is in asm/fsp_entrypoint.S.
* It calls fsp_c_main(), which is in fsp_c_main.c. This is in C.
* It calls fsp_main(), which is in fsp/src/lib.rs. This is in Rust.
* It calls fsp_print_debug_message(), which is in fsp_c_main.c. This is in C.

The build command is:

```bash
$ make PLAT=qemu MBEDTLS_DIR=/home/stevko/dev/mbedtls TRUSTED_BOARD_BOOT=1 GENERATE_COT=1 DEBUG=1 LOG_LEVEL=70 BL33=/home/stevko/dev/bin/bl33.bin SPD=fspd all certificates
```

The run command is:

```bash
$ qemu-system-aarch64 -nographic -smp 1 -s -machine virt,secure=on -cpu cortex-a57 -d unimp -semihosting-config enable,target=native -m 1057 -bios bl1.bin
```
