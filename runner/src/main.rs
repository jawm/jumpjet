extern crate jump_jet;

use std::collections::HashMap;

fn main() {
    println!("Testing JumpJet");

    // Builds the runtime, ready to import some modules.
    let mut runtime = jump_jet::Runtime::new();

    // Adds a group of functions to be used, accessible through their namespace
    runtime.expose("namespace".to_string(), vec![]);

    // opens the module, generates the program tree
    runtime.add_module("main".to_string(), "program.wasm".to_string());

    // runs any startup specified by the module
    runtime.prepare("main".to_string());

    // runs a function from the module
    match runtime.get("main".to_string()) {
        Some(module) => module.run(),
        None         => println!("The module failed to load! Quitting or something")
    }

    //let module = jump_jet::build_module("program.wasm");
}
