//! Примеры использования квантовых симуляторов и алгоритмов.
//! 
//! Этот модуль содержит различные примеры и демонстрации использования
//! TRIAD для квантовых вычислений и симуляции.

/// Пример генерации квантовых случайных чисел.
pub mod random_number;

/// Реализация алгоритма Дойча и Дойча-Йожи.
pub mod deutsch_algorithm;

/// Расширенные квантовые алгоритмы, демонстрирующие суперпозицию и запутанность
pub mod advanced_algorithms;

/// Публичный интерфейс для примеров.
pub use random_number::{demonstrate_random_number_generation, generate_random_number};
pub use deutsch_algorithm::{
    demonstrate_deutsch_algorithm, 
    demonstrate_deutsch_jozsa_algorithm,
    run_deutsch_algorithm,
    run_deutsch_jozsa_algorithm,
    FunctionType
};
pub use advanced_algorithms::{
    demonstrate_advanced_algorithms,
    quantum_teleportation,
    superdense_coding,
    grover_search
};

/// Запустить все демонстрационные примеры.
pub fn run_all_demos() {
    println!("=== TRIAD: Демонстрация квантовых алгоритмов ===\n");
    
    demonstrate_random_number_generation();
    println!("\n---\n");
    
    demonstrate_deutsch_algorithm();
    demonstrate_deutsch_jozsa_algorithm();
    println!("\n---\n");
    
    demonstrate_advanced_algorithms();
    
    println!("\n=== Демонстрация завершена ===");
}

/// Примеры использования API библиотеки TRIAD.
pub mod examples {
    use crate::api::QuantumEngine;
    
    /// Создает и запутывает два кубита (создание состояния Белла).
    pub fn create_bell_state() -> QuantumEngine {
        let mut engine = QuantumEngine::new(2);
        
        // Создаем состояние Белла |00⟩ + |11⟩ / √2
        engine.hadamard(0);
        engine.cnot(0, 1);
        
        engine
    }
    
    /// Квантовая телепортация состояния.
    pub fn quantum_teleportation() -> bool {
        // Создаем 3 кубита: источник, вспомогательный и целевой
        let mut engine = QuantumEngine::new(3);
        
        // Подготавливаем состояние для телепортации (например, |1⟩)
        engine.x(0);
        
        // Создаем запутанную пару между вспомогательным и целевым кубитами
        engine.hadamard(1);
        engine.cnot(1, 2);
        
        // Выполняем протокол телепортации
        engine.cnot(0, 1);
        engine.hadamard(0);
        
        // Измеряем первые два кубита
        let m0 = engine.measure(0);
        let m1 = engine.measure(1);
        
        // Применяем корректирующие операции в зависимости от результатов
        if m1 {
            engine.x(2);
        }
        
        if m0 {
            engine.z(2);
        }
        
        // Проверяем, что целевой кубит получил состояние |1⟩
        let result = engine.measure(2);
        result // Должно быть true
    }
} 