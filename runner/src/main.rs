#[macro_use(args)]
extern crate jump_jet;

use std::collections::HashMap;
use std::fs::File;

use jump_jet::runtime::exports::GetExport;
use jump_jet::runtime::exports::ValueTypeProvider;
use jump_jet::tree::language_types::ExternalKind;


fn main() {
    println!("Testing JumpJet");
    let module = jump_jet::instantiate(&mut File::open("program.wasm").unwrap(), HashMap::new()).unwrap();
    module.exports.get_function("callByIndex", &module).unwrap()(args![1i32]);
}
