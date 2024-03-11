import ast
import os

# ['a', 'b'] -> Símbolos del alfabeto
# 5          -> Número de estados
# [2, 4]     -> Lista de estados de aceptación

#     0: 1:
# 0: [1, 3]
# 1: [1, 2]
# 2: [1, 2]
# 3: [4, 3]
# 4: [4, 3]

def dfa_test(input_filename, test_filename):
    symbols: list[str]
    aceptation_states: set[int]
    graph: list[list[int]] = []

    with open(input_filename, 'r') as file:
        symbols = ast.literal_eval(file.readline())
        file.readline()  # Ignorar el número de estados
        aceptation_states = set(ast.literal_eval(file.readline()))

        for line in file:
            graph.append(ast.literal_eval(line))

    response: str = ''

    with open(test_filename, 'r') as file:
        for line in file:
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
        file.write(response)

for i in range(1, 5):
    dfa_test(f'DFA-pruebas/dfa-0{i}.txt', f'DFA-pruebas/test-0{i}.txt')
