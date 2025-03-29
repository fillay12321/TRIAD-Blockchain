// Пример демонстрации суперпозиции и запутанности в TRIAD

extern crate triad;

use triad::api::{QuantumEngine, visualization};
use triad::api::algorithms;

fn main() {
    println!("=== Демонстрация суперпозиции и запутанности в TRIAD ===\n");
    
    // Создаем квантовый движок с 3 кубитами
    let mut engine = QuantumEngine::new(3);
    
    println!("1. Демонстрация суперпозиции");
    println!("----------------------------");
    
    // Сбрасываем состояние для чистого начала
    engine.reset();
    println!("Исходное состояние (|000⟩):");
    println!("{}", visualization::state_to_string(&engine));
    
    // Создаем равномерную суперпозицию
    engine.create_uniform_superposition();
    println!("Равномерная суперпозиция всех состояний:");
    println!("{}", visualization::state_to_string(&engine));
    
    // Создаем параметризованную суперпозицию на первом кубите
    engine.reset();
    let theta = std::f64::consts::PI / 8.0; // 22.5 градусов
    engine.create_parametrized_superposition(0, theta);
    println!("Параметризованная суперпозиция с θ = π/8 на кубите 0:");
    println!("{}", visualization::state_to_string(&engine));
    
    // Проверяем, находится ли кубит в суперпозиции
    println!("Кубит 0 в суперпозиции: {}", engine.is_in_superposition(0));
    println!("Кубит 1 в суперпозиции: {}", engine.is_in_superposition(1));
    
    println!("\n2. Демонстрация запутанности");
    println!("---------------------------");
    
    // Создаем состояние Белла
    engine.reset();
    engine.create_bell_state(0, 1).unwrap();
    println!("Состояние Белла между кубитами 0 и 1:");
    println!("{}", visualization::state_to_string(&engine));
    println!("Граф запутанности:");
    println!("{}", visualization::entanglement_graph(&engine));
    
    // Создаем состояние GHZ
    engine.reset();
    engine.create_ghz_state().unwrap();
    println!("Состояние GHZ для 3 кубитов:");
    println!("{}", visualization::state_to_string(&engine));
    println!("Граф запутанности:");
    println!("{}", visualization::entanglement_graph(&engine));
    
    println!("\n3. Демонстрация квантовых алгоритмов");
    println!("-----------------------------------");
    
    // Телепортация
    let state_to_teleport = (0.7, 0.7);
    let success = algorithms::teleport_state(state_to_teleport);
    println!("Телепортация состояния {:?}: {}", 
             state_to_teleport, if success { "успешно" } else { "не удалась" });
    
    // Сверхплотное кодирование
    let bits = (true, false);
    let result = algorithms::dense_coding(bits);
    println!("Сверхплотное кодирование битов {:?}: получено {:?}", bits, result);
    
    // Алгоритм Гровера
    let num_qubits = 3;
    let target = 6; // |110⟩
    let result = algorithms::grover(num_qubits, target);
    println!("Алгоритм Гровера: искали |{:03b}⟩, нашли |{:03b}⟩", target, result);
    
    println!("\n=== Демонстрация завершена ===");
} 