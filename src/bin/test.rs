// Упрощенная версия для локального запуска без сетевого взаимодействия

// Импортируем необходимые библиотеки
use triad::quantum::triadvantum::state::QuantumState;
use triad::quantum::triadvantum::circuit::QuantumCircuit;
use triad::quantum::triadvantum::simulator::QrustSimulator;

fn main() {
    println!("Запуск локальной версии TRIAD...");
    
    // Создаем симулятор квантовой системы
    let node_id = "test_node".to_string();
    let mut simulator = QrustSimulator::new(node_id, 5, false).expect("Не удалось создать симулятор"); 
    println!("Создан квантовый симулятор с 5 кубитами");
    
    // Создаем схему запутанных состояний
    let mut circuit = QuantumCircuit::new(5);
    
    // Создаем состояние суперпозиции на первом кубите
    circuit.h(0);
    
    // Запутываем кубиты через CNOT (контролируемое НЕ)
    circuit.cnot(0, 1);
    circuit.cnot(1, 2);
    circuit.cnot(2, 3);
    circuit.cnot(3, 4);
    
    println!("Создана схема запутанных кубитов");
    
    // Выполняем схему
    println!("Выполнение квантовой схемы...");
    let result = simulator.run_circuit(&circuit).expect("Ошибка выполнения схемы");
    println!("Схема выполнена, результат: {:?}", result);
    
    // Получаем итоговое состояние
    let state = simulator.get_state();
    
    // Проверяем вероятности и запутанность
    println!("\nРезультаты симуляции:");
    
    for i in 0..5 {
        let qubit_state = state.get_qubit_state(i);
        println!("Кубит {}: |0⟩ = {:.4}, |1⟩ = {:.4}", 
                i, qubit_state.prob_zero(), qubit_state.prob_one());
    }
    
    // Проверяем запутанность между соседними кубитами
    println!("\nУровни запутанности:");
    for i in 0..4 {
        let entanglement = state.calculate_pair_entanglement(i, i+1);
        println!("Запутанность между кубитами {} и {}: {:.4}", i, i+1, entanglement);
    }
    
    println!("\nСимуляция успешно завершена");
} 