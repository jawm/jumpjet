#[macro_use(args)]
extern crate jump_jet;
extern crate env_logger;

use std::collections::HashMap;
use std::fs::File;

use jump_jet::runtime_tree::ExternalKindInstance;
use jump_jet::runtime::exports::GetExport;
use jump_jet::runtime_tree::ValueTypeProvider;

fn main() {
    println!("Testing JumpJet");
    env_logger::init().unwrap();
    let mut imports = HashMap::new();
    let mut imports_env = HashMap::new();
    imports_env.insert("test".to_string(), ExternalKindInstance::Function(Box::new(|a,b|vec![])));
    imports.insert("env".to_string(), imports_env);
    let mut module = jump_jet::instantiate(&mut File::open("out.wasm").unwrap(), imports).unwrap();
    println!("parsed");
    let rets = module.exports().call_fn("add", vec![ValueTypeProvider::I32(1), ValueTypeProvider::I32(-10)]);
    println!("rets {:#?}", rets);
}
