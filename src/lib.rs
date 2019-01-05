
pub mod imposter;
pub mod stub;
pub mod predicate;
pub mod response;
pub mod can;
pub mod utils;
pub mod webapi;

use crate::imposter::ImposterList;
use crate::imposter::Imposter;

pub fn run(imposter_files: Vec<String>)
{
    webapi::start_listener("localhost", 8080, ImposterList::new());

    for file in imposter_files {
        Imposter::from_file(&file).run();
        // TODO: at the moment the run function blocks; only first imposter runs
    }
}
