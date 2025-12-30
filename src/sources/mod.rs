pub mod brew;
pub mod curl;
pub mod mise;

use anyhow::Result;

pub trait Source {
    fn install(&self, name: &str) -> Result<()>;
    fn is_installed(&self, name: &str) -> bool;
}
