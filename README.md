# AzathOS
AzathOS (previously called Real Linux From Scratch) is userland for Linux written from scratch.
I started this project to get a better understanding of the kernel.
My goal is to create a usable UNIX environment with my own init, shell and utilities.

All code is written in Rust and limited to just a few common libraries like libc, chrono, clap and serde.

## Building
```SHELL
make
make run
```

# Intramfs
Currently AzathOS works by creating a tiny filesystem in a cpio archive.
This is then used as the initramfs for the kernel supplied with `-initrd` flag in qemu.
The kernel will then decompress my filesystem on top of the initramfs already baked into the kernel.

The kernel creates that initramfs during compilation according to `default_cpio_list`, it creates the /dev/console device and also the /root mountpoint.
Because AzathOS does not actually intend to mount a root disk and doesn't use a `/root` directory you can patch the kernel to not create that directory.
See `no_root.patch`.

## Build into the kernel
It is possible to build the initrd into the kernel itself.
This is done by specifying the filesystem files, `rootfs` in `CONFIG_INITRAMFS_SOURCE`.

You also want to include `default_cpio_list` which will ensure a console device is created.
`CONFIG_INITRAMFS_SOURCE="azathos/rootfs azathos/default_cpio_list"`

If you also want to have some decent size, I recommend turning off all the debug options I configured and set `CONFIG_CC_OPTIMIZE_FOR_SIZE`.
