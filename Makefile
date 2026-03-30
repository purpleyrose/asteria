ARCH := $(shell uname -m)

ifeq ($(ARCH), arm64)
	BOOTLOADER_TARGET := aarch64-unknown-uefi
	KERNEL_TARGET     := aarch64-unknown-none
	EFI_NAME          := BOOTAA64.EFI
	QEMU              := qemu-system-aarch64
	MACHINE           := -machine virt -cpu cortex-a72
	BIOS              := -bios /opt/homebrew/share/qemu/edk2-aarch64-code.fd
else
	BOOTLOADER_TARGET := x86_64-unknown-uefi
	KERNEL_TARGET     := x86_64-unknown-none
	EFI_NAME          := BOOTX64.EFI
	QEMU              := qemu-system-x86_64
	MACHINE           :=
	BIOS              := -bios /opt/homebrew/share/ovmf/OVMF.fd
endif

.PHONY: build build-bootloader build-kernel run

build: build-bootloader build-kernel

build-bootloader:
	cargo build --release --target $(BOOTLOADER_TARGET) --manifest-path asteria-bootloader/Cargo.toml

build-kernel:
	cargo build --release --target $(KERNEL_TARGET) --manifest-path asteria-kernel/Cargo.toml

run: build
	mkdir -p esp/EFI/BOOT
	cp asteria-bootloader/target/$(BOOTLOADER_TARGET)/release/asteria-bootloader.efi esp/EFI/BOOT/$(EFI_NAME)
	$(QEMU) \
		$(MACHINE) \
		$(BIOS) \
		-drive format=raw,file=fat:rw:esp \
		-nographic
