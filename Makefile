.PHONY: all clean kernel bootloader userland shell test qemu iso

# Default target
all: kernel bootloader

# Build the kernel
kernel:
	@echo "Building kernel..."
	cd kernel && cargo build --release

# Build the bootloader
bootloader:
	@echo "Building bootloader..."
	cd bootloader && cargo build --release

# Build userspace components (to be implemented in later phases)
userland:
	@echo "Userland build not yet implemented"
	# make -C libc
	# make -C userland

# Build the shell (to be implemented in later phases)
shell:
	@echo "Shell build not yet implemented"
	# make -C shell

# Clean all build artifacts
clean:
	cargo clean
	rm -f *.iso *.img
	# make -C libc clean
	# make -C shell clean
	# make -C userland clean

# Run tests
test:
	cd kernel && cargo test
	cd bootloader && cargo test

# Launch in QEMU
qemu: all
	./qemu.sh

# Generate bootable ISO
iso: all
	./iso.sh

# Help target
help:
	@echo "m5rOS Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build kernel and bootloader (default)"
	@echo "  kernel     - Build kernel only"
	@echo "  bootloader - Build bootloader only"
	@echo "  userland   - Build userspace utilities"
	@echo "  shell      - Build m5sh shell"
	@echo "  clean      - Remove all build artifacts"
	@echo "  test       - Run tests"
	@echo "  qemu       - Build and run in QEMU"
	@echo "  iso        - Generate bootable ISO image"
	@echo "  help       - Show this help message"
