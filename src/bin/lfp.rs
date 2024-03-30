use std::env::args;

use glotaran_converter_lib::run_lfp;


fn main() {
	let filename =  args().last().unwrap();
	let _otp = run_lfp(&filename).unwrap();
}