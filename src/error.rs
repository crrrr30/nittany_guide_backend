use thiserror::Error;

/*

make function to check for exisiting url, 

call that from the route (return error from this)

remove auto conversion from error to api error. 

*/

/// A custom error enum to encapsulate various errors that can occur within the application.
///
/// # Variants:
///
/// - `Sled`: Wraps errors originating from the `sled` database interactions.
/// - `Bincode`: Encapsulates serialization and deserialization errors from the `bincode` crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Sled Error: {0}")]
    Sled(#[from] sled::Error),

    #[error("Bincode Error: {0}")]
    Bincode(#[from] bincode::Error),
}
