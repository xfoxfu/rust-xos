MODE ?= release
OVMF := OVMF.fd
ESP := esp
BUILD_ARGS := -Z build-std=core,alloc
QEMU_ARGS := -net none

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

build: $(ESP)


$(ESP): $(ESP)/EFI/BOOT/BOOTX64.EFI $(ESP)/KERNEL.ELF $(ESP)/EFI/BOOT/rboot.conf \
	$(ESP)/user0 $(ESP)/user1 $(ESP)/user2 $(ESP)/user3 \

$(ESP)/EFI/BOOT/BOOTX64.EFI: target/x86_64-unknown-uefi/$(MODE)/boot.efi
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/EFI/BOOT/rboot.conf: rboot.conf
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/KERNEL.ELF: target/x86_64-unknown-none/$(MODE)/kernel
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/user0: target/x86_64-unknown-xos/$(MODE)/user0
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/user1: target/x86_64-unknown-xos/$(MODE)/user1
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/user2: target/x86_64-unknown-xos/$(MODE)/user2
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/user3: target/x86_64-unknown-xos/$(MODE)/user3
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
	$(QEMU_ARGS)

test:
	cargo test -p fatpart
