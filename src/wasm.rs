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
  fuzz_ptr: i32,
  fuzz_len: i32,

}

impl WasmModule {
  pub fn new(module: Vec<u8>) -> Result<Self, SideFuzzError> {
    let parsed = Module::from_buffer(&module).unwrap();
    let instance = ModuleInstance::new(&parsed, &ImportsBuilder::default())?.assert_no_start();

    // Get memory instance exported by name 'mem' from the module instance.
    let memory = instance.export_by_name("memory");
    let memory = memory.ok_or(SideFuzzError::WasmModuleNoMemory)?;
    let memory = memory
      .as_memory()
      .ok_or(SideFuzzError::WasmModuleBadMemory)?;

    let mut wasm_module = Self {
      module: module,
      instance: instance,
      memory: memory.to_owned(),
      fuzz_ptr: 0,
      fuzz_len: 0,
    };

    // Set input pointers
    wasm_module.set_input_pointer();

    // Prime lazy statics
    wasm_module.prime_lazy_statics();

    Ok(wasm_module)
  }

  pub fn from_file(filename: &str) -> Result<Self, SideFuzzError> {
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(Self::new(buf)?)
  }

  pub fn fuzz_len(&self) -> usize {
    self.fuzz_len as usize
  }

  pub fn bytes(&self) -> Vec<u8> {
    self.module.clone()
  }

  // Count instructions for a given input
  pub fn count_instructions(&mut self, input: &[u8]) -> Result<u64, SideFuzzError> {
    self
      .memory
      .set(0, input)
      .map_err(|e| SideFuzzError::MemorySetError(e))?;
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
      if let wasmi::Error::Trap(trap) = &err {
        if let wasmi::TrapKind::MemoryAccessOutOfBounds = trap.kind() {
          self.reboot();
        }
      }
      return Err(SideFuzzError::WasmError(err));
    }
    let count = wasmi::get_instruction_count();

    Ok(count)
  }

  // Restart / Reboot the instance
  fn reboot(&mut self) {
    // This should be ok to expect here since the module has already been instantiated previously.
    let new = Self::new(self.module.clone()).expect("Could not reboot wasm module instance.");
    self.instance = new.instance;
    self.memory = new.memory;
  }

  // Measure and report the running time for a single execution
  pub fn measure_time(&mut self) -> Result<FloatDuration, SideFuzzError> {
    let input: Vec<u8> = (0..self.fuzz_len).map(|_| rand::random::<u8>()).collect();
    let start_time = Instant::now();
    self.count_instructions(&input)?;
    let end_time = Instant::now();

    Ok(end_time.float_duration_since(start_time).unwrap())
  }

  // Prime lazy statics
  pub fn prime_lazy_statics(&mut self) {

    // Prime until it completes successfully.
    loop {
      let input: Vec<u8> = (0..self.fuzz_len).map(|_| rand::random::<u8>()).collect();
      let result = self.count_instructions(&input);
      if result.is_ok() {
        return;
      }
    }

  }

  // Set the input fuzz length
  fn set_input_pointer(&mut self) {
    // Call "sidefuzz" to prime INPUT static global and set it's length
    let _ = crate::black_box(self.count_instructions(&vec![]));

    // Call the "input_pointer" exported function to get the pointer to the input
    let input_pointer = self
      .instance
      .invoke_export("input_pointer", &[], &mut NopExternals)
      .expect("wasm module did not export a input_pointer() function")
      .unwrap();

    // Call the "input_len" exported function to get the input length
    let input_len = self
      .instance
      .invoke_export("input_len", &[], &mut NopExternals)
      .expect("wasm module did not export a input_len() function")
      .unwrap();

    let input_pointer = match input_pointer {
      wasmi::RuntimeValue::I32(inner) => inner,
      _ => {
        // TODO: return don't panic;
        panic!("Invalid input_pointer() return type");
      }
    };

    let input_len = match input_len {
      wasmi::RuntimeValue::I32(inner) => inner,
      _ => {
        // TODO: return don't panic;
        panic!("Invalid input_len() return type");
      }
    };

    self.fuzz_ptr = input_pointer;
    self.fuzz_len = input_len;
  }

}

impl Clone for WasmModule {
  fn clone(&self) -> Self {
    // This should be ok to expect here since the module has already been instantiated previously.
    Self::new(self.module.clone()).expect("Unable to clone wasm module")
  }
}

