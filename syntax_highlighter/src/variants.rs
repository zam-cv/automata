use crate::{analyzer::Analyzer, generate::create_mark};
use rayon::prelude::*;
use std::{collections::VecDeque, fs::DirEntry};

// Procesa cada archivo en la cola en paralelo.
#[allow(dead_code)]
pub fn parallel(files_queue: VecDeque<DirEntry>, analyzer: &Analyzer, template: &String) {
    println!("Procesando en paralelo...");
    files_queue
        .par_iter()
        .for_each(|entry| create_mark(&analyzer, entry, template));
}

// Procesa cada archivo en la cola de forma secuencial.
#[allow(dead_code)]
pub fn sequential(files_queue: VecDeque<DirEntry>, analyzer: &Analyzer, template: &String) {
    println!("Procesando de forma secuencial...");
    for entry in files_queue {
        create_mark(&analyzer, &entry, template);
    }
}