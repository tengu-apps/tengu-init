//! Output renderers for installation manifests

mod bash;

pub use bash::BashRenderer;

use crate::Manifest;

/// A renderer that can convert a manifest to some output format
pub trait Renderer {
    /// Output type
    type Output;
    /// Error type
    type Error;

    /// Render the manifest to the output format
    fn render(&self, manifest: &Manifest) -> Result<Self::Output, Self::Error>;
}
