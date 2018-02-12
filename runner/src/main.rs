#[macro_use(args)]
extern crate jump_jet;
extern crate env_logger;

use std::collections::HashMap;
use std::fs::File;

use jump_jet::runtime::exports::GetExport;
use jump_jet::runtime_tree::ValueTypeProvider;

fn main() {
    println!("Testing JumpJet");
    env_logger::init().unwrap();
    let mut module = jump_jet::instantiate(&mut File::open("out.wasm").unwrap(), HashMap::new()).unwrap();
    let rets = module.exports().call_fn("add", vec![ValueTypeProvider::I32(1), ValueTypeProvider::I32(-10)]);
    println!("rets {:#?}", rets);
}
