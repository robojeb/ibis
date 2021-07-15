# Cleanup on PID 1 Please

We left off last episode with a very basic crashy shell, which relied on `init`
respawning it over and over again. 
We also have a (potentially) very crashy `init` program. 
Neither of our programs do any error handling, in this issue we are going to 
look at adding some error handling to `init` and do some general cleanup and 
add code to let us cleanly shutdown. 

Our goals will be the following: 

* Break our code into some modules
* Remove panicking calls like `unwrap()` and properly handle errors
* Clean up our `main()` function
* Add a function for if something goes horribly wrong
* Figure out how to properly terminate the system
* Adding some documentation comments to our functions and data

[Full code for this post here](#)

# Prepare to be Modularized, resistance is futile

Lets create a few module files so we can have a little code separation. 

```Bash
touch boot.rs
touch debug.rs
touch defaults.rs
touch shutdown.rs
```

Then we can bring these modules into the program

```Rust
// init/main.rs: Near the top
mod boot;
mod debug;
mod defaults;
mod shutdown;
```

In the following sections we will start moving code into these modules.

# Cleaning up the boot banner

The first thing that is very likely to crash is the custom `logo.txt` feature we 
added in the first post. 
First lets start by refactoring our boot banner code out to a function in our 
new boot module.

```Rust
// init/boot.rs
/// Print out useful information during boot including a nice logo
pub fn print_boot_banner_info() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);
}
```

Then we can call this function from our main using the relative module path.

```Rust
// init/main.rs
fn main() {
    // ... 
    //let mut logo_file = File::open("/logo.txt").unwrap();
    //let mut buffer = String::new();

    //logo_file.read_to_string(&mut buffer).unwrap();

    //println!("Hello, Ibis!\n{}", buffer);
    boot::print_boot_banner_info();
    // ... 
}
```

This is cleaner, but still has a lot of potential to crash. 
We can handle getting a string to print as the boot banner in a function as well. 
Factoring this out just helps make it cleaner if we want to add more information
to our boot banner later. 

```Rust
// init/boot.rs
pub fn get_boot_banner_logo() -> String {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();
    logo_file.read_to_string(&mut buffer).unwrap();
    buffer
}

pub fn print_boot_banner_info() {
    let logo = get_boot_banner_logo();
    println!("{}", logo);
}
```

Now we can add error handling to our function for getting the logo. 
We will also add a fallback to a built-in logo if we cant get the user defined
one for any reason.
Lets put this logo in `defaults.rs`.

```Rust
// init/defaults.rs
/// The default Ibis logo
const DEFAULT_BANNER_LOGO: &'static str = r#" _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/"#;
```

Then our function can return the default logo if either opening or reading the 
file returns an error. 

```Rust
// init/boot.rs
use crate::defaults;

/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `defaults::DEFAULT_BANNER_LOGO`.
fn get_boot_banner_logo() -> Cow<'static, str> {
    match File::open("/logo.txt") {
        Ok(mut logo_file) => {
            let mut buffer = String::new();
            if let Ok(_) = logo_file.read_to_string(&mut buffer) {
                Cow::Owned(buffer)
            } else {
                Cow::Borrowed(defaults::DEFAULT_BANNER_LOGO)
            }
        }
        Err(_) => Cow::Borrowed(defaults::DEFAULT_BANNER_LOGO),
    }
}
```

One interesting item of note is the use of the `Cow<'static str>` type. 
This type allows us to return either an owned value (as read from the file) or
a borrowed type (the constant built into the binary). 

# Making the Custom Logo Optional

The custom logo feature is a nice to have, but it isn't really critical to the 
operation of our `init` program. 
We are going to set up a Cargo feature to allow users who want this feature
to keep it, but everyone else will just use the built-in function. 

First we have to edit our `Cargo.toml` for the `init` project. 

```Toml
# init/Cargo.toml
# Add the following
[features]
default = [] # No default features
customizable_logo = [] # No dependencies
```

This informs Cargo that we want a configuration property called `customizable_logo` 
that we can pass as a build option. 

We can then use this feature to conditionally compile our function: 

```Rust
// init/boot.rs
#[cfg(feature = "customizable_logo")]
use std::{borrow::Cow, fs::File, io::Read};

/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `DEFAULT_BANNER_LOGO`.
#[cfg(feature = "customizable_logo")]
pub fn get_boot_banner_logo() -> Cow<'static, str> {
    /// *snip*
}
```

We also need to provide an alternate version when the feature isn't enabled. 

```Rust
// init/boot.rs
/// Load the default Ibis logo
#[cfg(not(feature = "customizable_logo"))]
pub fn get_boot_banner_logo() -> &'static str {
    DEFAULT_BANNER_LOGO
}
```

This version doesn't need to return a `Cow` type because it only ever returns the
default string. 
It doesn't matter that the function signatures don't match because each return 
type will successfully typecheck where we use the function in `print_boot_banner_info()`. 

# `make`-ing Cargo features

There are two ways to enable a feature with Cargo. The first is to add the
feature to the dependency list when using a library. This obviously won't work
for our binaries so we can use method two, the `--feature` flag when building. 

Because we are using `make` we are going to have to add some rules to our Makefile
so that Cargo will appropriately build. 

First lets edit our `rust_build` rule.

```Makefile
.PHONY: rust_build
rust_build: 
	cargo build $(CARGO_FLAGS)
```

Then we can add some logic to manipulate our new `CARGO_FLAGS` variable. 

```Makefile
# Should we use the default features for the binaries
RUST_USE_DEFAULT_FEATURES=true

# Should we build in debug or release mode
RUST_DEBUG_BUILD=true

# Add features to enable for each program (using cargo feature syntax)
RUST_FEATURES=

CARGO_FLAGS=--all --target=$(TARGET)

# Turn off default package features
ifeq ($(RUST_USE_DEFAULT_FEATURES), false)
	CARGO_FLAGS+= --no-default-features
endif

ifneq ($(RUST_DEBUG_BUILD), true)
	CARGO_FLAGS+= --release
endif

ifneq ($(RUST_FEATURES),)
	CARGO_FLAGS+= --features "$(RUST_FEATURES)"
endif
```

Now we have a few flags we can tweak to control the Cargo build.
First we can turn on or off the default features enabled with all the binaries. 
Right now this doesn't do much because we have no default features. 
Second we can enable or disable debug build. 
Finally we can add features as we want, for example if we wanted the `customizable_logo`
feature we would add 

```Makefile
RUST_FEATURES+= init/customizable_logo
```

# A Little More Cleanup: Being PID 1, and the default PATH

Another issue we noticed last time was that it is possible to re-execute our
`init` program by calling it from `ibish`. 
As we have it designed right now `init` isn't really intended to do that.
Lets add a quick check at the top of `main()` before we do anything else 
to be sure we are actually PID 1. 

```Rust
// init/main.rs
fn main() {
    // Let's make sure we are PID 1, we're not designed to do anything else.
    if std::process::id() != 1 {
        println!("This process must be run as PID 1 (init)");
        // Exit with an error
        std::process::exit(1);
    }
    // *snip*
}
```

It's also bad practice to use "magic nubmers" like the literal `"/bin"` when
we set the path. 
We can also add a constant to define the default `PATH`. 

```Rust
// init/defaults.rs
/// Set the defaults for the PATH variable we want to set up
const DEFAULT_PATH: &'static str = "/sbin;/bin";

// init/main.rs
// Inside `main()`
//...
    // We need a PATH or `ibish` won't work :(
    std::env::set_var("PATH", defaults::DEFAULT_PATH);
//...
```

# Shutdown and a Five-star crash rating

The last things on our list of cleanup activities is a clean shutdown and 
something to handle when we can't recover from a crash. 

Let us start with a crash handler, in our `debug.rs` file we can do the following.

```Rust
// init/debug.rs

/// Fall into an unrecoverable state and display a helpful message
///
/// This function should be called with a helpful message when the `init`
/// program cannot recover from an error it has encountered.
/// This will spin forever right now.
pub fn unrecoverable_error<M: std::fmt::Display>(msg: M) {
    println!("init has encountered a serious error.\n\t{}\n\nPlease report a bug and reboot your system.", msg);
    loop {}
}
```

This is good enough for now, it lets us tell the user that something is wrong
and won't crash the kernel by terminating `init`. 
Later we can add more debugging features as we need them. 

Next we can tackle shutting down, to do this we are going to bring in our first
dependency, the [`nix`](https://crates.io/crates/nix) crate. `nix` creates safe
bindings to Linux (and other Unix style) system calls and C library functions. 

```Toml
# init/Cargo.toml

[dependencies]
nix = "0.22"
```

We are particularly interested in the function 
[`nix::sys::reboot::reboot()`](https://docs.rs/nix/0.22.0/nix/sys/reboot/fn.reboot.html)
which handles rebooting and shutting down the system. 
We can look up this function's system call reference with the `man` Manpage program, 
We have to be aware to look in the correct Manpage section because there is also
a program called `reboot`, in this case we want the second section "2: Syscalls".

```
man 2 reboot
```

This manual entry has a lot of useful information about shutting down and rebooting 
the system. It gives us two critical pieces of information: 

1. The command we want to issue to shut-down is `RB_POWER_OFF`. 
1. Before we shutdown we need to issue a `sync()` or we can lose data. 

At this time we don't have any persistant storage set-up, but it is a good idea
to follow all the safety rules before we forget and get angry bug reports later. 

Finally there is one last thing we will want to do when we shutdown. Signal to 
every process that they should terminate gracefully if they can. 
Lets put all this together into a new function. 

```Rust
// init/shutdown.rs
use crate::debug::unrecoverable_error;

/// Perform a graceful shutdown of the system
///
/// There are several stages here:
///  1. Terminate all processes in the system
///  2. Sync the filesystem
///  3. Inform the kernel to shutdown and power-off
pub fn on_shutdown_request() {
    println!("Terminating all processes");
    // Setting PID to -1 indicates we want to kill every process we have
    // permission to do so (man 2 kill). In this case it should be everything
    // because we are `init`
    if let Err(_error) = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(-1),
        nix::sys::signal::Signal::SIGTERM,
    ) {
        println!("Failure trying to kill processes during shutdown");
    }

    // Per the documentaiton (`man 2 reboot`) we must issue a `sync` prior
    // to using `RB_POWER_OFF` or else we could lose data.
    // This would make our users very unhappy
    nix::unistd::sync();
    if let Err(_error) = nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_POWER_OFF) {
        unrecoverable_error("Could not initiate shutdown");
    }
}
```

Let's break this down into the three parts in the documentation comment. 
First, we try to tell every process in the system that they should terminate. 
We do this by sending `SIGTERM` to the `-1` pid. From the `man 2 kill` page: 

> If pid equals -1, then sig is sent to every process for which the calling process has permission to send signals, except for process 1 (init), but see below.

This is convenient because as `init` we should have permissions to send this signal
to every other process in the system. 
If for some reason this fails, we print a message for the user before moving on. 

Next we issue a `sync()` command as specified in the documentation for `shutdown()`. 

Finally we issue a `reboot()` system call, asking for the system to power-down.
If this fails we could theoretically just crash, because we were going to 
shut-down anyway, but as a courtesy to the user we are going to use our new
error function to print a helpful error message and hang. 

We can finally tie this all together and give ourselves a clean shutdown. 
For the moment we don't have a way send any signals to our `init` so instead we 
will just initiate shutdown after our shell goes away. 

```Rust
// init/main.rs

fn main() {
    // *snip*
    loop {
        // Spawn one shell and then shutdown
        if let Ok(mut child) = std::process::Command::new("/ibish").spawn() {
            match child.wait() {
                Ok(_) => {} //Nothing to do
                Err(_) => println!("Error waiting for child to terminate"),
            }
            // initiate shutdown.
            shutdown::on_shutdown_request();
        }
    }
}
```

