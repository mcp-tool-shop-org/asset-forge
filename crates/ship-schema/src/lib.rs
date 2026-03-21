pub mod defaults;
pub mod spec;
pub mod style;
pub mod validate;
pub mod variant;

// Re-export key types at crate root
pub use spec::SloopAssetSpec;
pub use validate::{validate_spec, SpecError};
