// This file contains all errors

use failure::*;
use std::convert::From;
use std::io::Error as IOError;

#[derive(Debug, Fail)]
pub enum SideFuzzError {
    #[fail(display = "The first input and the second input are not the same size.")]
    InputsDifferentSizes,

    #[fail(display = "Could not read file: {}", 0)]
    CouldNotReadFile(IOError),

    #[fail(display = "invalid toolchain name: {}", name)]
    InvalidToolchainName { name: String },
    #[fail(display = "unknown toolchain version: {}", version)]
    UnknownToolchainVersion { version: String },
}

impl From<IOError> for SideFuzzError {
    fn from(error: IOError) -> Self {
        SideFuzzError::CouldNotReadFile(error)
    }
}