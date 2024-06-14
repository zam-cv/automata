use crate::{analyzer::Analyzer, generate::create_mark};
use rayon::prelude::*;
use std::{collections::VecDeque, fs::DirEntry};

// Procesa cada archivo en la cola en paralelo.
#[allow(dead_code)]
pub fn parallel(files_queue: VecDeque<DirEntry>, analyzer: &Analyzer, template: &String, n: usize) {
    println!("Procesando en paralelo...");
    files_queue
        .range(..n)
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|entry| create_mark(&analyzer, entry, template));
}

// Procesa cada archivo en la cola de forma secuencial.
#[allow(dead_code)]
pub fn sequential(
    files_queue: VecDeque<DirEntry>,
    analyzer: &Analyzer,
    template: &String,
    n: usize,
) {
    println!("Procesando de forma secuencial...");
    files_queue
        .range(..n)
        .collect::<Vec<_>>()
        .iter()
        .for_each(|entry| create_mark(&analyzer, entry, template));
}
