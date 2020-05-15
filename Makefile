MODE ?= release
EFI := target/x86_64-unknown-uefi/$(MODE)/boot.efi
OVMF := OVMF.fd
ESP := esp
BUILD_ARGS := -Z build-std=core,alloc --target x86_64-unknown-uefi
QEMU_ARGS := -net none
#	-debugcon file:debug.log -global isa-debugcon.iobase=0x402


ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

.PHONY: build run header asm doc

build:
	cargo build $(BUILD_ARGS)

clippy:
	cargo clippy $(BUILD_ARGS)

doc:
	cargo doc

uefi-run: build
	uefi-run \
		-b ${OVMF} \
		-q $(shell which qemu-system-x86_64) \
		$(EFI) \
		-- $(QEMU_ARGS)

run: build
	mkdir -p $(ESP)/EFI/Boot
	cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi
	qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

header:
	rust-objdump -h $(EFI) | less

asm:
	rust-objdump -d $(EFI) | less
