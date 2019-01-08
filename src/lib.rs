extern crate core;
extern crate gotham;
extern crate hyper;
extern crate serde;
extern crate futures;
extern crate serde_json;

pub mod controller;
pub mod imposter;
pub mod stub;
pub mod predicate;
pub mod response;
pub mod can;
pub mod utils;
pub mod webapi;

pub fn run(imposter_files: Vec<String>)
{
    controller::run(imposter_files);
}
