cpio: directory init
	cd root; find | cpio --quiet -o -H newc | gzip > ../rootfs.cpio.gz

directory:
	rm -rf root
	mkdir root

init:
	gcc init.c -static -o root/init

run:
	qemu-system-x86_64 -kernel /boot/vmlinuz-linux -initrd rootfs.cpio.gz

clean:
	rm rootfs.cpio
