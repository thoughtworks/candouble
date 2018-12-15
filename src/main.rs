extern crate candouble;
extern crate getopts;

use getopts::Options;
use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
//    opts.optopt("o", "", "set output file name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let input_files = if !matches.free.is_empty() {
        matches.free.clone()
    } else {
        vec![]
    };
    candouble::run(input_files);
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] files", program);
    print!("{}", opts.usage(&brief));
}
