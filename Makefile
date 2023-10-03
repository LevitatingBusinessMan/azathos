cpio: directory #install
	cd rootfs; find | cpio --quiet -o -H newc | gzip > ../rootfs.cpio.gz

directory:
	rm -rf rootfs
	mkdir rootfs
	cp -r root/* rootfs

install:
	mv target/debug/init rootfs

run:
	qemu-system-x86_64 -kernel /boot/vmlinuz-linux -append "console=ttyS0" -initrd rootfs.cpio.gz -nographic -m 256

clean:
	rm rootfs.cpio.gz
	rm -r target
