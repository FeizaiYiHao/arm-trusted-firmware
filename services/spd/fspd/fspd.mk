#
# Copyright info needed
#

FMPD_DIR := services/spd/fspd

SPD_SOURCES := 	services/spd/fspd/fspd_common.c		\
			    services/spd/fspd/fspd_helpers.S	\
				services/spd/fspd/fspd_main.c		\
				services/spd/fspd/fspd_pm.c

# This dispatcher is paired with the FSP source, so include the FSP's Makefile.
include bl32/fsp/fsp.mk

# Let the top-level Makefile know that we intend to build the SP from source
NEED_BL32 := yes

SPIN_ON_FSPD := 0
$(eval $(call add_define,SPIN_ON_FSPD))
