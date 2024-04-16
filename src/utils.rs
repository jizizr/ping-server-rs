pub mod dns;
pub mod ping;

use std::error::Error;

pub type BoxError = Box<dyn Error + Send + Sync>;
