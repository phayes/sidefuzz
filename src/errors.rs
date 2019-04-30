// This file contains all errors

use failure::*;
use std::convert::From;
use std::io::Error as IOError;
use wasmi::Error as WasmError;

#[derive(Debug, Fail)]
pub enum SideFuzzError {
    #[fail(display = "The first input and the second input are not the same size.")]
    InputsDifferentSizes,

    #[fail(
        display = "The input is of the wrong length for this fuzzing target. The target wants an input of {} bytes.",
        0
    )]
    InputsWrongSize(usize),

    #[fail(display = "Could not read file: {}", 0)]
    CouldNotReadFile(IOError),

    #[fail(display = "wasm error: {}", 0)]
    WasmError(WasmError),

    #[fail(display = "wasm module expected to have 'memory' export")]
    WasmModuleNoMemory,

    #[fail(display = "wasm module exported non-memory to 'memory' export")]
    WasmModuleBadMemory,

    #[fail(display = "wasm module expected to have 'input_pointer' that returns an i32")]
    WasmModuleNoInputPointer,

    #[fail(display = "wasm module expected to have 'input_len' that returns an i32")]
    WasmModuleNoInputLen,

    #[fail(display = "wasm module expected to have 'fuzz' function export")]
    WasmModuleNoFuzz,

    #[fail(display = "wasm module input_pointer returned bad type, i32 expected.")]
    WasmModuleBadInputPointer,

    #[fail(display = "wasm module 'input_len' returned bad type, i32 expected.")]
    WasmModuleBadInpuLen,

    #[fail(display = "error writing input memory to wasm: {}", 0)]
    MemorySetError(WasmError),

    #[fail(
        display = "requested fuzzing input length of {} is too long. 1024 bytes is the maximum.",
        0
    )]
    FuzzLenTooLong(u32),
}

impl From<IOError> for SideFuzzError {
    fn from(error: IOError) -> Self {
        SideFuzzError::CouldNotReadFile(error)
    }
}

impl From<WasmError> for SideFuzzError {
    fn from(error: WasmError) -> Self {
        SideFuzzError::WasmError(error)
    }
}