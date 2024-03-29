MODE ?= release
OVMF := OVMF.fd
ESP := esp
BUILD_ARGS := -Z build-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem
QEMU_ARGS ?= 

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

.PHONY: build run header asm doc .FORCE \
	target/x86_64-unknown-uefi/$(MODE)/boot.efi   \
	target/x86_64-unknown-none/$(MODE)/kernel     \
	target/x86_64-unknown-xos/$(MODE)/sampleio \
	target/x86_64-unknown-xos/$(MODE)/plota \
	target/x86_64-unknown-xos/$(MODE)/plotb \
	target/x86_64-unknown-xos/$(MODE)/plotc \
	target/x86_64-unknown-xos/$(MODE)/diskread \

build: $(ESP)


$(ESP): $(ESP)/EFI/BOOT/BOOTX64.EFI $(ESP)/KERNEL.ELF $(ESP)/EFI/BOOT/rboot.conf \
	$(ESP)/sampleio $(ESP)/plota $(ESP)/plotb $(ESP)/plotc $(ESP)/diskread \

$(ESP)/EFI/BOOT/BOOTX64.EFI: target/x86_64-unknown-uefi/$(MODE)/boot.efi
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/EFI/BOOT/rboot.conf: rboot.conf
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/KERNEL.ELF: target/x86_64-unknown-none/$(MODE)/kernel
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/sampleio: target/x86_64-unknown-xos/$(MODE)/sampleio
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/plota: target/x86_64-unknown-xos/$(MODE)/plota
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/plotb: target/x86_64-unknown-xos/$(MODE)/plotb
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/plotc: target/x86_64-unknown-xos/$(MODE)/plotc
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/diskread: target/x86_64-unknown-xos/$(MODE)/diskread
	@mkdir -p $(@D)
	cp $< $@

target/x86_64-unknown-uefi/$(MODE)/boot.efi: boot
	cargo build -p $< --target x86_64-unknown-uefi $(BUILD_ARGS)
target/x86_64-unknown-none/$(MODE)/kernel: kernel
	cargo build -p $< --target x86_64-unknown-none.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/sampleio: sampleio
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/plota: plota
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/plotb: plotb
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/plotc: plotc
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)
target/x86_64-unknown-xos/$(MODE)/diskread: diskread
	cargo build -p $< --target x86_64-unknown-xos.json $(BUILD_ARGS)

qemu: build
	qemu-system-x86_64 -bios ${OVMF} \
	-drive format=raw,file=fat:rw:$(ESP) \
	-net none \
	$(QEMU_ARGS)

test:
	cargo test -p fatpart
