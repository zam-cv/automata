# Joel Vargas Reynoso A01752464
# Carlos Aberto Zamudio Velazquez A01799283

import ast
import os

def dfa_test(input_filename, test_filename):
    symbols: list[str]
    aceptation_states: set[int]
    graph: list[list[int]] = []

    # Pre-procesamiento del archivo
    with open(input_filename, 'r') as file:
        # "ast.literal_eval" para una evaluación segura de los datos
        symbols = ast.literal_eval(file.readline().strip())
        file.readline()  # Ignora la línea del número de estados
        aceptation_states = set(ast.literal_eval(file.readline().strip()))
        graph = [ast.literal_eval(line.strip()) for line in file]

    response: str = ''

    with open(test_filename, 'r') as file:
        for line in file:
            line = line.strip() # Elimina los espacios en blanco y saltos de línea
            current_state = 0

            for symbol in line:
              transitions = graph[current_state]

              for index, state in enumerate(transitions):
                transition_symbol = symbols[index]

                if transition_symbol == symbol:
                  current_state = state
                  break

            if current_state in aceptation_states:
              response += 'A\n'
            else:
              response += 'R\n'

    base, extension = os.path.splitext(test_filename)
    output_file_name = f"{base}-output{extension}"
    with open(output_file_name, 'w') as file:
      # Escribe todos los resultados de una vez para
      # minimizar la escritura en disco
      file.writelines(response)

# Itera sobre un rango fijo de archivos de prueba, a
# sumiendo que existen y están correctamente nombrados
for i in range(1, 5):
    dfa_test(f'DFA-pruebas/dfa-0{i}.txt', f'DFA-pruebas/test-0{i}.txt')
