alias l := load_kernel
alias c := chainloader
alias t := test_kernel
alias m := miniterm

set positional-arguments := true

docker_img := "rustembedded/osdev-utils:2021.12"
docker_arg := "run -t --rm -v " + `pwd`+ ":/work/tutorial -w /work/tutorial " + docker_img
as_binary  := "aarch64-none-elf-as"
ld_binary  := "aarch64-none-elf-ld"
asm_path   := "./kernel/src/_arch/aarch64/cpu"
ld_path    := asm_path

target  := "aarch64-unknown-none-softfloat"
output_path := "./target"/target

kernel_manifest := `pwd`/"kernel/Cargo.toml"

dev_serial := "/dev/cu.usbserial-AQ043M36"

test_rustc_flags := "-C target-cpu=cortex-a72 -C link-arg=--library-path=./target/aarch64-unknown-none-softfloat -C link-arg=--library=:test-boot.o -C link-arg=--script=./kernel/src/_arch/aarch64/cpu/test.ld"




default: load_kernel

load_kernel: (build_kernel "kernel") 
    @ruby ./common/serial/minipush.rb {{dev_serial}} kernel.img
    

test_kernel: (build_kernel "test") 

chainloader: (build_kernel "chainloader")   



compile_boot TARGET:
    @if [ "{{TARGET}}" == "kernel" ];then \
        docker {{docker_arg}} {{as_binary}} -mcpu=cortex-a72 -I {{asm_path}} -o {{output_path}}/{{TARGET}}-boot.o {{asm_path}}/{{TARGET}}-boot.s;\
    elif [ "{{TARGET}}" == "test" ];then \
        docker {{docker_arg}} {{as_binary}} -mcpu=cortex-a72 -I {{asm_path}} --defsym QEMU_MODE=1 -o {{output_path}}/{{TARGET}}-boot.o {{asm_path}}/{{TARGET}}-boot.s;\
    else \
        docker {{docker_arg}} {{as_binary}} -mcpu=cortex-a72 -I {{asm_path}} -o {{output_path}}/{{TARGET}}-boot.o {{asm_path}}/{{TARGET}}-boot.s;\
    fi


compile_lib TARGET: (compile_boot TARGET)  
    @if [ "{{TARGET}}" == "kernel" ];then \
        cargo rustc --manifest-path {{kernel_manifest}} --features bsp_rpi4 --lib --release;\
    elif [ "{{TARGET}}" == "test" ];then \
        RUSTFLAGS="{{test_rustc_flags}}" cargo test --target=aarch64-unknown-none-softfloat --manifest-path {{kernel_manifest}} --features build_qemu --lib;\
    else \
        cargo rustc --manifest-path {{kernel_manifest}} --features build_chainloader --lib --release;\
    fi

build_kernel TARGET: (compile_lib TARGET)
    @if [ "{{TARGET}}" == "kernel" ];then \
        docker {{docker_arg}} {{ld_binary}} -T {{ld_path}}/{{TARGET}}.ld -n -o {{TARGET}}.elf {{output_path}}/{{TARGET}}-boot.o {{output_path}}/release/liblibkernel.a && rust-objcopy --strip-all -O binary {{TARGET}}.elf {{TARGET}}.img;\
    elif [ "{{TARGET}}" == "test" ];then \
        echo "";\
    else \
        docker {{docker_arg}} {{ld_binary}} -T {{ld_path}}/{{TARGET}}.ld -n -o {{TARGET}}.elf {{output_path}}/{{TARGET}}-boot.o {{output_path}}/release/liblibkernel.a && rust-objcopy --strip-all -O binary {{TARGET}}.elf {{TARGET}}.img;\
    fi




miniterm:
    @ruby ./common/serial/miniterm.rb {{dev_serial}}
    
