//! # Алгоритм Дойча
//! 
//! Этот пример демонстрирует реализацию алгоритма Дойча с использованием квантового симулятора TRIAD.
//! Алгоритм определяет, является ли бинарная функция константной или сбалансированной за один запрос.

use log::{info, debug};
use std::fmt;

use crate::core::gates::{Gate, BasicGate, TwoQubitGate};
use crate::core::quantum_simulator::QuantumSimulator;
use crate::quest::QuESTSimulator;

/// Тип квантового оракула для функции f(x).
#[derive(Debug, Clone)]
enum OracleType {
    /// Константная функция f(x) = 0 для всех x
    Constant0,
    /// Константная функция f(x) = 1 для всех x
    Constant1,
    /// Сбалансированная функция f(x) = x
    Identity,
    /// Сбалансированная функция f(x) = !x
    Negation,
}

impl fmt::Display for OracleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OracleType::Constant0 => write!(f, "Constant f(x) = 0"),
            OracleType::Constant1 => write!(f, "Constant f(x) = 1"),
            OracleType::Identity => write!(f, "Balanced f(x) = x"),
            OracleType::Negation => write!(f, "Balanced f(x) = !x"),
        }
    }
}

/// Реализует алгоритм Дойча для определения свойств функции f через оракул.
/// 
/// Возвращает true, если функция константная (f(0) = f(1)), и false, если функция сбалансированная.
fn deutsch_algorithm(oracle_type: OracleType) -> bool {
    // Создаем квантовый симулятор с 2 кубитами
    let mut simulator = QuESTSimulator::new(2);
    
    // Подготавливаем состояние |01⟩
    simulator.reset(); // Сначала сбрасываем кубиты в |00⟩
    simulator.x(1);    // Применяем X к кубиту 1, получаем |01⟩
    
    // Применяем вентиль Адамара к обоим кубитам
    simulator.hadamard(0);
    simulator.hadamard(1);
    
    // Применяем оракул (Uf) на основе типа функции
    match oracle_type {
        OracleType::Constant0 => {
            // Для f(x) = 0 не требуется дополнительных операций
            // Uf: |x,y⟩ -> |x,y⟩
        },
        OracleType::Constant1 => {
            // Для f(x) = 1 применяем X к выходному кубиту
            // Uf: |x,y⟩ -> |x,y⊕1⟩
            simulator.x(1);
        },
        OracleType::Identity => {
            // Для f(x) = x, применяем CNOT
            // Uf: |x,y⟩ -> |x,y⊕x⟩
            simulator.cnot(0, 1);
        },
        OracleType::Negation => {
            // Для f(x) = !x, инвертируем контрольный кубит, затем CNOT, затем снова инвертируем
            // Uf: |x,y⟩ -> |x,y⊕(1-x)⟩ = |x,y⊕1⊕x⟩
            simulator.x(0);
            simulator.cnot(0, 1);
            simulator.x(0);
        },
    }
    
    // Применяем Адамара к первому кубиту
    simulator.hadamard(0);
    
    // Измеряем первый кубит для определения результата
    let result = simulator.measure(0);
    
    // Если результат 0, функция константная; если 1, то сбалансированная
    !result
}

/// Точка входа в программу
pub fn run() {
    info!("Запуск алгоритма Дойча");
    
    // Тестируем все четыре типа оракулов
    let oracles = [
        OracleType::Constant0,
        OracleType::Constant1,
        OracleType::Identity,
        OracleType::Negation,
    ];
    
    for oracle in &oracles {
        let is_constant = deutsch_algorithm(oracle.clone());
        info!(
            "Оракул: {} -> Функция {}", 
            oracle,
            if is_constant { "константная" } else { "сбалансированная" }
        );
    }
    
    info!("Алгоритм Дойча успешно завершен");
}

// Эти функции для обратной совместимости с примером
pub fn demonstrate_deutsch_algorithm() {
    run();
}

pub fn demonstrate_deutsch_jozsa_algorithm() {
    // Пустая заглушка для совместимости
    println!("Алгоритм Дойча-Йожи не реализован");
}

pub fn run_deutsch_algorithm(function_type: FunctionType) -> bool {
    match function_type {
        FunctionType::Constant0 => deutsch_algorithm(OracleType::Constant0),
        FunctionType::Constant1 => deutsch_algorithm(OracleType::Constant1),
        FunctionType::Identity => deutsch_algorithm(OracleType::Identity),
        FunctionType::Negation => deutsch_algorithm(OracleType::Negation),
    }
}

pub fn run_deutsch_jozsa_algorithm(_n: usize, is_constant: bool) -> bool {
    // Заглушка для совместимости
    is_constant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionType {
    Constant0,
    Constant1,
    Identity,
    Negation,
} 