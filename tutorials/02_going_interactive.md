# Going Interactive

Last time we got an `init` program which doesn't really do much of anything. 
To really start making our project useful we are going to have to start building
some core programs. 
One of the most important is the shell, this will let us interactively call other 
programs as we create them. 

Lets start by creating a project for our shell, we will call it `ibish` because it is
the "Ibis shell".  

```Bash
cd src
cargo new ibish --vcs none --bin
```

Cargo will complain to us that things might not build correctly because this 
program isn't part of the workspace. To fix this we will modify our `Cargo.toml` in the root of our project. 

```diff
[workspace]

+members = ["src/*"]
-members = ["src/ibis"]
```

Using a wildcard will allow all of our other programs to immediately become part of the Cargo workspace. 

# Calcium Carbonate, the basis of the shell

At its core (at least in interactive mode) a shell is just a REPL (Read Evaluate Print Loop). Lets start with a `stdin` only version which prints a simple prompt, 
reads a line of text, and echo's that text back. 
I will show the full text below and then explain each part. 

```Rust
// (1)
use std::io::{BufRead, Write};

// (2)
const PROMPT: &'static str = "> ";

fn main() {
    // (3)
    // Lock stdout because we are the only thread
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    // Lock stdin because we are the only thread
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    // (4)
    let mut line_buf = String::with_capacity(256);
    loop {
        // (5)
        stdout.write_all(PROMPT.as_bytes()).unwrap();
        stdout.flush().unwrap();
        // (6)
        stdin.read_line(&mut line_buf).unwrap();
        // (7)
        stdout.write_all(line_buf.as_bytes()).unwrap();

        // (8)
        // Clear the line as `read_line` will just continue appending to our line buffer
        line_buf.clear();
    }
}
```

1. First we bring in some traits from the standard library. In this case we need to 
be able to read into a buffer, and write data back out to the console. 
2. Here we will set up our prompt as a constant string. 
3. In Rust there is an implicit lock on `stdin` and `stdout` to prevent things printed from multiple threads mangling each other. Right now we are the only thread so we should just lock both files so we don't have to incur the locking penalty every time we go to write. 
4. We create a string to read into, and force it to have an initial capacity of something small and reasonable. This will reduce the chances of use reallocating a lot when reading from input. 
5. Now we write out the prompt, we have to make sure to flush `stdout` to make sure that the prompt appears. Otherwise the prompt will only appear when a newline is written. 
6. We block here to read input from `stdin`, this will be placed into our `line_buf` buffer. 
7. Echo the line back out
8. Its important to clear our buffer before the next line, otherwise when we echo again both lines will appear!

We can easily test this on our local machine: 

```
$ cargo run --bin ibish
   Compiling ibish v0.1.0 (/home/jbrooks/repos/ibis/src/ibish)
    Finished dev [unoptimized + debuginfo] target(s) in 0.28s
     Running `target/debug/ibish`
> test
test
>
```

We can't exit gracefully so we just have to `Ctrl+c`. 

Before we try hooking it into our project lets add some basic actions. 
We'll replace the line which echos with some really basic logic: 

```Rust 
        // Remove
        // stdout.write_all(line_buf.as_bytes()).unwrap();

        // (1) Parse the line
        let parsed_line = parse_line(&line_buf);

        // (2) Try to find some keywords which the shell will interpret directly
        match parsed_line.as_slice() {
            // Leave the shell, ignore any other arguments
            ["exit", ..] => std::process::exit(0),
            // (3) Print that everything else is unknown
            _ => {
                println!("Unknown input: {}", line_buf);
            }
        }
```

1. First we need to parse the input line. We haven't written anything yet, but let us assume it has the signature `fn parse_line<'a>(line_buf: &'a str) -> Vec<&'a str>`. In Rust the `<'a>` and `&'a str` are lifetime annotations. 
We are telling the compiler that the output `&str` slices cannot outlive the input `line_buf`. This allows us to safely share the backing memory from the input buffer with our parsed line. 

1. Here we are going to look at the pieces we parsed in our non-existant function. 
Rust's `match` statement is very powerful and allows us to search for patterns. 
In this case we are providing the pattern `["exit", ..]` which means to match against any slice like object with a string that matches "exit" and any other content which we don't care about. In this case we will just terminate the process
gracefully. 

# Pretty Pathetic Parsing, is practically provided by prelude

As fun as pretending is, we do eventually have to implemente the `parse_line` function. 
In a typical shell implementation parsing would handle things like quoted strings, variable substitution, and glob expansion. Right now we are going to do the simplest possible thing and just split on whitespace. 
Fortunately this is provided by the Rust implementation of `str` with the `.split_ascii_whitespace()` function. 

Lets create the line parsing function: 

```Rust
fn parse_line<'a>(line_buf: &'a str) -> Vec<&'a str> {
    //TODO: Parsing with escapes and quotes and other shell things
    line_buf.split_ascii_whitespace().collect()
}
```

Now lets test again: 

```
$ cargo run --bin ibish
   Compiling ibish v0.1.0 (/home/jbrooks/repos/ibis/src/ibish)
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
     Running `target/debug/ibish`
> test
Unknown input: test

> exit
```

Looks good! We have the world's least useful shell. 
Lets integrate it into our project. 

# `init` spawns Ibis Shells at the sea shore

Before we can make our `init` spawn the shell we need to add it to our rfs. 
Lets modify the Makefile rule for `rfs_update`:

```Makefile
rfs_update: $(wildcard rfs_template/*) $(wildcard target/$(TARGET)/debug/**/*)
	mkdir -p rfs
	cp -r rfs_template/* rfs/
	cp ./target/$(TARGET)/debug/init ./rfs/
# add this vvvv
	cp ./target/$(TARGET)/debug/ibish ./rfs/
# Keep track of when we last updated the RFS so that we can build properly
	touch rfs_update
```

We'll organize our `rfs` later, but this will make sure that we have our `ibish` 
executable in our initramfs. 

Next lets update `init` to spawn `ibish` as a child process: 

```Rust
// Update the inifinite loop
    loop {
        // Infinitely respawn shells
        let mut child = std::process::Command::new("/ibish").spawn().unwrap();
        child.wait().unwrap();
    }
}
```
Now its test time:

```
make run
...
[    2.039674] Run /init as init process
Hello, Ibis!
 _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/
> [    2.304150] tsc: Refined TSC clocksource calibration: 2711.343 MHz
[    2.306059] clocksource: tsc: mask: 0xffffffffffffffff max_cycles: 0x271519e2cca, max_idle_ns: 440795300194 ns
[    2.308814] clocksource: Switched to clocksource tsc
[    2.533415] input: ImExPS/2 Generic Explorer Mouse as /devices/platform/i8042/serio1/input/input3

Unknown input: 

> test
Unknown input: test

> foo
Unknown input: foo

> exit
[   31.165741] ibish (66) used greatest stack depth: 14784 bytes left
> 
```

Looks like it is working! `init` respawns the shell after it exits. 

# Doing something useful 

Right now our shell is fine, but its empty. 
Like a shell on the beach, its pretty, its ours, but at the end of the day its
useless. 
We would like to set it up with a nice little hermit crab who can run around
and push buttons for us and be cute and... perhaps this analogy got away from me a bit. 
Suffice to say our goal right now is to make the shell able to actually call 
other programs. 

Right now, if we don't recognize a command as a built-in command we just print
a little message and move on. 
Instead we should try to execute a program!
Were going to assume at this point that the only thing anyone types into the 
command line is going to be a program name (or path to a program) and a series
of arguments. We won't be handling any shell extras like pipes, variables, or 
globs. 

To do this we can modify our `match` statement: 

```Rust
    // Replace this
    //_ => {
    //    println!("Unknown input: {}", line_buf);
    //}
    &[] => {} // Empty line on input, don't do anything
    _ => {
        // Line isn't empty and isn't a keyword, try to resolve the items
        // Its safe to get this item (eg no panic) because we didn't match 
        // the empty slice pattern, so there is at least one item.
        let path_or_name = parsed_line[0];
        let args = &parsed_line[1..];

        let mut child_process = std::process::Command::new(path_or_name)
            .args(args.iter())
            .spawn()
            .unwrap();
        child_process.wait().unwrap();
    }
```

The first thing to note is that we have to take care if our `parsed_line` is empty. 
If this is the case it would be dangerous to try to index the vector. 

The second thing to note is that we aren't doing any path resolution at this time. 
We don't support aliases, or variables, or anything really fancy so we can let
Rust's `std::process::Command` handle resolving paths for us. 

Lets try this out on our host machine: 

```
$ cargo run --bin ibish
   Compiling ibish v0.1.0 (/home/jbrooks/repos/ibis/src/ibish)
    Finished dev [unoptimized + debuginfo] target(s) in 0.64s
     Running `target/debug/ibish`
> grep
Usage: grep [OPTION]... PATTERNS [FILE]...
Try 'grep --help' for more information.
> /usr/bin/ls
Cargo.lock  Cargo.toml	initramfs  LICENSE.md  linux-5.13  linux-5.13.tar.xz  Makefile	README.md  rfs	rfs_template  rfs_update  src  target  vmlinuz
>
> exit
```

Woo hoo! It looks like both names and paths work for us. 
Lets try it in our system!

```
make run
...
> grep
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }', src/ibish/src/main.rs:43:22
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[    4.628580] ibish (66) used greatest stack depth: 14784 bytes left
```

Okay that crash makes sense, we don't have a `grep` implementation yet, 
fortunately our `init` respawns our shell when it crashes (we should probably
do something about that, and we will in the next issue).
Lets try calling our `inibsh` directly, we only have two binaries to call so lets
go with that. 

```
> ibish
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }', src/ibish/src/main.rs:43:22
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Hmmm... that's weird, we definitely have an `ibish` binary? 
Lets try an absolute path: 

```
> /ibish
> exit
> exit
[  253.199366] ibish (70) used greatest stack depth: 14536 bytes left
> 
```

That looks like it worked! we had two nested shells and exited each in turn. 
But why didn't it work with just the name? On our host machine we can just 
write `grep`, we don't have to say `/usr/bin/grep`. 

Remember when I said "we can let Rust's `std::process::Command` handle resolving paths for us. ". This mechanism relies on having a `PATH` variable in the first 
place. 
Lets quickly modify our `init` to print out what our environment looks like. 
At the top of `main()` lets add: 

```Rust
for (var, value) in std::env::vars() {
    println!("{}: {}", var, value);
}
```

Then run: 
```
make run
...
[    2.044701] Freeing unused kernel image (rodata/data gap) memory: 552K
[    2.045370] Run /init as init process
HOME: /
TERM: linux
Hello, Ibis!
...
```
Well there's your problem! No `PATH`!

We can set up `PATH` pretty easily, above environment print loop lets add
the following: 

```Rust
std::env::set_var("PATH", "/");
```
We can try running this now: 

```
make run
...
[    1.968235] Run /init as init process
HOME: /
TERM: linux
PATH: /
Hello, Ibis!
...
> ibish
> 
```

Success! Our shell is properly resolving a path (now that we have `PATH`). 
We can even re-exec our `init` if we want: 

```
> init
HOME: /
TERM: linux
PATH: /
Hello, Ibis!
 _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/
> 
```

Though, this probably isn't super useful, and we won't want to allow this in 
the future. 

# Summary

Its starting to look like our system might be going somewhere. 
We have a very basic shell which can execute other programs (though we have 
none of those for now). We also have fixed our `init` so that it sets up
the `PATH` variable properly to allow us to look up programs. 

In the next installment we will be doing some cleanup and error handling 
(our shell is very crashy). 
From there we will start building some other core utilities and upgrade both our
`init` and `ibish` shell. 

Stay tuned, same Bat-time, same Bat-channel. 
