#!/usr/bin/env bash

    # The cargo test runner seems to change into the crate under test's directory. Therefore, ensure
    # this script executes from the root.
    cd /Users/lsw/Code/pi-OS

    TEST_ELF=$(echo $1 | sed -e 's/.*target/target/g')
    TEST_BINARY=$(echo $1.img | sed -e 's/.*target/target/g')

	echo $TEST_BINARY
	echo $TEST_ELF

    rust-objcopy --strip-all -O binary $TEST_ELF $TEST_BINARY
    docker run -t --rm -v /Users/lsw/Code/pi-OS:/work/tutorial -w /work/tutorial -v /Users/lsw/Code/pi-OS/common:/work/common rustembedded/osdev-utils:2021.12 ruby common/tests/dispatch.rb qemu-system-aarch64 -M raspi3b -serial stdio -display none  -machine raspi3b -semihosting  -kernel $TEST_BINARY
