#[macro_use(args)]
extern crate jump_jet;
extern crate env_logger;

use std::collections::HashMap;
use std::fs::File;

use jump_jet::runtime_tree::Import;
use jump_jet::runtime::exports::GetExport;
use jump_jet::runtime_tree::ValueTypeProvider;

fn build_imports() -> HashMap<String, HashMap<String, Import>> {
    let mut imports = HashMap::new();
    let mut imports_env = HashMap::new();
    imports_env.insert("test".to_string(), Import::Function(Box::new(|a,b|{
        println!("being called");
        vec![ValueTypeProvider::I32(16)]
    })));
    imports.insert("env".to_string(), imports_env);
    imports
}

fn main() {
    println!("Testing JumpJet");
    env_logger::init().unwrap();
    let imports = build_imports();
    let module_template = jump_jet::instantiate(&mut File::open("out.wasm").unwrap(), imports).unwrap();
    let mut module_instance = module_template.instantiate().unwrap();
    let rets = module_instance.exports().call_fn("add", vec![ValueTypeProvider::I32(42), ValueTypeProvider::I32(123)]);
    println!("rets {:#?}", rets);
}
