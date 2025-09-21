//! Validation module for schema compliance and contract testing

pub mod schema_validator;

pub use schema_validator::{
    get_validator, is_validation_enabled, validate_json, validate_json_conditional,
    validate_message, validate_message_conditional, SchemaValidator, ValidationError,
};
