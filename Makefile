MODE ?= release
OVMF := OVMF.fd
ESP := esp
BUILD_ARGS := -Z build-std=core,alloc
QEMU_ARGS := -net none
SYSROOT_IMG := sysroot.img
SYSROOT := sysroot

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

.PHONY: build run header asm doc .FORCE \
	target/x86_64-unknown-uefi/$(MODE)/boot.efi   \
	target/x86_64-unknown-none/$(MODE)/kernel     \
	target/x86_64-unknown-xos/$(MODE)/user0 \
	target/x86_64-unknown-xos/$(MODE)/user1 \
	target/x86_64-unknown-xos/$(MODE)/user2 \
	target/x86_64-unknown-xos/$(MODE)/user3 \

build: $(ESP) $(SYSROOT_IMG)

$(SYSROOT_IMG): executables.txt \
	target/x86_64-unknown-xos/$(MODE)/user0 \
	target/x86_64-unknown-xos/$(MODE)/user1 \
	target/x86_64-unknown-xos/$(MODE)/user2 \
	target/x86_64-unknown-xos/$(MODE)/user3 \

	dd if=/dev/zero of=$@ bs=512 count=2880
	# mkfs.fat $@ -F12
	# cd $< && mcopy -si ../$@ . ::/
	dd if=executables.txt of=$@ bs=512 seek=0 conv=notrunc
	dd if=target/x86_64-unknown-xos/$(MODE)/user0 of=$@ bs=512 seek=1 conv=notrunc
	dd if=target/x86_64-unknown-xos/$(MODE)/user1 of=$@ bs=512 seek=33 conv=notrunc
	dd if=target/x86_64-unknown-xos/$(MODE)/user2 of=$@ bs=512 seek=65 conv=notrunc
	dd if=target/x86_64-unknown-xos/$(MODE)/user3 of=$@ bs=512 seek=97 conv=notrunc

# $(SYSROOT):
# 	@mkdir -p $(SYSROOT)

$(ESP): $(ESP)/EFI/BOOT/BOOTX64.EFI $(ESP)/KERNEL.ELF $(ESP)/EFI/BOOT/rboot.conf
$(ESP)/EFI/BOOT/BOOTX64.EFI: target/x86_64-unknown-uefi/$(MODE)/boot.efi
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/EFI/BOOT/rboot.conf: rboot.conf
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/KERNEL.ELF: target/x86_64-unknown-none/$(MODE)/kernel
	@mkdir -p $(@D)
	cp $< $@

target/x86_64-unknown-uefi/$(MODE)/boot.efi: boot
	cargo build -p $< --target x86_64-unknown-uefi $(BUILD_ARGS)
target/x86_64-unknown-none/$(MODE)/kernel: kernel
	cargo build -p $< --target x86_64-unknown-none.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/user0: user0
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/user1: user1
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/user2: user2
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/user3: user3
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)

qemu: build
	qemu-system-x86_64 -bios ${OVMF} \
	-drive format=raw,file=fat:rw:$(ESP) \
	-drive format=raw,file=$(SYSROOT_IMG) \
	$(QEMU_ARGS)

test:
	cargo test -p fatpart
