# Project Goals

My goal with this project is to implement a suite of Linux userspace programs to 
provide a minimal environment. We are going to jump over implementing `libc` at 
the beginning and start with `init`. 
From there we will build up each component to allow us to have a useable system.

# But first some setup

Before we begin, we have to set-up our development environment. 
First we need to install Rust, the typicall installer can be found [here](https://rustup.rs/). 

Along with Rust we want to install the `x86_64-unknown-linux-musl` target type for Rust. 
This will let us statically link against [musl libc](https://www.musl-libc.org/). 
You can accomplish this with the command: 
```bash
rustup target add x86_64-unknown-linux-musl
```

At time of writing any version of Rust which supports edition 2018 should work. 
I am personally using 1.53 stable. 
If I notice later that the minimum supported rust version has changed I will note
it. 

We are also going to need a few tools: 
1. make
1. cpio
1. qemu (for x86_64)

These will let us build our system, create an initramfs, and run that system respectively. 

# Bulding our Project

To build and run our project we are going to use `make`. 
`make` is a great program which allows tracking dependencies and only building
what is needed. 

I am no expert at `make`, I only have fuzzy memories from 8 years ago while using 
it for a class in college. This `Makefile` will likely be super hacky, but should
be enough to get things done.

Let start by creating a directory for our project and starting a makefile:
```bash
mkdir ibis
cd ibis
touch Makefile
```

Now lets get editing with whatever your favorite editor is. 

# A Kernel of truth

The first thing we are going to need is a Linux Kernel image that will run our
init, manager processes, and handle all the low level hardware stuff that is 
beneath us (at least in an abstraction layer sense). 

It is possible to get a kernel from your host Linux installation, but instead 
lets download and build Linux from source. 
We'll start with Linux 5.13, so lets add the following variables at the top of 
our `Makefile`. 

```Makefile
# We will need a Linux Kernel to run in Qemu so make sure that is downloaded and built here
KERNEL_MAJOR_VERSION=5
KERNEL_MINOR_VERSION=13
KERNEL_VERSION=$(KERNEL_MAJOR_VERSION).$(KERNEL_MINOR_VERSION)
KERNEL_DIRECTORY=linux-$(KERNEL_VERSION)
KERNEL_ARCHIVE=$(KERNEL_DIRECTORY).tar.xz
KERNEL_URL=https://cdn.kernel.org/pub/linux/kernel/v$(KERNEL_MAJOR_VERSION).x/$(KERNEL_ARCHIVE)
```

Next lets tell make how to download and build the kernel. 
We will have it output the compressed kernel image as `vmlinuz` in the project
root. 

```Makefile
#
# Build and download Linux from sources
#

vmlinuz: $(KERNEL_DIRECTORY)
	cd $(KERNEL_DIRECTORY) && make defconfig && make -j`nproc`
	cp $(KERNEL_DIRECTORY)/arch/x86_64/boot/bzImage vmlinuz

# Build a Linux kernel
$(KERNEL_DIRECTORY):
	wget $(KERNEL_URL)
	tar xf $(KERNEL_ARCHIVE)
```

Awesome! Now we can run `make vmlinuz` and get a useful kernel. 
Lets try to get it running, we can add another make command for this: 

```Makefile
# We need the kernel built to run this
run: vmlinuz
    qemu-system-x86_64 -m 2048 -kernel vmlinuz -nographic --append console=ttyS0
```

Sweet, now we just run `make run` and...

```
...
[    2.229807] VFS: Cannot open root device "(null)" or unknown-block(0,0): error -6
[    2.230277] Please append a correct "root=" boot option; here are the available partitions:
[    2.230771] 0b00         1048575 sr0 
[    2.230818]  driver: sr
[    2.231381] Kernel panic - not syncing: VFS: Unable to mount root fs on unknown-block(0,0)
[    2.231999] CPU: 0 PID: 1 Comm: swapper/0 Not tainted 5.13.0 #1
[    2.232379] Hardware name: QEMU Standard PC (i440FX + PIIX, 1996), BIOS ArchLinux 1.14.0-1 04/01/2014
[    2.232968] Call Trace:
[    2.234209]  dump_stack+0x64/0x7c
[    2.234490]  panic+0xf6/0x2b7
[    2.234704]  mount_block_root+0x18c/0x205
[    2.234978]  prepare_namespace+0x136/0x165
[    2.235187]  kernel_init_freeable+0x20b/0x216
[    2.235374]  ? rest_init+0xa4/0xa4
[    2.235527]  kernel_init+0x5/0xfc
[    2.235672]  ret_from_fork+0x22/0x30
[    2.236558] Kernel Offset: 0x3c200000 from 0xffffffff81000000 (relocation range: 0xffffffff80000000-0xffffffffbfffffff)
[    2.237386] ---[ end Kernel panic - not syncing: VFS: Unable to mount root fs on unknown-block(0,0) ]---
```

Hmmm, thats not great, we can quit by issuing `Ctrl-a+X`. 
It makes sense we don't have a filesystem or anything
for the kernel to run. The kernel expects a file `/init`  or `/sbin/init` that it
can run as the first process in the system. 

We should probably build that.

# The world's worst `init` program

Let's get started creating something for the kernel to run, which can in turn
run our other programs so we can do something useful. 

I am going to put all the code in a folder called `src`, we can create a simple
binary framework using `cargo` as follows: 

```bash
mkdir src
cd src
# Don't initialize a git repo here because the whole project is in its own repo
cargo new init --vcs none
cd ..
```

Cargo will populate this with a simple "Hello, World!" program which will work 
for now. 
We are also going to want to set up a Cargo workspace. 
This will let Cargo build all of our various binaries from the project root. 

Lets create a `Cargo.toml` in the project root with the following contents: 

```Toml
# Cargo.toml
[workspace]

members = [
    "src/init"
]
```

Now lets try to build, remembering to specify the target type we want:

```
cargo build --all --target=x86_64-unknown-linux-musl
```

We can even run this newly built executable. There isn't anything special about
it other than being linked with Musl libc: 

```
$ ./target/x86_64-unknown-linux-musl/debug/init
Hello, world!
```

Lets add a make rule to build all the binaries and a variable just in case we 
want to change the target type. 
Note, typically Make expects you to list dependencies and list the output of 
your build operations. 
In this case Cargo will handle dependencies for us so we will make a phony rule
which is always executed: 

```Makefile
# This is defining our Rust target, we will use the musl C library 
TARGET=x86_64-unknown-linux-musl

.PHONY: rust_build
rust_build: 
	cargo build --all --target=$(TARGET)
```

Next we will use the `cpio` program to create an initramfs image. 
We will have to copy our new init to the project root for the time being: 

```bash
cp ./target/x86_64-unknown-linux-musl/debug/init .
echo init | cpio -o --format=newc > initramfs
```

We then modify our old `run` command:

```diff
-run: vmlinuz
-    qemu-system-x86_64 -m 2048 -kernel vmlinuz -nographic --append console=ttyS0
+run: vmlinuz
+    qemu-system-x86_64 -m 2048 -kernel vmlinuz -initrd initramfs -nographic --append console=ttyS0
```

Finally `make run` Tada!... oh:

```
[    1.634345] Freeing unused kernel image (text/rodata gap) memory: 2032K
[    1.634854] Freeing unused kernel image (rodata/data gap) memory: 584K
[    1.635134] Run /init as init process
Hello, World!
[    1.661243] Kernel panic - not syncing: Attempted to kill init! exitcode=0x00000000
[    1.661536] CPU: 0 PID: 1 Comm: init Not tainted 5.13.0 #1
[    1.661682] Hardware name: QEMU Standard PC (i440FX + PIIX, 1996), BIOS ArchLinux 1.14.0-1 04/01/2014
[    1.661926] Call Trace:
[    1.662566]  dump_stack+0x64/0x7c
[    1.662691]  panic+0xf6/0x2b7
[    1.662744]  do_exit.cold+0xd7/0xe3
[    1.662811]  do_group_exit+0x2e/0x90
[    1.662886]  __x64_sys_exit_group+0xf/0x10
[    1.662975]  do_syscall_64+0x40/0x80
[    1.663043]  entry_SYSCALL_64_after_hwframe+0x44/0xae
[    1.663258] RIP: 0033:0x7fb097a5c49c
[    1.663458] Code: eb ef 48 8b 76 28 e9 a2 03 00 00 64 48 8b 04 25 00 00 00 00 48 8b b0 b0 00 00 00 e9 af ff ff ff 48 63 ff b8 e7 00 00 04
[    1.663898] RSP: 002b:00007ffe7331fe08 EFLAGS: 00000246 ORIG_RAX: 00000000000000e7
[    1.664048] RAX: ffffffffffffffda RBX: 00007fb097a23a20 RCX: 00007fb097a5c49c
[    1.664173] RDX: 0000000000000000 RSI: 0000000000000000 RDI: 0000000000000000
[    1.664292] RBP: 0000000000000001 R08: 0000555556828110 R09: 0000000000000001
[    1.664416] R10: 0000000000000000 R11: 0000000000000246 R12: 00007ffe7331fe68
[    1.664534] R13: 00007ffe7331fe78 R14: 0000000000000000 R15: 0000000000000000
[    1.665034] Kernel Offset: 0xe000000 from 0xffffffff81000000 (relocation range: 0xffffffff80000000-0xffffffffbfffffff)
[    1.665442] ---[ end Kernel panic - not syncing: Attempted to kill init! exitcode=0x00000000 ]---
```

Turns out Linux doesn't like it when `init` goes away and will kernel panic. 
That is easy enough to fix, lets go modify `src/init/src/main.rs`: 

```rust
fn main() {
    println!("Hello, Ibis!");
    loop {}
}
```

We add a `loop{}` construct which tells Rust to infinitely spin. 
I also modified the print to feel a little more personal. 
We just need to rebuild the initramfs:

```shell
make rust_build
cp ./target/x86_64-unknown-linux-musl/debug/init .
echo init | cpio -o --format=newc > initramfs
make run
```

Fingers crossed:

```
[    1.637313] Freeing unused kernel image (text/rodata gap) memory: 2032K
[    1.637868] Freeing unused kernel image (rodata/data gap) memory: 584K
[    1.638215] Run /init as init process
Hello, Ibis!
[    2.085277] tsc: Refined TSC clocksource calibration: 3599.936 MHz
[    2.085504] clocksource: tsc: mask: 0xffffffffffffffff max_cycles: 0x33e416808e6, max_idle_ns: 440795261485 ns
[    2.085729] clocksource: Switched to clocksource tsc
[    2.166215] input: ImExPS/2 Generic Explorer Mouse as /devices/platform/i8042/serio1/input/input3
```

Et, voila! The worlds worst `init` program. It really do anything useful but its 
ours. &lt;/warm fuzzies&gt;

# Lets do a little more, not much though

I think we can all admit, even with as proud as we are of our little `init` we
would like it to do a little more. Also, it would be nice if make could build 
the initramfs for us.
Lets tackle both of these issues. 

Please be kind, this section is going to start showing just how inept I am at
crafting make rules. 

To make our program more interesting I have decided that we need a sweet ASCII 
logo, but also it should be customizable and read from a file. (This is all
super contrived but were just trying to take small steps)

Lets modify the `main.rs` of our `init` program: 

```rust
use std::{
    io::Read,
    fs::File,
};

fn main() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);
    loop {}
}
```

Were just ignoring error handling at the moment, but this code probably isn't 
long for the world anyway so it should be okay (famous last words I know).

This is great, but if we try to rebuild the initramfs as we did before and run it
we will crash. Its worse because this time its our fault! Not just the kernel being
picky about what we are doing. 

Lets start by setting up a directory structure with the template for our initramfs. 
I'll call this directory `rfs_template` and it will contain any folders and files
we want, like logos and configuraiton files. 

```shell
mkdir rfs_template
touch rfs_template/logo.txt
```

Lets fill in that logo file with this sweet ACII art (Generated 
[here](https://patorjk.com/software/taag/#p=display&f=Doom&t=Ibis)).

```
 _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/
```

Now to teach make how to handle our files. 
I am going to dump what I have here and then explain it afterwards: 

```Makefile
#
# Build an initramfs for QEMU
#
initramfs: | rfs build_initramfs

#FIXME: This always rebuilds the initramfs even if the RFS didn't change
.PHONY: build_initramfs
build_initramfs: $(wildcard rfs/**/*)
	cd rfs && find . | cpio -o --format=newc > ../initramfs

.PHONY: rfs
rfs: | rust_build rfs_update

rfs_update: $(wildcard rfs_template/*) $(wildcard target/$(TARGET)/debug/**/*)
	mkdir -p rfs
	cp -r rfs_template/* rfs/
	cp ./target/$(TARGET)/debug/init ./rfs/
# Keep track of when we last updated the RFS so that we can build properly
	touch rfs_update
```

The goal here is for make to create the `initramfs` target file.
This is going to take two steps: First, we have to create a rfs directory tree 
with our init and config files, second we have to run `cpio` to turn this into
an initramfs image. 

To build the `rfs` first we need to invoke `cargo` via the `rust_build` rule to 
see if any of our binaries changed and then we need to update the contents of our
`rfs` directory tree. 

We use the `$(wildcard ...)` command to depend on every file in both our template
directory and our binary output directory. We also touch a file called `rfs_update`
which keeps track of how recently we updated the rfs tree.

I know this has issues, but it works enough for me. 

With these rules we can now execute `make run` and everything should update. 
Magical!

```
[    1.660751] Freeing unused kernel image (text/rodata gap) memory: 2032K
[    1.661321] Freeing unused kernel image (rodata/data gap) memory: 584K
[    1.661689] Run /init as init process
Hello, Ibis!
 _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/
[    2.084826] tsc: Refined TSC clocksource calibration: 3599.938 MHz
[    2.085086] clocksource: tsc: mask: 0xffffffffffffffff max_cycles: 0x33e41921b99, max_idle_ns: 440795389779 ns
[    2.085355] clocksource: Switched to clocksource tsc
[    2.189615] input: ImExPS/2 Generic Explorer Mouse as /devices/platform/i8042/serio1/input/input3
```

# Final steps, clean up our mess

There are a few things left we can add to our Makefile: 

```Makefile

.PHONY: all
all: vmlinuz initramfs

# Clean only the rust dependencies
.PHONY: clean
clean: 
	cargo clean
	rm -rf ./rfs ./rfs_update initramfs

# Clean rust and Linux dependencies
.PHONY: cleaner
cleaner: clean
	rm -rf $(KERNEL_ARCHIVE) $(KERNEL_DIRECTORY) vmlinuz
```

The `all` command we want to place at the top, this will be our default command
so when we execute `make` without an argument. 

`clean` is a common command in makefiles, in this case we are only going to remove
things that build quickly. I feel its too costly to remove the Linux source every 
time I want to do a clean build. 

If I really want to delete everythign I put in the `cleaner` command. 

# Where to go from here?

Let's take stock of what we have. 
We have a basic framework to build a Linux kernel and super barebones initramfs. 
We have the world's least useful `init` program. 

Thats nice but we really want to be able to *do* something with our system. 
To accomplish that we are going to need some tools, of most importance we need
a shell so we can interact with the system. We also will need some basic 
utilities like `ls` and `cat`. 

Next issue we will put together a battle plan and start with the utility programs. 


