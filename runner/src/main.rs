extern crate jump_jet;

fn main() {

	let test = &vec![1,2,3];
	for i in test {
		println!("{}", i);
	}


    println!("Testing JumpJet");
    jump_jet::build_module("program.wasm");
}
