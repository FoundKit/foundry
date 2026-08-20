pub mod carnival_demo;
pub mod external;

use carnival_demo::CarnivalDemoModule;
use external::load_external_subsystems;
use foundry_core::SubsystemModule;

/// Factory function to discover and instantiate all code-first and external sub-system modules
pub fn register_subsystems() -> Vec<Box<dyn SubsystemModule>> {
    let mut list: Vec<Box<dyn SubsystemModule>> = vec![Box::new(CarnivalDemoModule)];
    let external_list = load_external_subsystems();
    list.extend(external_list);
    list
}
