mod boot;
mod debug;
mod defaults;
mod shutdown;

fn main() {
    // Let's make sure we are PID 1, we're not designed to do anything else.
    if std::process::id() != 1 {
        println!("This process must be run as PID 1 (init)");
        // Exit with an error
        std::process::exit(1);
    }

    boot::print_boot_banner_info();

    // We need a PATH or `ibish` won't work :(
    std::env::set_var("PATH", defaults::DEFAULT_PATH);

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
