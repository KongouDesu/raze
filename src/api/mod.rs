//! Raw Rust API bindings for BackBlaze B2
//!
//! The various API calls are grouped into different modules based on their functionality

/// Methods related to authenticating usage of the API
///
/// This module if for getting the authorization token and url to use the API
///
/// Official documentation: [b2_authorize_account](https://www.backblaze.com/b2/docs/b2_authorize_account.html)
///
/// This really just sends a HTTP auth header containing "Basic " + Base64(keyId:applicationKey)
pub mod auth;
/// Methods for getting and modifying a users buckets
///
/// Official documentation: [Buckets](https://www.backblaze.com/b2/docs/buckets.html)
pub mod buckets;
/// A submodule for dealing with file related calls
pub mod files;