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

.PHONY: build run header asm doc

build: $(ESP) $(SYSROOT_IMG)

$(SYSROOT_IMG): $(SYSROOT)
	dd if=/dev/zero of=$@ bs=512 count=2880
	mkfs.fat $@ -F12
	mcopy -i $@ $< ::

$(SYSROOT):
	@mkdir -p $(SYSROOT)

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

target/x86_64-unknown-uefi/$(MODE)/boot.efi: boot $(shell find boot/ -type f -name '*')
	cargo build -p $< --target x86_64-unknown-uefi $(BUILD_ARGS)
target/x86_64-unknown-none/$(MODE)/kernel: kernel $(shell find kernel/ -type f -name '*')
	cargo build -p $< --target kernel/x86_64-unknown-none.json $(BUILD_ARGS)

qemu: build
	qemu-system-x86_64 -bios ${OVMF} \
	-drive format=raw,file=fat:rw:$(ESP) \
	-drive format=raw,file=$(SYSROOT_IMG) \
	$(QEMU_ARGS)
