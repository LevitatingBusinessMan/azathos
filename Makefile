cpio: install
	#cd rootfs; find | cpio --quiet -H newc -o | lz4 -f - ../rootfs.cpio.lz4
	cd rootfs; find | cpio --quiet -H newc -o | gzip -1 > ../rootfs.cpio.gz

install: build directory
	install target/x86_64-unknown-linux-musl/debug/init rootfs
	install target/x86_64-unknown-linux-musl/debug/schelp rootfs/bin
	ln -rs rootfs/bin/schelp rootfs/bin/sh

# Make a copy of the root/
directory:
	rm -rf rootfs
	cp -r root rootfs

build:
	cargo build

run:
	qemu-system-x86_64 -kernel linux -append "console=ttyS0 init=/init loglevel=7" -initrd rootfs.cpio.gz -nographic -m 1024

clean:
	rm -f rootfs.cpio.gz
	rm -fr rootfs
	cargo clean
