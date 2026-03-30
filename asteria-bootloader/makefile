ARCH := $(shell uname -m)

ifeq ($(ARCH), arm64)
	TARGET := aarch64-unknown-uefi
	EFI_NAME := BOOTAA64.EFI
	QEMU := qemu-system-aarch64
	MACHINE := -machine virt -cpu cortex-a72
	BIOS := -bios /opt/homebrew/share/qemu/edk2-aarch64-code.fd
else
	TARGET := x86_64-unknown-uefi
	EFI_NAME := BOOTX64.EFI
	QEMU := qemu-system-x86_64
	MACHINE :=
	BIOS := -bios /opt/homebrew/share/ovmf/OVMF.fd
endif

.PHONY: build run

build:
	cargo build --release --target $(TARGET)


run: build
	mkdir -p esp/EFI/BOOT
	cp target/$(TARGET)/release/asteria-bootloader.efi esp/EFI/BOOT/$(EFI_NAME)
	$(QEMU) \
		$(MACHINE) \
		$(BIOS) \
		-drive format=raw,file=fat:rw:esp \
		-nographic