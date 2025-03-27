// Локальный пример TRIAD без сетевого взаимодействия
//
// Этот исполняемый файл запускает локальный пример TRIAD,
// демонстрирующий работу квантового ядра без взаимодействия с другими компьютерами.
//
// Использование:
//   cargo run --bin local_example -- --qubits=10

use triad::quantum::triadvantum;

fn main() {
    // Парсинг аргументов командной строки
    let args: Vec<String> = std::env::args().collect();
    
    // Количество кубитов
    let qubit_count = args.iter()
        .find(|a| a.starts_with("--qubits="))
        .map(|s| s.trim_start_matches("--qubits=").parse::<usize>().unwrap_or(10))
        .unwrap_or(10);
    
    // Запуск локального примера
    triadvantum::run_local_example(qubit_count);
} 