## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

include ./common/docker.mk
include ./common/format.mk
include ./common/operating_system.mk

##--------------------------------------------------------------------------------------------------
## Optional, user-provided configuration values
##--------------------------------------------------------------------------------------------------

# Default to the RPi3.
BSP ?= rpi4

ifeq ($(shell uname -s),Linux)
	DEV_SERIAL = /dev/ttyUSB1
	DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)
    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
 	DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
else
	DEV_SERIAL = /dev/cu.usbserial-AQ043M36

endif




##--------------------------------------------------------------------------------------------------
## BSP-specific configuration values
##--------------------------------------------------------------------------------------------------
QEMU_MISSING_STRING = "This board is not yet supported for QEMU."
TARGET            = aarch64-unknown-none-softfloat
KERNEL_BIN        = kernel8.img
QEMU_BINARY       = qemu-system-aarch64
QEMU_MACHINE_TYPE = raspi3b
QEMU_RELEASE_ARGS = -serial stdio -display none -machine $(QEMU_MACHINE_TYPE) 
QEMU_TEST_ARGS    = $(QEMU_RELEASE_ARGS) -semihosting
OBJDUMP_BINARY    = aarch64-none-elf-objdump
NM_BINARY         = aarch64-none-elf-nm
READELF_BINARY    = aarch64-none-elf-readelf
AS_BINARY         = aarch64-none-elf-as
LD_BINARY         = aarch64-none-elf-ld
ASM_SEARCH_PATH   = ./kernel/src/_arch/aarch64/cpu

KERNEL_LINKER_SCRIPT = kernel.ld
LD_SCRIPT_PATH    = ./kernel/src/bsp/raspberrypi/$(KERNEL_LINKER_SCRIPT)
LD_SCRIPT_FOLDER    = $(shell pwd)/kernel/src/bsp/raspberrypi/

ifeq ($(BSP),rpi3)
	AS_ARGS           = -mcpu=cortex-a53 -I $(ASM_SEARCH_PATH)
    OPENOCD_ARG       = -f /openocd/c232hm-ddhsl-0.cfg -f /openocd/rpi3.cfg
    JTAG_BOOT_IMAGE   = ./X1_JTAG_boot/jtag_boot_rpi3.img
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
else ifeq ($(BSP),rpi4)
	AS_ARGS           = -mcpu=cortex-a72 -I $(ASM_SEARCH_PATH)
    OPENOCD_ARG       = -f /openocd/c232hm-ddhsl-0.cfg -f /openocd/rpi4.cfg
    JTAG_BOOT_IMAGE   = ./X1_JTAG_boot/jtag_boot_rpi4.img
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
endif

# Export for build.rs.
export LD_SCRIPT_FOLDER

BOOT_ASM = ./kernel/src/_arch/aarch64/cpu/boot.s
ASSEMBLED_BOOT = ./target/$(TARGET)/release/boot.o

KERNEL_LIB = ./target/$(TARGET)/release/liblibkernel.a
KERNEL_LIB_DEPS = $(filter-out %: ,$(file < $(KERNEL_LIB).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)



##--------------------------------------------------------------------------------------------------
## Targets and Prerequisites
##--------------------------------------------------------------------------------------------------
KERNEL_MANIFEST      = $(shell pwd)/kernel/Cargo.toml
KERNEL_LINKER_SCRIPT_PATH =./kernel/src/bsp/raspberrypi/kernel.ld
LAST_BUILD_CONFIG    = target/$(BSP).build_config

KERNEL_ELF      = target/$(TARGET)/release/kernel
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
KERNEL_ELF_DEPS = $(filter-out %: ,$(file < $(KERNEL_ELF).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)



##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
RUSTFLAGS = $(RUSTC_MISC_ARGS)                   \
    -C link-arg=--library-path=$(LD_SCRIPT_FOLDER) \
    -C link-arg=--script=$(KERNEL_LINKER_SCRIPT) \
	--emit asm                                   \
	-C opt-level=0                               \
	-C debuginfo=2                               

RUSTFLAGS_DEBUG =  -C opt-level=0                  \
	-C debuginfo=2                               

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \

FEATURES      = --features bsp_$(BSP)
COMPILER_ARGS = --target=$(TARGET) \
    $(FEATURES)                    \
	--release                      \
	--lib

RUSTC_ARGS = -- -Z mir-opt-level=0 --emit mir 
    

RUSTC_CMD   = cargo rustc   $(COMPILER_ARGS) --manifest-path $(KERNEL_MANIFEST)
RUSTC_LIB_CMD = cargo rustc   --manifest-path $(KERNEL_MANIFEST) $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS)
TEST_CMD    = cargo test $(COMPILER_ARGS) --manifest-path $(KERNEL_MANIFEST)
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
EXEC_TEST_DISPATCH = rudy common/tests/dispatch.rb
EXEC_MINITERM      = ruby common/serial/miniterm.rb
EXEC_MINIPUSH      = ruby ./common/serial/minipush.rb

##------------------------------------------------------------------------------
## Dockerization
##------------------------------------------------------------------------------
DOCKER_CMD          = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
DOCKER_CMD_INTERACT = $(DOCKER_CMD) -i
DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/common:/work/common
DOCKER_ARG_DIR_JTAG   = -v $(shell pwd)/X1_JTAG_boot:/work/X1_JTAG_boot
DOCKER_ARG_DEV        = --privileged -v /dev:/dev
DOCKER_ARG_NET        = --network host
DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

# DOCKER_IMAGE defined in include file (see top of this file).
DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
DOCKER_GDB   = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_NET) $(DOCKER_IMAGE)

DOCKER_OPENOCD   = $(DOCKER_CMD_DEV) $(DOCKER_ARG_NET) $(DOCKER_IMAGE)

EXEC_TEST_MINIPUSH = ruby tests/chainboot_test.rb

ifeq ($(shell uname -s),Linux)
    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
    DOCKER_JTAGBOOT  = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_ARG_DIR_JTAG) $(DOCKER_IMAGE)
    DOCKER_OPENOCD   = $(DOCKER_CMD_DEV) $(DOCKER_ARG_NET) $(DOCKER_IMAGE)
endif

##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------
.PHONY: all doc qemu clippy clean readelf objdump nm check miniterm chainboot test_unit $(KERNEL_LIB) 


all: $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Save the configuration as a file, so make understands if it changed.
##------------------------------------------------------------------------------
$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)



##------------------------------------------------------------------------------
## Generate the documentation
##------------------------------------------------------------------------------
doc:
	$(call color_header, "Generating docs")
	@$(DOC_CMD) --document-private-items --open


miniterm:
	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)

##------------------------------------------------------------------------------
## Run the kernel in QEMU
##------------------------------------------------------------------------------
ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu qemuasm:
	$(call color_header, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu: FEATURES := --features build_qemu

qemu: qemu_exec

qemu_exec: $(KERNEL_BIN)
	$(call color_header, "Launching QEMU")
	@$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) 

qemuasm: $(KERNEL_BIN)
	$(call color_header, "Launching QEMU with ASM output")
	@$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
endif

##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
	rm -rf target $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Run readelf
##------------------------------------------------------------------------------
readelf: $(KERNEL_ELF)
	$(call color_header, "Launching readelf")
	@$(DOCKER_TOOLS) $(READELF_BINARY) --headers $(KERNEL_LIB) 

##------------------------------------------------------------------------------
## Run objdump
##------------------------------------------------------------------------------
objdump: $(KERNEL_ELF)
	$(call color_header, "Launching objdump")
	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                --section .rodata   \
                --section .bss   \
                $(KERNEL_ELF) | rustfilt


#objdump: $(KERNEL_ELF)
#	$(call color_header, "Launching objdump")
#	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) -d $(KERNEL_ELF) | rustfilt | less
	
##------------------------------------------------------------------------------
## Run nm
##------------------------------------------------------------------------------
nm: $(KERNEL_ELF)
	$(call color_header, "Launching nm")
	@$(DOCKER_TOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort | rustfilt


mir: ${KERNEL_ELF_DEPS}
	$(call color_header, "Generating Rust mir")
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD) $(RUSTC_ARGS)


chainboot : $(KERNEL_BIN)
	@$(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)


gdb: $(KERNEL_BIN)
	$(call color_header, "Launching GDB")
	@$(DOCKER_GDB) gdb-multiarch -q $(KERNEL_ELF)

openocd:
	$(call color_header, "Launching OpenOCD")
	@$(DOCKER_OPENOCD) openocd $(OPENOCD_ARG)


jtagboot: 
	@$(DOCKER_JTAGBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(JTAG_BOOT_IMAGE)

test_unit: FEATURES := --features test_build --features bsp_rpi3

define KERNEL_TEST_RUNNER
#!/usr/bin/env bash

    # The cargo test runner seems to change into the crate under test's directory. Therefore, ensure
    # this script executes from the root.
    cd $(shell pwd)

    TEST_ELF=$$(echo $$1 | sed -e 's/.*target/target/g')
    TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')

	# $(DOCKER_TOOLS) $(READELF_BINARY) --headers $$TEST_ELF 
	# $(DOCKER_TOOLS) $(NM_BINARY) --demangle --print-size $$TEST_ELF | sort | rustfilt
    $(OBJCOPY_CMD) $$TEST_ELF $$TEST_BINARY
    $(DOCKER_TEST) ruby common/tests/dispatch.rb $(EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
endef

export KERNEL_TEST_RUNNER

define test_prepare
    @mkdir -p target
    @echo "$$KERNEL_TEST_RUNNER" > target/kernel_test_runner.sh
    @chmod +x target/kernel_test_runner.sh
endef

test_unit:
	$(call color_header, "Compiling unit test(s) - $(BSP)")
	$(call test_prepare)
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD) 



$(KERNEL_LIB): $(KERNEL_ELF_DEPS) 
	$(call color_header, "Compiling kernel static lib - $(BSP)")
	@RUSTFLAGS="$(RUSFTFLAGS_DEBUG)" $(RUSTC_LIB_CMD)

$(ASSEMBLED_BOOT): $(BOOT_ASM)
	$(call color_header, "Assembling boot.s")
	@$(DOCKER_TOOLS) $(AS_BINARY) $(AS_ARGS)  -o $(ASSEMBLED_BOOT) $(BOOT_ASM)


$(KERNEL_ELF): $(KERNEL_LIB) $(ASSEMBLED_BOOT)
	$(call color_header, "Linking kernel ELF - $(BSP)")
	@$(DOCKER_TOOLS) aarch64-none-elf-ld -T  $(LD_SCRIPT_PATH) -n -o $(KERNEL_ELF) $(ASSEMBLED_BOOT) $(KERNEL_LIB) 

$(KERNEL_BIN): $(KERNEL_ELF)
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)
	$(call color_progress_prefix, "Name")
	@echo $(KERNEL_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(KERNEL_BIN))
