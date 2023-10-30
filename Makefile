cpio: install
	cd rootfs; find | cpio --quiet -o -H newc | gzip -1 > ../rootfs.cpio.gz

install: build directory
	cp target/x86_64-unknown-linux-musl/debug/init rootfs

# Make a copy of the root/
directory:
	rm -rf rootfs
	mkdir rootfs
	cp -r root/* rootfs

build:
	cargo build

run:
	qemu-system-x86_64 -kernel /boot/vmlinuz-linux -append "console=ttyS0 init=/init loglevel=7" -initrd rootfs.cpio.gz -nographic -m 1024

clean:
	rm rootfs.cpio.gz
	rm -r target
