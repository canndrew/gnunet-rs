use util::ToCPathError;
use std::error::FromError;

/// Errors returned by `Configuration::load`.
#[derive(Debug)]
pub enum ConfigurationLoadError {
  /// The path given was malformed.
  BadPath(ToCPathError),
  /// The path given does not exist.
  NoSuchFile,
}
error_chain! {ToCPathError, ConfigurationLoadError, BadPath}

/// Errors returned by `Configuration::save`.
#[derive(Debug)]
pub enum ConfigurationSaveError {
  /// The path given was malformed.
  BadPath(ToCPathError),
  /// The underlying library call failed.
  UnknownError,
}
error_chain! {ToCPathError, ConfigurationSaveError, BadPath}

/// Returned by `Configuration::from_str` when parsing fails.
#[derive(Debug)]
pub struct ConfigurationFromStrError;

