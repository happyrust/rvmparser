pub mod tessellator;
pub mod obj;
pub mod json;
pub mod gltf;
pub mod rev;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Export error: {0}")]
    Other(String),
}
