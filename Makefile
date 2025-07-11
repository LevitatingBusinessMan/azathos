QEMU_FLAGS ?= -kernel linux -initrd rootfs.cpio.zst -m 1024

default: build

cpio: rootfs
	cd rootfs; find | cpio --quiet -H newc -o | zstd > ../rootfs.cpio.zst

tar: rootfs
	tar -cJf azathos.tar.xz -C rootfs .

rootfs: directory
	install target/x86_64-unknown-linux-musl/debug/init rootfs/sbin
	install target/x86_64-unknown-linux-musl/debug/schelp rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/id rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/ls rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/display rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/input rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/segfault rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/ps rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/echo rootfs/bin
	ln -rs rootfs/bin/schelp rootfs/bin/sh

# Make a copy of the root/
directory:
	rm -rf rootfs
	cp -r root rootfs

build:
	cargo build

nspawn:
	sudo systemd-nspawn -D rootfs /init --link-journal=no

run:
	qemu-system-x86_64 $(QEMU_FLAGS) -nographic -append "console=ttyS0 loglevel=6"

view:
	# enabling kvm gives some issues when trying to use vga
	# vga can be enabled but the console doesn't open in it correctly
	# instead you can open the console in serial
	# qemu-system-x86_64 $(QEMU_FLAGS) -vga virtio -append "console=ttyS0 loglevel=6" -enable-kvm

	qemu-system-x86_64 $(QEMU_FLAGS) -vga virtio -append "loglevel=6"

debug:
	qemu-system-x86_64  $(QEMU_FLAGS) -nographic -append "console=ttyS0 loglevel=8"  -s -S

clean:
	rm -f rootfs.cpio*
	rm -fr rootfs
	cargo clean
