cpio: install
	#cd rootfs; find | cpio --quiet -H newc -o | lz4 -f - ../rootfs.cpio.lz4
	cd rootfs; find | cpio --quiet -H newc -o | gzip -1 > ../rootfs.cpio.gz

install: build directory
	install target/x86_64-unknown-linux-musl/debug/init rootfs
	install target/x86_64-unknown-linux-musl/debug/schelp rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/id rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/ls rootfs/bin
	install target/x86_64-unknown-linux-musl/debug/display rootfs/bin
	ln -rs rootfs/bin/schelp rootfs/bin/sh

# Make a copy of the root/
directory:
	rm -rf rootfs
	cp -r root rootfs

build:
	cargo build

run:
	qemu-system-x86_64 -nographic -kernel linux -append "console=ttyS0 init=/init loglevel=7" -initrd rootfs.cpio.gz -m 1024

view:
	qemu-system-x86_64 -vga virtio -kernel linux -append "init=/init loglevel=7" -initrd rootfs.cpio.gz -m 1024

clean:
	rm -f rootfs.cpio.gz
	rm -fr rootfs
	cargo clean
