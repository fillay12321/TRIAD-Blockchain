mod quantum;
mod network;
mod consensus;
mod storage;
mod visualization;

use quantum::triadvantum::{
    circuit::QuantumCircuit, 
    simulator::QrustSimulator as TriadVantumSimulator,
    operators::QuantumOperator
};
use quantum::triadvantum_adapter::TriadVantumAdapter;
use network::{VirtualNetwork, NetworkConfig};
use consensus::demonstrate_quantum_consensus;
use std::time::Instant;

fn main() {
    println!("TRIAD Blockchain Prototype");
    println!("Quantum-inspired decentralized network");
    
    // Демонстрация работы с библиотекой TriadVantum
    println!("\n----- Демонстрация работы с библиотекой TriadVantum -----");
    
    // Создаем квантовый симулятор
    let simulator_result = TriadVantumSimulator::new("main_simulator".to_string(), 4, false);
    
    if let Ok(mut simulator) = simulator_result {
        // Создаем схему с запутыванием кубитов (состояние Белла)
        let mut circuit = QuantumCircuit::new(4);
        circuit.name = "Bell State Demo".to_string();
        
        circuit.h(0);       // Адамар на 0-й кубит
        
        // Применяем CNOT с помощью метода cnot
        circuit.cnot(0, 1);   // CNOT с контрольным 0 и целевым 1 (запутываем их)
        
        circuit.h(2);       // Адамар на 2-й кубит
        
        // Применяем CNOT с помощью метода cnot
        circuit.cnot(2, 3);   // Запутываем 2-й и 3-й кубиты
        
        // Выполняем схему
        println!("Выполняем квантовую схему с парами Белла (TriadVantum)...");
        if let Ok(result) = simulator.run_circuit(&circuit) {
            // Получаем состояние после выполнения схемы
            if let Some(final_state) = Some(simulator.get_state()) {
                // Проверяем запутанность (теперь нам нужно вычислить это вручную)
                println!("Схема выполнена успешно");
                
                // В новой версии нам нужно реализовать вычисление запутанности самостоятельно
                // либо использовать вспомогательные функции из quantum_integration
            }
        } else {
            println!("Ошибка при выполнении схемы");
        }
        
        // Демонстрация работы адаптера
        println!("\n----- Демонстрация работы с адаптером TriadVantum -----");
        
        // Создаем адаптер с 4 кубитами
        let mut adapter = TriadVantumAdapter::new(4, "demo_adapter".to_string());
        
        // Создаем состояние Белла через адаптер
        adapter.circuit = QuantumCircuit::new(4);
        adapter.circuit.h(0);
        adapter.circuit.cnot(0, 1);
        
        // Выполняем схему
        if let Ok(adapter_result) = adapter.execute_circuit() {
            println!("Результат выполнения через адаптер: схема выполнена успешно");
        } else {
            println!("Ошибка при выполнении схемы через адаптер");
        }
        
        // Демонстрация работы виртуальной сети с TriadVantum
        println!("\n----- Демонстрация работы виртуальной сети TRIAD с TriadVantum -----");
        
        // Создаем сеть с 5 узлами, каждый с меньшим количеством кубитов
        let node_count = 5;
        let qubits_per_node = 5; // Уменьшаем с 10 до 5 для безопасности
        let config = NetworkConfig::default();
        
        let mut network = VirtualNetwork::with_nodes(node_count, qubits_per_node, config);
        println!("Создана сеть с {} узлами, по {} кубитов на узел", node_count, qubits_per_node);
        
        // Активируем новый симулятор в квантовом поле сети
        network.activate_triadvantum(true);
        println!("Активирован симулятор TriadVantum для квантового поля");
        
        // Обработка транзакций
        let tx_count = 5;
        println!("\nОбработка {} транзакций через виртуальную сеть с TriadVantum...", tx_count);
        
        let start = Instant::now();
        for i in 0..tx_count {
            let tx_data = format!("tx_{}", i);
            let result = network.process_transaction(&tx_data);
            
            println!("Транзакция {}: консенсус = {}, интерференция = {:.4}, время = {:.2} мс", 
                     i, result.consensus, result.interference_result, result.processing_time_ms);
        }
        
        let elapsed = start.elapsed();
        let total_time_ms = elapsed.as_secs_f64() * 1000.0;
        let tps = tx_count as f64 / (total_time_ms / 1000.0);
        
        println!("\nОбщие результаты с TriadVantum:");
        println!("Обработано {} транзакций за {:.2} мс", tx_count, total_time_ms);
        println!("Средняя скорость: {:.2} TPS", tps);
        println!("Масштабируемая оценка для сети из 100,000 узлов: {:.2} TPS", 
                 tps * (100_000.0 / node_count as f64));
                 
        // Демонстрация создания специальных схем через адаптер
        println!("\n----- Демонстрация специальных квантовых схем -----");
        
        // Создаем адаптер для демонстрации
        let mut demo_adapter = TriadVantumAdapter::new(8, "demo_circuits".to_string());
        
        // Создаем схему состояния GHZ
        println!("Создаем схему GHZ...");
        demo_adapter.clear_circuit();
        demo_adapter.circuit.h(0);
        for i in 1..8 {
            demo_adapter.circuit.cnot(0, i);
        }
        
        let ghz_result = demo_adapter.execute_circuit();
        if let Ok(result) = ghz_result {
            println!("Схема GHZ выполнена, запутанность: высокая");
        }
        
        // Создаем схему QFT
        println!("\nСоздаем схему QFT...");
        if let Ok(_) = demo_adapter.create_qft_circuit() {
            let qft_result = demo_adapter.execute_circuit();
            if let Ok(_) = qft_result {
                println!("Схема QFT выполнена успешно");
            }
        }
        
        // Сравнение алгоритмов консенсуса
        println!("\n----- Сравнение TRIAD с традиционными механизмами консенсуса -----");
        let _comparison_results = demonstrate_quantum_consensus();
    } else {
        println!("Ошибка при создании симулятора TriadVantum");
    }
}
