#
# Copyright info needed
#

FMPD_DIR := services/spd/fspd

SPD_SOURCES := 	services/spd/fspd/fspd_common.c		\
			    services/spd/fspd/fspd_helpers.S	\
				services/spd/fspd/fspd_main.c		\
				services/spd/fspd/fspd_pm.c

NEED_BL32 := yes
