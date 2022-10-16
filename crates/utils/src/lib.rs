pub mod error;
pub mod passhash;
pub mod time;
pub mod rate_limit;
pub mod settings;
pub mod utils;
pub mod version;

pub use error::PorplError;
pub use passhash::hash_password;
pub use time::time;

use std::{fmt, time::Duration};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct IpAddr(pub String);


impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);


#[macro_export]
macro_rules! location_info {
  () => {
    format!(
      "None value at {}:{}, column {}",
      file!(),
      line!(),
      column!()
    )
  };
}