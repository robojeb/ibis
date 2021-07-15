use crate::defaults;

// Supress unused include warnings when the feature is disabled
#[cfg(feature = "customizable_logo")]
use std::{borrow::Cow, fs::File, io::Read};

/// Try to load the logo provided by a user from `/logo.txt`
///
/// If this file cannot be found or read, this will provide a default logo
/// from `defaults::DEFAULT_BANNER_LOGO`.
#[cfg(feature = "customizable_logo")]
pub fn get_boot_banner_logo() -> Cow<'static, str> {
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

/// Load the default Ibis logo
#[cfg(not(feature = "customizable_logo"))]
pub fn get_boot_banner_logo() -> &'static str {
    defaults::DEFAULT_BANNER_LOGO
}

/// Print out useful information during boot including a nice logo
pub fn print_boot_banner_info() {
    let logo = get_boot_banner_logo();
    println!("{}", logo);
}
