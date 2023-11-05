# Real Linux From Scratch
RLFS is userland for Linux written from scratch.
I started this project to get a better understanding of the Linux kernel itself.
My goal is to create a usable UNIX environment with my own init, shell and utilities.

All code is written in Rust limited to just a few common libraries like libc, chrono, clap and serde.

## Building
```SHELL
make
make run
```

## Build into the kernel
It is possible to build the initrd into the kernel itself.
This is done by specifying the filesystem files, `rootfs` in `CONFIG_INITRAMFS_SOURCE`.

You also want to include `default_cpio_list` which will ensure a console device is created.
`CONFIG_INITRAMFS_SOURCE="rlfs/rootfs rlfs/default_cpio_list"`

If you also want to have some decent size, I recommend turning off all the debug options I configured and set `CONFIG_CC_OPTIMIZE_FOR_SIZE`.
