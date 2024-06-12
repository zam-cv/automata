use crate::{
    analyzer::{Analyzer, Error},
    utils::preprocess_content,
};
use std::fs::{self, DirEntry};

// Función auxiliar para calcular el número de línea en el archivo de entrada a partir de una posición dada.
fn calculate_line_number_from_position(position: usize, input: &str) -> usize {
    input[..position].matches('\n').count() + 2
}

// Función para resaltar errores en el contenido HTML basado en las posiciones de los errores parseados.
pub fn highlight_errors_in_html(errors: &[Error], html_content: &mut String, input: &str) {
    let code_start_marker = "<code class=\"code\">";
    let code_end_marker = "</code>";

    if let Some(code_block_start) = html_content.find(code_start_marker) {
        if let Some(code_block_end) = html_content.find(code_end_marker) {
            let code_block_content =
                &html_content[code_block_start + code_start_marker.len()..code_block_end];
            let code_lines: Vec<&str> = code_block_content.split("<br>").collect();
            let mut new_code_block_content = String::new();

            for (i, line) in code_lines.iter().enumerate() {
                let line_num = i + 1;
                let is_error_line = errors.iter().any(|error| {
                    let start_line = calculate_line_number_from_position(error.first, input);
                    line_num == start_line
                });

                if is_error_line {
                    new_code_block_content.push_str(&format!(
                        "<span style=\"background-color: rgba(255, 0, 0, 0.3);\">{}</span><br>",
                        line
                    ));
                } else {
                    new_code_block_content.push_str(line);
                    if i < code_lines.len() - 1 {
                        new_code_block_content.push_str("<br>");
                    }
                }
            }

            let new_code_content = format!(
                "{}{}{}",
                code_start_marker, new_code_block_content, code_end_marker
            );
            html_content.replace_range(
                code_block_start..code_block_end + code_end_marker.len(),
                &new_code_content,
            );
        }
    }
}

pub fn create_mark(analyzer: &Analyzer, entry: &DirEntry) {
    let input_path = entry.path();
    let input_filename = input_path.file_stem().unwrap().to_str().unwrap();
    let input = fs::read_to_string(&input_path).expect("Failed to read file");

    // Preprocesa el contenido para asegurarte de que todos los caracteres sean ASCII.
    let preprocessed_input = preprocess_content(&input);

    // Procesa el contenido del archivo de texto.
    let parsed = analyzer.parse(&preprocessed_input);

    let mut html_content =
        fs::read_to_string("template/tem.html").expect("Failed to read template");
    let mut generated_content = format!("<span></span><br>1 ");
    let mut count = 1;

    parsed.visit(&mut |chunk| match chunk.rule() {
        "WHITESPACE*" | "WHITESPACE+" => {
            let content = chunk.value();
            for c in content.chars() {
                match c {
                    '\n' => {
                        generated_content.push_str("</span>");
                        count += 1;
                        generated_content.push_str(&format!("<br><span class=\"line\">{} ", count));
                    }
                    ' ' => generated_content.push_str("&nbsp;"),
                    _ => {}
                }
            }
        }
        "string" => match chunk.value() {
            "," | "." | ";" | "=" => {
                let span = format!("<span class=\"white\">{}</span>", chunk.value());
                generated_content.push_str(&span);
            }
            "{" | "(" | ")" | "}" => {
                let span = format!("<span class=\"brackets\">{}</span>", chunk.value());
                generated_content.push_str(&span);
            }
            _ => {}
        },
        "unknown" => {
            let span = format!("<span class=\"unknown\">{}</span>", chunk.value());
            generated_content.push_str(&span);
        }
        _ => {
            if generated_content.ends_with("<br>") || generated_content.is_empty() {
                generated_content.push_str(&format!("{} ", count));
            }
            let capture = match chunk.rule() {
                "keyword" | "ident" | "number" | "logical_operators" | "algebraic_operators" => {
                    Some(chunk.value())
                }
                _ => None,
            };

            if let Some(capture) = capture {
                let span = format!("<span class=\"{}\">{}</span>", chunk.rule(), capture);
                generated_content.push_str(&span);
            }
        }
    });

    let content_insert = "<!-- Insert point for editor content -->";
    let error_insert = "<!-- Insert errors -->";

    if let Some(insert_pos) = html_content.find(content_insert) {
        html_content.insert_str(insert_pos, &generated_content);
    } else {
        println!("Marcador de inserción de contenido no encontrado.");
    }

    if !parsed.errors.is_empty() {
        let error_content = parsed
            .errors
            .iter()
            .map(|error| {
                format!(
                    ">>Error: {} en la posición {} a {} \n <br>",
                    error.message, error.first, error.last
                )
            })
            .collect::<String>();

        if let Some(insert_pos) = html_content.find(error_insert) {
            html_content.insert_str(insert_pos + error_insert.len(), &error_content);
            highlight_errors_in_html(&parsed.errors, &mut html_content, &input);
        } else {
            println!("Marcador de inserción de errores no encontrado.");
        }
    }

    let output_path = format!("results/{}.html", input_filename);
    fs::write(output_path, html_content).expect("Failed to write HTML file");
}
