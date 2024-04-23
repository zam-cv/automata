// Importamos el módulo analizador personalizado y utilidades para manejo de archivos.
use analyzer::Analyzer;
use std::fs;
use crate::analyzer::Error;

// Declaramos los módulos que usaremos en este archivo.
mod analyzer;
mod parsed;

// Constante que almacena la gramática cargada desde un archivo de texto.
const GRAMMAR: &'static str = include_str!("grammar.txt");

// La función main dirige el flujo principal de ejecución del programa.
fn main() -> anyhow::Result<()> {
    // Se crea una instancia del analizador con la gramática y la regla inicial.
    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    // Se valida la gramática para asegurarse de que es coherente y completa.
    analyzer.validate()?;

    // Se lee el archivo de entrada y se parsea el contenido.
    let input = fs::read_to_string("src/test.txt")?;
    let parsed = analyzer.parse(&input);
    println!("{:?}", parsed.token);

    // Se prepara el contenido HTML inicial para la visualización del código parseado.
    let mut html_content = fs::read_to_string("template/tem.html")?;
    let mut generated_content = format!("<span></span><br>1 ");
    let mut count = 1;

    // Se visita recursivamente los elementos parseados hasta que hagan match con alguna de las condiciones.
    // Aplicando un estilo específico a cada tipo de token para resaltarlos en el HTML.

    parsed.visit(&mut |chunk| match chunk.rule() {
        // Espacios en blanco se manejan especialmente para preservar el formato en HTML.
        "WHITESPACE*" | "WHITESPACE+" => {
            let content = chunk.value();
            for c in content.chars() {
                match c {
                    '\n' => {
                        generated_content.push_str("</span>");
                        count += 1;
                        generated_content.push_str(&format!("<br><span class=\"line\">{} ", count));
                    },
                    ' ' => generated_content.push_str("&nbsp;"),
                    _ => {},
                }
            }
        },
        // Se manejan diferentes tipos de símbolos para aplicarles un estilo específico.
        "string" => {
            match chunk.value() {
                "," | "." | ";" | "=" => {
                    let span = format!("<span class=\"white\">{}</span>", chunk.value());
                    generated_content.push_str(&span);
                },
                "{" | "(" | ")" | "}" => {
                    let span = format!("<span class=\"brackets\">{}</span>", chunk.value());
                    generated_content.push_str(&span);
                }
                _ => {}
            }
        },
        // Fragmentos desconocidos se marcan especialmente.
        "unknown" => {
            let span = format!("<span class=\"unknown\">{}</span>", chunk.value());
            generated_content.push_str(&span);
        },
        _ => {
            if generated_content.ends_with("<br>") || generated_content.is_empty() {
                generated_content.push_str(&format!("{} ", count));
            }
            let capture = match chunk.rule() {
                "keyword" | "ident" | "number" | "logical_operators" | "algebraic_operators" => Some(chunk.value()),
                _ => None,
            };

            if let Some(capture) = capture {
                let span = format!("<span class=\"{}\">{}</span>", chunk.rule(), capture);
                generated_content.push_str(&span);
            }
        }
    });

    // Se inserta el contenido generado en el archivo HTML en los puntos designados.
    let content_insert = "<!-- Insert point for editor content -->";
    let error_insert = "<!-- Insert errors -->";

    if let Some(insert_pos) = html_content.find(content_insert) {
        html_content.insert_str(insert_pos, &generated_content);
    } else {
        println!("Marcador de inserción de contenido no encontrado.");
    }

    // Se insertan los errores en el archivo HTML si los hay.
    if !parsed.errors.is_empty() {
        let error_content = parsed.errors.iter()
            .map(|error| format!(">>Error: {} en la posición {} a {} \n <br>", error.message, error.first, error.last))
            .collect::<String>();

        if let Some(insert_pos) = html_content.find(error_insert) {
            html_content.insert_str(insert_pos + error_insert.len(), &error_content);
            highlight_errors_in_html(&parsed.errors, &mut html_content, &input);
        } else {
            println!("Marcador de inserción de errores no encontrado.");
        }
    }

    // Finalmente, se escribe el contenido modificado de vuelta al archivo HTML.
    fs::write("index.html", html_content)?;

    Ok(())
}

// Función para resaltar errores en el contenido HTML basado en las posiciones de los errores parseados.
fn highlight_errors_in_html(errors: &[Error], html_content: &mut String, input: &str) {
    let code_start_marker = "<code class=\"code\">";
    let code_end_marker = "</code>";

    if let Some(code_block_start) = html_content.find(code_start_marker) {
        if let Some(code_block_end) = html_content.find(code_end_marker) {
            let code_block_content = &html_content[code_block_start + code_start_marker.len()..code_block_end];
            let code_lines: Vec<&str> = code_block_content.split("<br>").collect();
            println!("{:?}", code_lines);
            let mut new_code_block_content = String::new();

            for (i, line) in code_lines.iter().enumerate() {
                let line_num = i + 1;
                let is_error_line = errors.iter().any(|error| {
                    let start_line = calculate_line_number_from_position(error.first, input);
                    line_num == start_line
                });

                if is_error_line {
                    new_code_block_content.push_str(&format!("<span style=\"background-color: rgba(255, 0, 0, 0.3);\">{}</span><br>", line));
                } else {
                    new_code_block_content.push_str(line);
                    if i < code_lines.len() - 1 {
                        new_code_block_content.push_str("<br>");
                    }
                }
            }

            let new_code_content = format!("{}{}{}", code_start_marker, new_code_block_content, code_end_marker);
            html_content.replace_range(code_block_start..code_block_end + code_end_marker.len(), &new_code_content);
        }
    }
}

// Función auxiliar para calcular el número de línea en el archivo de entrada a partir de una posición dada.
fn calculate_line_number_from_position(position: usize, input: &str) -> usize {
    input[..position].matches('\n').count() + 2
}
