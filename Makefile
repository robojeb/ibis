# This is defining our Rust target, we will use the musl C library 
TARGET=x86_64-unknown-linux-musl

# We will need a Linux Kernel to run in Qemu so make sure that is downloaded and built here
KERNEL_MAJOR_VERSION=5
KERNEL_MINOR_VERSION=13
KERNEL_VERSION=$(KERNEL_MAJOR_VERSION).$(KERNEL_MINOR_VERSION)
KERNEL_DIRECTORY=linux-$(KERNEL_VERSION)
KERNEL_ARCHIVE=$(KERNEL_DIRECTORY).tar.xz
KERNEL_URL=https://cdn.kernel.org/pub/linux/kernel/v$(KERNEL_MAJOR_VERSION).x/$(KERNEL_ARCHIVE)

.PHONY: all
all: vmlinuz initramfs

.PHONY: rust_build
rust_build: 
	cargo build --all --target=$(TARGET)

# Clean only the rust dependencies
.PHONY: clean
clean: 
	cargo clean
	rm -rf ./rfs ./rfs_update initramfs

# Clean rust and Linux dependencies
.PHONY: cleaner
cleaner: clean
	rm -rf $(KERNEL_ARCHIVE) $(KERNEL_DIRECTORY) vmlinuz


#
# Run in qemu
#
.PHONY: run
run: initramfs vmlinuz
#	echo "run"
	qemu-system-x86_64 -m 2048 -kernel vmlinuz -initrd initramfs -nographic --append console=ttyS0

#
# Build an initramfs for QEMU
#
initramfs: | rfs build_initramfs

#FIXME: This always rebuilds the initramfs even if the RFS didn't change
.PHONY: build_initramfs
build_initramfs: $(wildcard rfs/**/*)
	cd rfs && find . | cpio -o --format=newc > ../initramfs

.PHONY: rfs
rfs: | rust_build rfs_update

rfs_update: $(wildcard rfs_template/*) $(wildcard target/$(TARGET)/debug/**/*)
	mkdir -p rfs
	cp -r rfs_template/* rfs/
	cp ./target/$(TARGET)/debug/init ./rfs/
	cp ./target/$(TARGET)/debug/ibish ./rfs/
# Keep track of when we last updated the RFS so that we can build properly
	touch rfs_update

#
# Build and download Linux from sources
#

vmlinuz: $(KERNEL_DIRECTORY)
	cd $(KERNEL_DIRECTORY) && make defconfig && make -j`nproc`
	cp $(KERNEL_DIRECTORY)/arch/x86_64/boot/bzImage vmlinuz

# Build a Linux kernel
$(KERNEL_DIRECTORY):
	wget $(KERNEL_URL)
	tar xf $(KERNEL_ARCHIVE)