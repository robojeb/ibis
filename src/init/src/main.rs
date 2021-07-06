use std::borrow::Cow;
#[cfg(feature = "customizable_logo")]
use std::{fs::File, io::Read};

use nix::sys::signal::{sigprocmask, SigSet, SigmaskHow};

const DEFAULT_BANNER_LOGO: &'static str = r#" _____ _     _     
|_   _| |   (_)    
  | | | |__  _ ___ 
  | | | '_ \| / __|
 _| |_| |_) | \__ \
 \___/|_.__/|_|___/"#;

const DEFAULT_PATH: &'static str = "/sbin;/bin";

fn unrecoverable_error<M: std::fmt::Display>(msg: M) {
    println!("init has encountered a serious error:\n\t{}\n\nPlease report a bug and reboot your system.", msg);
    #[cfg(feature = "verbose_debug")]
    debug_dump_env();
    loop {}
}

/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `DEFAULT_BANNER_LOGO`.
#[cfg(feature = "customizable_logo")]
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

#[cfg(not(feature = "customizable_logo"))]
fn get_boot_banner_logo() -> Cow<'static, str> {
    Cow::Borrowed(DEFAULT_BANNER_LOGO)
}

#[cfg(feature = "verbose_debug")]
fn debug_dump_env() {
    for (var, value) in std::env::vars() {
        println!("{}: {}", var, value);
    }
}

fn on_shutdown_request() {
    println!("Terminating all processes");
    // Setting PID to -1 indicates we want to kill every process we have
    // permission to do so (man 3 kill). In this case it should be everything
    // because we are `init`
    if let Err(_error) = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(-1),
        nix::sys::signal::Signal::SIGTERM,
    ) {
        println!("Failure trying to kill processes during shutdown");
    }

    // Per the documentaiton (`man 3 reboot`) we must issue a `sync` prior
    // to using `RB_POWER_OFF` or else we could lose data.
    // This would make our users very unhappy
    nix::unistd::sync();
    if let Err(_error) = nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_POWER_OFF) {
        unrecoverable_error("Could not initiate shutdown");
    }
}

fn main() {
    // Let's make sure we are PID 1, we're not designed to do anything else.
    if std::process::id() != 1 {
        println!("This process must be run as PID 1 (init)");
        std::process::exit(1);
    }

    // Before we get too far, we should disable signals and other items like
    // Ctrl-Alt-Delete to reboot. Later we can reenable things we want to handle
    // as we get that set up properly.
    if let Err(_error) = nix::sys::reboot::set_cad_enabled(false) {
        unrecoverable_error("Could not disable Ctrl-Alt-Delete");
    }

    let signal_set = SigSet::all();
    if let Err(_error) = sigprocmask(SigmaskHow::SIG_SETMASK, Some(&signal_set), None) {
        unrecoverable_error("Could not disable signals");
    }

    // Get and print a logo indicating that we are booting
    let logo = get_boot_banner_logo();
    println!("{}", logo);
    println!("\tv{}", env!("CARGO_PKG_VERSION"));

    // We need a PATH or `ibish` won't work :(
    std::env::set_var("PATH", DEFAULT_PATH);

    loop {
        // Infinitely respawn shells
        let mut child = std::process::Command::new("/ibish").spawn().unwrap();
        child.wait().unwrap();
        on_shutdown_request();
    }
}
