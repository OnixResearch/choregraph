//! Pure revalidation and lowering for evaluated restricted Nickel data.
//!
//! Source evaluation and capability-relative file access remain shell responsibilities.

pub mod construction;
pub mod diagnostic;
pub mod identity;
pub mod imports;
pub mod lower;
pub mod model;
pub mod profile;
pub mod source;

pub mod boundary;

#[cfg(test)]
mod tests;
