// ish-stdlib: Standard library, code analyzer, and Rust generator for the ish prototype.
//
// All components are defined as ish programs (ASTs built via the Rust builder API)
// and executed by the interpreter. This demonstrates the self-hosting capability.

pub mod analyzer;
pub mod generator;
pub mod stdlib;

use std::cell::RefCell;
use std::rc::Rc;
use ish_vm::interpreter::IshVm;

/// Load all stdlib functions, analyzer, and generator into the VM.
pub async fn load_all(vm: &Rc<RefCell<IshVm>>) {
    stdlib::register_stdlib(vm).await;
    analyzer::register_analyzer(vm).await;
    generator::register_generator(vm).await;
}
