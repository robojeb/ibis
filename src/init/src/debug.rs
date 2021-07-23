/// Fall into an unrecoverable state and display a helpful message
///
/// This function should be called with a helpful message when the `init`
/// program cannot recover from an error it has encountered.
/// This will spin forever right now.
pub fn unrecoverable_error<M: std::fmt::Display>(msg: M) {
    println!("init has encountered a serious error:\n\t{}\n\nPlease report a bug and reboot your system.", msg);
    loop {}
}
