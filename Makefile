QEMU_FLAGS ?= -kernel linux -initrd rootfs.cpio.lz4 -m 1024

cpio: install
	cd rootfs; find | cpio --quiet -H newc -o | lz4 -f > ../rootfs.cpio.lz4
	#cd rootfs; find | cpio --quiet -H newc -o | gzip -1 > ../rootfs.cpio.gz

install: build directory
	install target/x86_64-unknown-linux-musl/debug/init rootfs
	install target/x86_64-unknown-linux-musl/debug/schelp rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/id rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/ls rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/display rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/input rootfs/bin
	ln -rs rootfs/bin/schelp rootfs/bin/sh

# Make a copy of the root/
directory:
	rm -rf rootfs
	cp -r root rootfs

build:
	cargo build

run:
	qemu-system-x86_64 $(QEMU_FLAGS) -nographic -append "console=ttyS0 loglevel=7"

view:
	qemu-system-x86_64 $(QEMU_FLAGS) -vga virtio -append "loglevel=7"

debug:
	qemu-system-x86_64  $(QEMU_FLAGS) -nographic -append "console=ttyS0 loglevel=7"  -s -S

clean:
	rm -f rootfs.cpio.gz
	rm -fr rootfs
	cargo clean
