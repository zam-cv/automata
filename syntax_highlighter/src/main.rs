use analyzer::Analyzer;

use std::{collections::VecDeque, fs, time::Instant};

// Declaramos los módulos que usaremos en este archivo.
mod analyzer;
mod generate;
mod parsed;
mod utils;
mod variants;

// Constante que almacena la gramática cargada desde un archivo de texto.
const GRAMMAR: &'static str = include_str!("grammar.txt");

// La función main dirige el flujo principal de ejecución del programa.
fn main() -> anyhow::Result<()> {
    // Inicia el temporizador.
    let start_time = Instant::now();

    // Crea una instancia del analizador con la gramática y la regla inicial.
    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    analyzer.validate()?;

    // Llama a la función que procesa los archivos en la cola.
    let files_queue = get_files_queue()?;
    println!("Archivos encontrados: {}", files_queue.len());

    let template = fs::read_to_string("template/tem.html")?;

    // Procesa cada archivo en la cola en paralelo.
    variants::parallel(files_queue, &analyzer, &template);

    // Procesa cada archivo en la cola de forma secuencial.
    // variants::sequential(files_queue, &analyzer, &template);

    // Detiene el temporizador y calcula la duración.
    let duration = start_time.elapsed();
    println!("Tiempo de ejecución: {:?}", duration);

    Ok(())
}

fn get_files_queue() -> anyhow::Result<VecDeque<std::fs::DirEntry>> {
    // Crea una cola de archivos de texto en la carpeta "multipletext".
    let files_queue: VecDeque<_> = fs::read_dir("test")?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "txt"))
        .collect();

    // Crea la carpeta "results" si no existe.
    fs::create_dir_all("results")?;

    Ok(files_queue)
}
