pub mod defaults;
pub mod migrate;
pub mod spec;
pub mod style;
pub mod validate;
pub mod variant;

// Re-export key types at crate root
pub use migrate::{migrate_to_current, CURRENT_VERSION};
pub use spec::SloopAssetSpec;
pub use validate::{validate_spec, SpecError};
