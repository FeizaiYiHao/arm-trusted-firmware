#
# Modeled after bl32/tsp/tsp.mk
#

BL32_SOURCES		+=	bl32/fsp/init/fsp_c_main.c				\
						bl32/fsp/init/fsp_qemu_console_init.c	\
						bl32/fsp/init/fsp_plat_helpers.S		\
						bl32/fsp/init/fsp_entrypoint.S		\
						bl32/fsp/init/fsp_exceptions.S

BL32_LINKERFILE		:=	bl32/fsp/init/fsp.ld.S

SPIN_ON_FSP			:=	0

$(eval $(call add_define,SPIN_ON_FSP))

#
# The following is for building the Rust source as a library.
# We assume a debug build for now.
#

FSP_RUST_ROOT		:=	bl32/fsp/kernel

FSP_RUST_SOURCES	:=	${FSP_RUST_ROOT}/src/lib.rs		\
						${FSP_RUST_ROOT}/src/log.rs		\
						${FSP_RUST_ROOT}/Cargo.toml

BL32_LIBS			:=	-lfsp

LIB_FSP				:=	${BUILD_PLAT}/lib/libfsp.a

.PHONY: ${BL32_LIBS}

${BL32_LIBS}: ${LIB_FSP}

${BUILD_PLAT}/bl32/bl32.elf: ${LIB_FSP}

${LIB_FSP}: ${FSP_RUST_SOURCES}
	$(ECHO) "Building FSP in Rust"
	$(Q)cd ${FSP_RUST_ROOT} && cargo build --target aarch64-unknown-linux-gnu
	$(Q)cp ${FSP_RUST_ROOT}/target/aarch64-unknown-linux-gnu/debug/libfsp.a ${LIB_FSP}
