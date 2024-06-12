pub fn preprocess_content(input: &str) -> String {
  input.chars()
      .map(|c| match c {
          '\u{feff}' => ' ',
          'á' | 'â' |  'à' | 'Á' => 'a',
          'é' | 'ë' | 'è' | 'ê' | 'É' => 'e',
          'í' | 'ì' | 'î' |  'Í' => 'i',
          'ó' | 'œ' | 'æ' | 'ò' | 'Ó' => 'o',
          'ú' | 'ù' | 'Ú' => 'u',
          'ç'=> 'c',
          'ñ' | 'Ñ' => 'n',
          _ => c,
      })
      .collect()
}