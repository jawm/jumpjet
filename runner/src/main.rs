#[macro_use(args)]
extern crate jump_jet;

use std::collections::HashMap;
use std::fs::File;

use jump_jet::runtime::exports::GetExport;
use jump_jet::runtime::exports::ValueTypeProvider;
use jump_jet::tree::language_types::ExternalKind;


fn main() {
    println!("Testing JumpJet");

    // // Builds the runtime, ready to import some modules.
    // let mut runtime = jump_jet::Runtime::new();

    // // Adds a group of functions to be used, accessible through their namespace
    // //runtime.expose("namespace", vec![]);

    // // opens the module, generates the program tree
    // runtime.add_module("main", "program.wasm");

    // // runs any startup specified by the module
    // runtime.prepare("main");

    // // runs a function from the module
    // if let Some(module) = runtime.get("main") {
    //     if let Some(&ExternalKind::Function(f)) = module.exports.get("callByIndex") {
    //         //f.execute();
    //     }
    // }

    let module = jump_jet::instantiate(&mut File::open("program.wasm").unwrap(), HashMap::new()).unwrap();
    module.exports.get_function("callByIndex", &module).unwrap()(args![1,2.4,3]);
}
