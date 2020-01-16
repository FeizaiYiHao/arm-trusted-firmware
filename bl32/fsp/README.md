# Preliminary 
FSP only targets QEMU virt Armv8-A (AArch64). It is a stripped-down version of TSP and binds Rust
code. The TSP part is not cleanly separated yet and it is not self-contained either. It needs to be
compiled together with TF-A. The Rust code so far simply calls a C function that prints out a log
message.

The following is the call sequence.

* The entry point is fsp_entrypoint, which is in asm/fsp_entrypoint.S.
* It calls fsp_c_main(), which is in fsp_c_main.c. This is in C.
* It calls fsp_main(), which is in fsp/src/lib.rs. This is in Rust.
* It calls fsp_print_debug_message(), which is in fsp_c_main.c. This is in C.

The build command is:

```
$ make PLAT=qemu MBEDTLS_DIR=/home/stevko/dev/mbedtls TRUSTED_BOARD_BOOT=1 GENERATE_COT=1 DEBUG=1 LOG_LEVEL=70 BL33=/home/stevko/dev/bin/bl33.bin SPD=fspd all certificates
```

The run command is:

```
$ qemu-system-aarch64 -nographic -smp 1 -s -machine virt,secure=on -cpu cortex-a57 -d unimp -semihosting-config enable,target=native -m 1057 -bios bl1.bin
```

# Build Environment Setup

The instructions here assume Ubuntu 18.04.3 LTS (Bionic Beaver). The ultimate goal is to compile
[ARM Trusted Firmware-A (TF-A)](https://developer.trustedfirmware.org/dashboard/view/6/) with our
FSP.

**The instructions here also assume that everything is done under a directory `~/dev`.** So you can
either create that directory and work from there, or change it appropriately as you follow the
instructions.

## Getting Necessary Packages

We need various packages in order to compile TF-A and run it on QEMU. Install the packages as
follows.

```
$ sudo apt install make build-essential bison flex libssl-dev qemu
```

## Getting the ARM GNU Toolchain

We need to download ARM GNU toolchain in order to cross-compile our source. What we need is an
ability to compile our source so it can run on bare-metal ARM hardware. The [target
triplet](https://wiki.osdev.org/Target_Triplet) for this is `aarch64-none-elf`. If you go to [ARM's
toolchain
website](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads),
it will list many targets. Among them, we need to use AArch64 ELF bare-metal target (which is
`aarch64-none-elf`). Browse down to the section and download
`gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf.tar.xz` (or whatever the current one is). After that,
unzip the file.

```
$ tar -xf gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf.tar.xz
```

The above command will create `gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf` directory. For
convenience, add the following to your shell's startup file, e.g., `~/.profile` or `~/.bashrc` if
you use bash.

```shell
export CROSS_COMPILE=aarch64-none-elf-

if [[ ! "$PATH" == */gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf/bin* ]]; then                                                   
  export PATH=$PATH:$HOME/dev/gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf/bin
fi
```

Note that exporting `$CROSS_COMPILE` means that you are always cross-compiling. If you don't want
to do this, then you need to provide `CROSS_COMPILE=aarch64-none-elf-` to `make` every time you
compile.

The source the startup file to make it take effect (assuming you added it to `~/.profile`).

```
$ . ~/.profile
```

## Getting Mbed TLS

We need to get the source for Mbed TLS 2.16.2. This is just for compiling TF-A. Do the following.

```
$ git clone https://github.com/ARMmbed/mbedtls.git -b mbedtls-2.16.2 --depth=1
```

It will create `mbedtls` directory.

## Getting a Normal Boot Loader

TF-A requires a normal boot loader at compile time. We will use U-Boot for now. We could use
[`QEMU_EFI.fd`](http://snapshots.linaro.org/components/kernel/leg-virt-tianocore-edk2-upstream/latest/QEMU-KERNEL-AARCH64/RELEASE_GCC5/)
instead, and we might in the future. To get U-Boot, do the following.

```
$ git clone https://github.com/ARM-software/u-boot.git --depth=1
```

Then we need to compile it.

```
$ cd u-boot
$ make qemu_arm64_defconfig
$ make
```

You can test if it compiled correctly by:

```
$ qemu-system-aarch64 -nographic -machine virt -cpu cortex-a57 -bios u-boot.bin
```

Check if U-Boot boots up and `pkill` it. After that, go back to the parent directory `~/dev`.

## Getting Rust

It is best to get Rust using [`rustup`](https://rustup.rs). To install it, do the following.

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

At the prompt, just proceed with the default installation. As instructed by the installer, do the
following to add `cargo` to your $PATH.

```
$ . $HOME/.cargo/env
```

Since we need to cross-compile, we need to add a new target for Rust.

```
$ rustup target add aarch64-unknown-linux-gnu
$ rustup component add llvm-tools-preview
```

Notice that the target is not `aarch64-none-elf`. For Rust, `aarch64-unknown-linux-gnu` is good
enough although it does not target bare metal hardware. This is because we will use `![no_std]` for
our Rust code. More on this later.

## Getting Our Version of ARM Trusted Firmware-A (TF-A)

Get our own version of TF-A (this repo), which includes FSP.

```
$ git clone https://github.com/steveyko/arm-trusted-firmware.git
```

## Creating File Links for QEMU

In order to run TF-A on QEMU, it is easier to put all necessary files under a single directory and
run from there. For this, we can create a directory with file links.

```
$ mkdir bin
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/bl1.bin ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/bl2.bin ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/bl31.bin ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/bl32.bin ./bin/
$ ln -s ~/dev/u-boot/u-boot.bin ./bin/bl33.bin
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/nt_fw_content.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/nt_fw_key.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/soc_fw_content.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/soc_fw_key.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/tb_fw.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/tos_fw_content.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/tos_fw_key.crt ./bin/
$ ln -s ~/dev/arm-trusted-firmware/build/qemu/debug/trusted_key.crt ./bin/
```

## Compiling FSP with TF-A

We use TF-A's build system to compile FSP. To compile FSP with TF-A, do the following.

```
$ cd arm-trusted-firmware
$ make PLAT=qemu MBEDTLS_DIR=~/dev/mbedtls TRUSTED_BOARD_BOOT=1 GENERATE_COT=1 DEBUG=1 LOG_LEVEL=70 BL33=~/dev/bin/bl33.bin SPD=fspd all certificates
```

To test if it is built correctly, do the following.

```
$ cd ../bin
$ qemu-system-aarch64 -nographic -smp 1 -s -machine virt,secure=on -cpu cortex-a57 -d unimp -semihosting-config enable,target=native -m 1057 -bios bl1.bin
```

It will hang, but if it shows the following messages roughly at the end, it means the build was
successful. These log messages are printed out by FSP (and FSP Dispatcher).

```
VERBOSE: Calling fspd_enter_sp
VERBOSE: fsp_c_main
NOTICE:  BL32: Debug message
VERBOSE: Done with fspd_enter_sp
```
