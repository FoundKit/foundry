pub mod carnival_demo;

use carnival_demo::CarnivalDemoModule;
use foundry_core::SubsystemModule;

/// Factory function to discover and instantiate all code-first sub-system modules
pub fn register_subsystems() -> Vec<Box<dyn SubsystemModule>> {
    vec![Box::new(CarnivalDemoModule)]
}
