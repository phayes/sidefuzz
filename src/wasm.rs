use crate::errors::SideFuzzError;
use float_duration::{FloatDuration, TimePoint};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

use wasmi::{
  ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef, NopExternals, RuntimeValue,
};

pub struct WasmModule {
  module: Vec<u8>,
  instance: ModuleRef,
  memory: MemoryRef,
  fuzz_len: usize,
}

impl WasmModule {
  pub fn new(module: Vec<u8>) -> Self {
    let parsed = Module::from_buffer(&module).unwrap();
    let instance = ModuleInstance::new(&parsed, &ImportsBuilder::default())
      .expect("failed to instantiate wasm module")
      .assert_no_start();

    // Get memory instance exported by name 'mem' from the module instance.
    let memory = instance.export_by_name("memory");
    let memory = memory.expect("Module expected to have 'mem' export");
    let memory = memory.as_memory().unwrap();

    // Get the fuzz length
    let fuzz_len = fuzz_len(&instance);

    let mut wasm_module = Self {
      module: module,
      instance: instance,
      memory: memory.to_owned(),
      fuzz_len,
    };

    // Run it once to prime lazy statics.
    let input: Vec<u8> = (0..fuzz_len).map(|_| rand::random::<u8>()).collect();
    crate::black_box(wasm_module.count_instructions(&input).unwrap());

    wasm_module
  }

  pub fn from_file(filename: &str) -> Result<Self, SideFuzzError> {
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(Self::new(buf))
  }

  pub fn fuzz_len(&self) -> usize {
    self.fuzz_len
  }

  pub fn bytes(&self) -> Vec<u8> {
    self.module.clone()
  }

  // Count instructions for a given input
  pub fn count_instructions(&mut self, input: &[u8]) -> Result<u64, ()> {
    self.memory.set(0, input).unwrap();
    wasmi::reset_instruction_count();
    let result = self.instance.invoke_export(
      "sidefuzz",
      &[
        RuntimeValue::I32(0),
        RuntimeValue::I32(i32::try_from(input.len()).unwrap()),
      ],
      &mut NopExternals,
    );
    if let Err(err) = result {
      // If we've got a MemoryAccessOutOfBounds error, then we've corrupted our memory.
      // In a real application this would be a crash, so reboot the instance and start over.
      if let wasmi::Error::Trap(trap) = err {
        if let wasmi::TrapKind::MemoryAccessOutOfBounds = trap.kind() {
          self.reboot();
        }
      }
      return Err(());
    }
    let count = wasmi::get_instruction_count();

    Ok(count)
  }

  // Restart / Reboot the instance
  fn reboot(&mut self) {
    let new = Self::new(self.module.clone());
    self.instance = new.instance;
    self.memory = new.memory;
  }

  // Measure and report the running time for a single execution
  pub fn measure_time(&mut self) -> FloatDuration {
    let input: Vec<u8> = (0..self.fuzz_len).map(|_| rand::random::<u8>()).collect();
    let start_time = Instant::now();
    self.count_instructions(&input).unwrap();
    let end_time = Instant::now();

    end_time.float_duration_since(start_time).unwrap()
  }
}

impl Clone for WasmModule {
  fn clone(&self) -> Self {
    Self::new(self.module.clone())
  }
}


/// Get the array input length for fuzzing
///
/// This is defined by the fuzzing target by exporting the "len()" function.
fn fuzz_len(module_instance: &ModuleRef) -> usize {
  // Call the "len" exported function to get the desired fuzzing length.
  let fuzz_len = module_instance
    .invoke_export("len", &[], &mut NopExternals)
    .expect("wasm module did not export a len() function")
    .unwrap();

  let fuzz_len = match fuzz_len {
    wasmi::RuntimeValue::I32(inner) => inner as usize,
    _ => {
      // TODO: return don't panic;
      panic!("Invalid len() return type");
    }
  };

  fuzz_len
}

