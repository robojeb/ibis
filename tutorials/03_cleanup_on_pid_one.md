# Cleanup on PID 1 Please

We left off last episode with a very basic crashy shell, which relied on `init`
respawning it over and over again. 
We also have a (potentially) very crashy `init` program. 
Neither of our programs do any error handling, in this issue we are going to 
look at adding some error handling to `init` and do some general cleanup and 
add code to let us cleanly shutdown. 

Our goals will be the following: 

* Remove panicking calls like `unwrap()` and properly handle errors
* Clean up our `main()` function
* Figure out how to properly terminate the system

This post will be a little heavier than previous posts on Rust specific topics 
because of the heavy focus on cleaning up our error handling.
Future posts will (probably) have less of this and just focus on the design
decisions.

# Cleaning up the boot banner

One thing that is very likely to crash is the custom `logo.txt` feature we 
added in the first post. 
First lets start by refactoring our boot banner code out to a function: 

```Rust
fn print_boot_banner_info() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);
}

fn main() {
    // ... 
    //let mut logo_file = File::open("/logo.txt").unwrap();
    //let mut buffer = String::new();

    //logo_file.read_to_string(&mut buffer).unwrap();

    //println!("Hello, Ibis!\n{}", buffer);
    print_boot_banner_info();
    // ... 
}
```

This is still cleaner, but still has a lot of potential to crash. 
We can handle getting a string to print as the boot banner in a function as well. 

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

For error handling, instead of calling `unwrap()` we wan to use a `match` or an `if let`
statement. 

In Rust, `match` and `if let` allow you to do pattern matching on various constructs like `enums`. 
Because the `Result` type (which is returned by fallible functions in Rust) is an `enum` we
can use these statements to perform operations based on what varients we get either `Err` 
or `Ok` depending on if the operation succeeded or not. 

The first thing we need to check is opening the file for the custom logo: 

```Rust
fn get_boot_banner_logo() -> String {
    match File::open("/logo.txt") {
        Ok(mut logo_file) => {
            let mut buffer = String::new();
            logo_file.read_to_string(&mut buffer).unwrap();
            buffer
        }
        Err(_) => {} // Hmmm....
    }
}
```

If we try to build this Rust will complain: 

```
error[E0308]: mismatched types
  --> src/init/src/main.rs:41:19
   |
41 |         Err(_) => {}
   |                   ^^ expected struct `String`, found `()`
```

But we also cannot ignore the `Err(_)` case, Rust forces us to handle all cases.

```
error[E0004]: non-exhaustive patterns: `Err(_)` not covered
   --> src/init/src/main.rs:30:11
    |
30  |     match File::open("/logo.txt") {
    |           ^^^^^^^^^^^^^^^^^^^^^^^ pattern `Err(_)` not covered
    | 
   ::: /home/jeb/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:250:5
    |
250 |     Err(#[stable(feature = "rust1", since = "1.0.0")] E),
    |     --- not covered
    |
    = help: ensure that all possible cases are being handled, possibly by adding wildcards or more match arms
    = note: the matched value is of type `Result<std::fs::File, std::io::Error>`
```

We are going to have to return something from this `Err` branch, so lets set up
a default logo at the top of our file. 

```Rust
/// The default Ibis logo
const DEFAULT_BANNER_LOGO: &'static str = r#" _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/"#;
```

Then in the `Err(_)` case we can just return that right‽

```Rust
fn get_boot_banner_logo() -> String {
    match File::open("/logo.txt") {
        Ok(mut logo_file) => {
            let mut buffer = String::new();
            logo_file.read_to_string(&mut buffer).unwrap();
            buffer
        }
        Err(_) => DEFAULT_BANNER_LOGO,
    }
}
```

And build

```
error[E0308]: `match` arms have incompatible types
  --> src/init/src/main.rs:41:19
   |
30 | /     match File::open("/logo.txt") {
31 | |         Ok(mut logo_file) => {
32 | |             let mut buffer = String::new();
33 | |             logo_file.read_to_string(&mut buffer).unwrap();
34 | |             buffer
   | |             ------ this is found to be of type `String`
...  |
41 | |         Err(_) => DEFAULT_BANNER_LOGO,
   | |                   ^^^^^^^^^^^^^^^^^^^
   | |                   |
   | |                   expected struct `String`, found `&str`
   | |                   help: try using a conversion method: `DEFAULT_BANNER_LOGO.to_string()`
42 | |     }
   | |_____- `match` arms have incompatible types
```

Not good. 
There are two solutions to this in Rust: 

1. We can turn the `&str` reference into a `String`. But this will involve an allocation. 
1. Or, we can use the `Cow` type. 

`Cow` (Copy-On-Write) is an `enum` type like `Result`, but instead it lets you 
choose between a borrowed (`&str`) or owned (`String`) type. 

Lets change our function around to use `Cow`, and also apply the same error 
handling magic to reading the data into the buffer.
We can also add a documentation comment (`///`) for extra cleanliness points. 

```Rust
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

We can then use this to conditionally compile our function: 

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

# Making Cargo features



# Checking that we are `init`

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
