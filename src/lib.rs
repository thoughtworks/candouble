
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate core;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate futures;

pub mod imposter;
pub mod stub;
pub mod predicate;
pub mod response;
pub mod can;
pub mod utils;
pub mod webapi;


pub fn run(input_files: Vec<String>)
{
    let mut webapi = webapi::WebApi::new();
    webapi.run("localhost", 8080);

    let mut imposter = imposter::Imposter::new();
    for i in 0..(input_files.len()) {
        let fname = &input_files[i];
        imposter.load_stub(fname).expect(&format!("Failed to load stub from {}", fname));
    }
    imposter.run();
}
