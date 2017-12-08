extern crate jump_jet;

fn main() {
    println!("Testing JumpJet");

    // Builds the runtime, ready to import some modules.
    let mut runtime = jump_jet::Runtime::new();

    // Adds a group of functions to be used, accessible through their namespace
    runtime.expose("namespace", vec![]);

    // opens the module, generates the program tree
    runtime.add_module("main", "program.wasm");

    // runs any startup specified by the module
    runtime.prepare("main");

    // runs a function from the module
    match runtime.get("main") {
        Some(module) => module.run(),
        None         => println!("The module failed to load! Quitting or something")
    }
}
