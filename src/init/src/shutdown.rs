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
