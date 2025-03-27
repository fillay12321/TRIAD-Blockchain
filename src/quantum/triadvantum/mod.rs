//! Модуль квантового симулятора TRIAD
//!
//! Реализует квантовое ядро TRIAD и обеспечивает симуляцию квантовых
//! состояний, схем и операторов.

pub mod state;
pub mod circuit;
pub mod gates;
pub mod operators;
pub mod simulator;
pub mod delta;
pub mod recovery;
pub mod interference;
pub mod qrng;
pub mod virtual_network;
pub mod triadvantum_adapter;

// Реэкспорт основных компонентов
pub use self::state::{QuantumState, QubitState};
pub use self::gates::QuantumGate;
pub use self::operators::QuantumOperator;
pub use self::circuit::QuantumCircuit;
pub use self::simulator::{QrustSimulator, SimulationResult, SimulationStats};
pub use self::delta::{QuantumDelta, DeltaCompressor};
pub use self::recovery::{RecoveryProtocol, RecoveryEvent, RecoveryEventType};
pub use self::interference::InterferencePattern;
pub use self::qrng::{QuantumRNG, RNGStats, QualityTestResult};
pub use self::virtual_network::VirtualNetwork;
pub use self::triadvantum_adapter::TriadVantumAdapter;

/// Версия библиотеки TRIAD
pub const VERSION: &str = "0.3.0";

/// Максимальное количество кубитов для стандартной симуляции
pub const MAX_QUBITS: usize = 24;

/// Максимальное количество узлов в сети
pub const MAX_NODES: usize = 16;

/// Максимальное число узлов в консенсусной группе
pub const MAX_CONSENSUS_NODES: usize = 7;

/// Инициализирует систему TRIAD Vantum
pub fn init(node_id: &str, qubit_count: usize) -> TriadVantumAdapter {
    // Создаем адаптер с указанным числом кубитов
    TriadVantumAdapter::new(node_id.to_string(), qubit_count)
}

/// Создает виртуальную сеть с заданным количеством узлов
pub fn create_network(node_count: usize, qubits_per_node: usize) -> Result<VirtualNetwork, String> {
    let mut network = VirtualNetwork::new();
    
    // Добавляем узлы
    for i in 0..node_count {
        let node_id = format!("node_{}", i);
        network.add_node(node_id.clone(), qubits_per_node)?;
    }
    
    // Соединяем узлы (полносвязная сеть)
    for i in 0..node_count {
        let from_id = format!("node_{}", i);
        for j in 0..node_count {
            if i != j {
                let to_id = format!("node_{}", j);
                network.connect_nodes(&from_id, &to_id)?;
            }
        }
    }
    
    Ok(network)
}

/// Создает квантовый генератор случайных чисел
pub fn create_qrng(qubit_count: usize) -> Result<QuantumRNG, String> {
    QuantumRNG::new(qubit_count)
}

// Запускаемый мини-пример для тестирования без сетевого взаимодействия
pub fn run_local_example(qubit_count: usize) {
    println!("Запуск локального примера TRIAD (без сетевого взаимодействия)");
    println!("Количество кубитов: {}", qubit_count);
    
    // Создаем симулятор
    let node_id = format!("local_node_{}", rand::random::<u32>());
    let simulator_result = QrustSimulator::new(node_id, qubit_count, true);
    
    if let Ok(mut simulator) = simulator_result {
        println!("Создан квантовый симулятор");
        
        // Создаем схему
        let mut circuit = QuantumCircuit::new(qubit_count);
        
        // Добавляем квантовые вентили
        println!("Создание квантовой схемы...");
        // Создаем состояние суперпозиции на всех кубитах
        for i in 0..qubit_count {
            circuit.h(i);
        }
        
        // Запутываем соседние кубиты
        for i in 0..qubit_count-1 {
            circuit.cnot(i, i+1);
        }
        
        // Выполняем схему
        println!("Выполнение квантовой схемы...");
        match simulator.run_circuit(&circuit) {
            Ok(_) => {
                println!("Схема успешно выполнена");
                
                // Получаем итоговое состояние
                let state = simulator.get_state();
                
                // Выводим результаты
                println!("\nРезультаты симуляции:");
                println!("Количество кубитов: {}", state.qubit_count);
                println!("Базисных состояний: {}", state.get_basis_states_count());
                
                // Проверяем состояния кубитов
                println!("\nСостояния кубитов:");
                for i in 0..qubit_count {
                    let qubit_state = state.get_qubit_state(i);
                    println!("Кубит {}: |0⟩ = {:.4}, |1⟩ = {:.4}", 
                            i, qubit_state.prob_zero(), qubit_state.prob_one());
                }
                
                // Проверяем запутанность
                println!("\nЗапутанность между кубитами:");
                for i in 0..qubit_count-1 {
                    let entanglement = state.calculate_pair_entanglement(i, i+1);
                    println!("Кубиты {} и {}: {:.4}", i, i+1, entanglement);
                }
            },
            Err(e) => {
                println!("Ошибка выполнения схемы: {}", e);
            }
        }
    } else {
        println!("Ошибка создания симулятора: {:?}", simulator_result.err());
    }
    
    println!("\nЛокальный пример успешно завершен");
} 