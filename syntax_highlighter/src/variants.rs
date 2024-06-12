use crate::{analyzer::Analyzer, generate::create_mark};
use rayon::prelude::*;
use std::{collections::VecDeque, fs::DirEntry};

// Procesa cada archivo en la cola en paralelo.
pub fn parallel(files_queue: VecDeque<DirEntry>, analyzer: &Analyzer) {
    files_queue
        .par_iter()
        .for_each(|entry| create_mark(&analyzer, entry));
}

// Procesa cada archivo en la cola de forma secuencial.
pub fn sequential(files_queue: VecDeque<DirEntry>, analyzer: &Analyzer) {
    for entry in files_queue {
        create_mark(&analyzer, &entry);
    }
}