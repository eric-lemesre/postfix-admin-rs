//! Domain models, traits, and shared types for postfix-admin-rs.
//!
//! This crate contains the core domain logic with no framework dependencies.
//! All other crates depend on this one.

pub mod dto;
pub mod error;
pub mod models;
pub mod pagination;
pub mod repository;
pub mod types;
pub mod validation;

pub use error::{CoreError, DomainError, ValidationError};
pub use pagination::{PageRequest, PageResponse, SortDirection};
pub use types::{DomainName, EmailAddress, IpAddress, Password};
