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
    let module = jump_jet::instantiate(&mut File::open("program.wasm").unwrap(), HashMap::new()).unwrap();
    module.exports().call_fn("callByIndex", vec![ValueTypeProvider::I32(0)]);
}
