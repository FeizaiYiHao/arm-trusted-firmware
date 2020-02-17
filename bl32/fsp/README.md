# Preliminary 
FSP only targets QEMU virt Armv8-A (AArch64). It is a stripped-down version of TSP and binds Rust
code. The TSP part is not cleanly separated yet and it is not self-contained either. It needs to be
compiled together with ARM Trusted Firmware-A (TF-A). The Rust code so far simply calls a C
function that prints out a log message. This is just to demonstrate that we can boot our own
[Secure-EL1
Payload (SP)](https://trustedfirmware-a.readthedocs.io/en/latest/getting_started/image-terminology.html#secure-el1-payload-sp-ap-bl32)
written in Rust.

The following is the call sequence.

* The entry point is fsp_entrypoint, which is in init/fsp_entrypoint.S.
* It calls fsp_c_main(), which is in init/fsp_c_main.c. This is in C.
* It calls fsp_main(), which is in fsp/src/lib.rs. This is in Rust.
* It calls fsp_print_debug_message(), which is in init/fsp_c_main.c. This is in C.

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

The above commands install the stable version of Rust. However, we sometimes use experimental
features that can only be compiled by using a nightly version of Rust. So install a nightly Rust.
We also need to install extra components as we need to cross-compile.

```
$ rustup toolchain install nightly
$ rustup default nightly
```

We also need the Rust source and `cargo-xbuild` in order to build libcore correctly for our target
and environment.

```
$ rustup component add rust-src
$ cargo install cargo-xbuild
```

Note that our target triplet is `aarch64-unknown-none-softfloat` (specified in fsp.mk). This
is because we're using `![no_std]` and running on bare metal hardware. `softfloat` means that we're
disabling floating point and SIMD registers. Enabling those registers does not work as it is
prevented by TF-A.  `aarch64-unknown-none-softfloat` is a [tier-3
target](https://forge.rust-lang.org/release/platform-support.html).  Because of that, we need to be
aware that it may cause a problem.

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
$ make PLAT=qemu MBEDTLS_DIR=~/dev/mbedtls TRUSTED_BOARD_BOOT=1 GENERATE_COT=1 DEBUG=1 LOG_LEVEL=50 BL33=~/dev/bin/bl33.bin SPD=fspd all certificates
```

To test if it is built correctly, do the following.

```
$ cd ../bin
$ qemu-system-aarch64 -nographic -smp 1 -s -machine virt,secure=on -cpu cortex-a57 -d unimp -semihosting-config enable,target=native -m 1057 -bios bl1.bin
```

It will hang, but if it shows the messages similar to the following, it means the build was
successful. These log messages are printed out by FSP.

```
INFO: BL31: Initializing BL32
FSP DEBUG: fsp debug
INFO: BL31: Preparing for EL3 exit to normal world
INFO: Entry point address = 0x60000000
```

## Critical Missing Pieces

Right now, it doesn't do much except printing out debug messages. Even when it prints out debugging
messages, it uses the existing TF-A's libc and console driver. The following are probably the
critical pieces that are needed right away.

### Memory Management

There is not even basic heap support yet. This is necessary to do anything useful and serious, so
it is urgently needed. It's probably not that we need to have a sophisticated paging system right
now, but we do need basic heap allocation/deallocation support.

### Testing Setup

Rust has a testing framework and we can use this to do unit testing. We need to take a look and see
how we can leverage it.

### Standard Library Functions

Once we have some heap support, we can perhaps implement standard library functions. Probably not
everything is needed, but some of them will be helpful, e.g., String, vec, etc.

### Exceptions and Interrupts

Currently, we register dummy handlers for exceptions and interrupts. We need to implement real
handlers.
