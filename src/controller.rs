use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};
use std::thread;

use gotham_derive::*;

use crate::imposter;
use crate::imposter::Imposter;
use crate::webapi;

#[derive(Clone, StateData)]
pub struct ImposterList {
    inner: Arc<Mutex<Vec<Imposter>>>,
}


impl ImposterList {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn upsert(&self, imposter: Imposter) -> bool {
        let mut did_insert = true;
        let mut guard = self.inner.lock().unwrap();
        let list = guard.borrow_mut();
        for i in 0..(list.len()) {
            if list[i].id == imposter.id {
                list.remove(i);
                did_insert = false;
            }
        }
        list.push(imposter);
        did_insert
    }

    pub fn delete_by_id(&self, id: u32) -> bool {
        let mut guard = self.inner.lock().unwrap();
        let list = guard.borrow_mut();
        for i in 0..(list.len()) {
            if list[i].id == id {
                list.remove(i);
                return true;
            }
        }
        false
    }

    pub fn get_all(&self) -> Vec<Imposter> {
        let mut guard = self.inner.lock().unwrap();
        guard.borrow_mut().clone()
    }

    pub fn get_by_id(&self, id: u32) -> Option<Imposter> {
        let mut guard = self.inner.lock().unwrap();
        for imposter in guard.borrow_mut().iter_mut() {
            if imposter.id == id {
                return Some(imposter.clone());
            }
        }
        None
    }

    pub fn do_with_imposter_by_id<F>(&mut self, id: u32, mut func: F) where F: FnMut(&mut Imposter) {
        let mut guard = self.inner.lock().unwrap();
        for imposter in guard.borrow_mut().iter_mut() {
            if imposter.id == id {
                func(imposter);
            }
        }
    }
}


pub fn run(imposter_files: Vec<String>) {
    let list = ImposterList::new();

    for file in imposter_files {
        let imposter = Imposter::from_file(&file);
        list.upsert(imposter);
    }

    let cloned_list = list.clone();
    let addr = format!("{}:{}", "localhost", 8080);
    thread::spawn(move || {
        webapi::run(addr, cloned_list);
    });

    let cloned_list = list.clone();
    let port_id = 0;
    let h = thread::spawn(move || {
        imposter::run(port_id, cloned_list)
    });
    h.join().unwrap();
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_differentiates_imposters_with_different_ids() {
        let list = ImposterList::new();

        list.upsert(Imposter::from_json(r#"{"id": 1, "stubs": []}"#));
        list.upsert(Imposter::from_json(r#"{"id": 2, "stubs": []}"#));

        assert_eq!(2, list.get_all().len());
    }

    #[test]
    fn upsert_replaces_existing_imposter_with_same_id() {
        let list = ImposterList::new();

        list.upsert(Imposter::from_json(r#"{"id": 1, "stubs": []}"#));
        list.upsert(Imposter::from_json(r#"{"id": 1, "stubs": []}"#));

        assert_eq!(1, list.get_all().len());
    }

}


