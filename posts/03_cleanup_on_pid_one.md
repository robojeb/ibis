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
* Figure out how to properly terminate the system

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
// main.rs: Near the top
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
new boot module: 

```Rust
// boot.rs
pub fn print_boot_banner_info() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);
}

// main.rs
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

This is still cleaner, but still has a lot of potential to crash. 
We can handle getting a string to print as the boot banner in a function as well. 
Factoring this out just helps make it cleaner if we want to add more information
to our boot banner later. 

```Rust
fn get_boot_banner_logo() -> String {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();
    logo_file.read_to_string(&mut buffer).unwrap();
    buffer
}

fn print_boot_banner_info() {
    let logo = get_boot_banner_logo();
    println!("{}", logo);
}
```

Now we can add error handling to our function for getting the logo. 
We will also add a fallback to a built-in logo if we cant get the user defined
one for any reason.

```Rust
/// The default Ibis logo
const DEFAULT_BANNER_LOGO: &'static str = r#" _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/"#;

/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `DEFAULT_BANNER_LOGO`.
fn get_boot_banner_logo() -> Cow<'static, str> {
    match File::open("/logo.txt") {
        Ok(mut logo_file) => {
            let mut buffer = String::new();
            if let Ok(_) = logo_file.read_to_string(&mut buffer) {
                Cow::Owned(buffer)
            } else {
                Cow::Borrowed(DEFAULT_BANNER_LOGO)
            }
        }
        Err(_) => Cow::Borrowed(DEFAULT_BANNER_LOGO),
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
# Add the following
[features]
default = [] # No default features
customizable_logo = [] # No dependencies
```

This informs Cargo that we want a configuration property called `customizable_logo` 
that we can pass as a build option. 

We can then use this feature to conditionally compile our function: 

```Rust
/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `DEFAULT_BANNER_LOGO`.
#[cfg(feature = "customizable_logo")]
fn get_boot_banner_logo() -> Cow<'static, str> {
    /// *snip*
}
```

We also need to provide an alternate version when the feature isn't enabled. 

```Rust
/// Load the default Ibis logo
#[cfg(not(feature = "customizable_logo"))]
fn get_boot_banner_logo() -> &'static str {
    DEFAULT_BANNER_LOGO
}
```

This version doesn't need to return a `Cow` type because it only ever returns the
default string. 
It doesn't matter that the function signatures don't match because each return 
type will successfully typecheck where we use the function in `print_boot_banner_info()`. 

# `make`-ing Cargo features

-TODO-

# A Little More Cleanup: Being PID 1, and the default PATH

Another issue we noticed last time was that it is possible to re-execute our
`init` program by calling it from `ibish`. 
As we have it designed right now `init` isn't really intended to do that.
Lets add a quick check at the top of `main()` before we do anything else 
to be sure we are actually PID 1. 

```Rust
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
/// Set the defaults for the PATH variable we want to set up
const DEFAULT_PATH: &'static str = "/sbin;/bin";

// Inside `main()`
//...
    // We need a PATH or `ibish` won't work :(
    std::env::set_var("PATH", DEFAULT_PATH);
//...
```

