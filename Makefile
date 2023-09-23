cpio: directory install
	cd root; find | cpio --quiet -o -H newc | gzip > ../rootfs.cpio.gz

directory:
	rm -rf root
	mkdir root

install:
	mv target/debug/init root

run:
	qemu-system-x86_64 -kernel /boot/vmlinuz-linux -initrd rootfs.cpio.gz

clean:
	rm rootfs.cpio.gz
	rm -r target
