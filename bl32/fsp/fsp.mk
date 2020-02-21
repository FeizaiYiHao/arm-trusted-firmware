#
# Modeled after bl32/tsp/tsp.mk
#

BL32_SOURCES		+=	bl32/fsp/init/fsp_c_helpers.c			\
						bl32/fsp/init/fsp_entrypoint.S		\
						bl32/fsp/init/fsp_exceptions.S		\
						bl32/fsp/init/fsp_plat_helpers.S	\

BL32_LINKERFILE		:=	bl32/fsp/init/fsp.ld.S

SPIN_ON_FSP			:=	0

$(eval $(call add_define,SPIN_ON_FSP))

#
# The following is for building the Rust source as a library.
# We assume a debug build for now.
#

FSP_RUST_ROOT		:=	bl32/fsp/kernel

FSP_RUST_SOURCES	:=	${FSP_RUST_ROOT}/src/console.rs			\
						${FSP_RUST_ROOT}/src/extern_c_defs.rs	\
						${FSP_RUST_ROOT}/src/fsp_alloc.rs		\
						${FSP_RUST_ROOT}/src/lib.rs				\
						${FSP_RUST_ROOT}/src/log.rs				\
						${FSP_RUST_ROOT}/src/qemu_constants.rs	\
						${FSP_RUST_ROOT}/Cargo.toml

#
# -lc is necessary because libfsp.a depends on TF-A's libc
#
BL32_LIBS			:=	-lfsp -lc

LIB_FSP				:=	${BUILD_PLAT}/lib/libfsp.a

TARGET				:=	aarch64-unknown-none-softfloat

.PHONY: ${BL32_LIBS}

${BL32_LIBS}: ${LIB_FSP}

${BUILD_PLAT}/bl32/bl32.elf: ${LIB_FSP}

${LIB_FSP}: ${FSP_RUST_SOURCES}
	$(ECHO) "Building FSP in Rust"
	$(Q)cd ${FSP_RUST_ROOT} && cargo xbuild --target ${TARGET}
	$(Q)cp ${FSP_RUST_ROOT}/target/${TARGET}/debug/libfsp.a ${LIB_FSP}
