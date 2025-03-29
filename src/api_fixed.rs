//! # API библиотеки TRIAD
//! 
//! Этот модуль является центральной точкой входа для всей функциональности
//! библиотеки TRIAD. Он объединяет и реэкспортирует компоненты из всех 
//! подмодулей библиотеки для удобства использования.

/// Реэкспорт основных компонентов ядра квантовой симуляции
pub mod core {
    pub use crate::core::*;
}

/// Реэкспорт компонентов для квантовой симуляции на основе QuEST
pub mod quest {
    pub use crate::quest::QuESTSimulator;
    pub use crate::quest::ffi::SafeQuESTEnv;
}

/// Реэкспорт примеров и демонстраций квантовых алгоритмов
pub mod examples {
    pub use crate::examples::*;
}

// Реэкспорт наиболее часто используемых типов для удобства
pub use crate::core::{Qubit, QuantumState, QuantumSimulator, Amplitude};
pub use crate::quest::QuESTSimulator;

/// Структура для создания и управления квантовым симулятором.
/// Предоставляет упрощенный интерфейс для работы с квантовыми вычислениями.
pub struct QuantumEngine {
    /// Квантовый симулятор, выполняющий реальные операции.
    simulator: QuESTSimulator,
}

impl QuantumEngine {
    /// Создает новый экземпляр квантового движка с указанным числом кубитов.
    pub fn new(num_qubits: usize) -> Self {
        Self {
            simulator: QuESTSimulator::new(num_qubits),
        }
    }
    
    /// Возвращает ссылку на внутренний симулятор.
    pub fn simulator(&self) -> &QuESTSimulator {
        &self.simulator
    }
    
    /// Возвращает изменяемую ссылку на внутренний симулятор.
    pub fn simulator_mut(&mut self) -> &mut QuESTSimulator {
        &mut self.simulator
    }
    
    /// Применяет гейт Адамара к указанному кубиту.
    pub fn hadamard(&mut self, qubit: usize) {
        self.simulator.hadamard(qubit);
    }
    
    /// Применяет X-гейт (NOT) к указанному кубиту.
    pub fn x(&mut self, qubit: usize) {
        self.simulator.x(qubit);
    }
    
    /// Применяет Y-гейт к указанному кубиту.
    pub fn y(&mut self, qubit: usize) {
        self.simulator.y(qubit);
    }
    
    /// Применяет Z-гейт к указанному кубиту.
    pub fn z(&mut self, qubit: usize) {
        self.simulator.z(qubit);
    }
    
    /// Применяет CNOT-гейт между указанными кубитами.
    pub fn cnot(&mut self, control: usize, target: usize) {
        self.simulator.cnot(control, target);
    }
    
    /// Измеряет указанный кубит и возвращает результат.
    pub fn measure(&mut self, qubit: usize) -> bool {
        self.simulator.measure(qubit)
    }
    
    /// Сбрасывает состояние всех кубитов в |0⟩.
    pub fn reset(&mut self) {
        self.simulator.reset();
    }
    
    // === Методы для работы с суперпозицией ===
    
    /// Создаёт равную суперпозицию всех базисных состояний (|+⟩^⊗n)
    pub fn create_uniform_superposition(&mut self) {
        // Применяем гейт Адамара ко всем кубитам
        let num_qubits = self.simulator.get_state().num_qubits();
        for qubit in 0..num_qubits {
            self.hadamard(qubit);
        }
    }
    
    /// Создает параметризованную суперпозицию на указанном кубите,
    /// используя вращение на угол theta.
    pub fn create_parametrized_superposition(&mut self, qubit: usize, theta: f64) {
        // Сначала сбрасываем кубит в состояние |0⟩
        self.reset();
        
        // Применяем вращение вокруг оси Y
        // Так как у нас есть прямой доступ к QuESTSimulator, мы можем вызвать метод ry напрямую
        self.simulator.ry(qubit, 2.0 * theta);
    }
    
    /// Проверяет, находится ли кубит в суперпозиции
    pub fn is_in_superposition(&self, qubit: usize) -> bool {
        let prob_0 = self.simulator.probability_of_outcome(qubit, false);
        
        // Если вероятность близка к 0 или 1, то нет суперпозиции
        !(prob_0 < 1e-10 || prob_0 > 1.0 - 1e-10)
    }
    
    /// Вычисляет вероятность измерить 0 для указанного кубита.
    pub fn probability_of_zero(&self, qubit: usize) -> f64 {
        let prob_0 = self.simulator.probability_of_outcome(qubit, false);
        prob_0
    }
    
    // === Методы для работы с запутанностью ===
    
    /// Создаёт состояние Белла (максимально запутанное состояние двух кубитов)
    /// |Φ⁺⟩ = (|00⟩ + |11⟩)/√2
    pub fn create_bell_state(&mut self, qubit1: usize, qubit2: usize) -> Result<(), String> {
        if qubit1 == qubit2 {
            return Err("Кубиты должны быть различными".to_string());
        }
        
        // Создаём состояние Белла
        self.reset();
        self.hadamard(qubit1);
        self.cnot(qubit1, qubit2);
        
        Ok(())
    }
    
    /// Создаёт состояние GHZ (обобщение состояния Белла на n кубитов)
    /// |GHZ⟩ = (|0...0⟩ + |1...1⟩)/√2
    pub fn create_ghz_state(&mut self) -> Result<(), String> {
        let n = self.simulator.get_state().num_qubits();
        if n < 2 {
            return Err("Для создания GHZ-состояния требуется минимум 2 кубита".to_string());
        }
        
        // Создаём GHZ-состояние
        self.reset();
        self.hadamard(0);
        
        for i in 1..n {
            self.cnot(0, i);
        }
        
        Ok(())
    }
    
    /// Проверяет, запутаны ли два кубита
    /// (упрощенная реализация, основанная на измерениях)
    pub fn is_entangled(&self, qubit1: usize, qubit2: usize) -> bool {
        let n = self.simulator.get_state().num_qubits();
        
        if qubit1 >= n || qubit2 >= n || qubit1 == qubit2 {
            return false;
        }
        
        // Упрощенная проверка на запутанность:
        // Если кубиты в суперпозиции, но результаты измерений коррелированы,
        // то они, вероятно, запутаны
        if !self.is_in_superposition(qubit1) || !self.is_in_superposition(qubit2) {
            return false;
        }
        
        // Проверяем корреляцию через вероятность совместных состояний
        let state = self.simulator.get_state();
        let p00 = self.probability_of_state(0, 0, qubit1, qubit2);
        let p01 = self.probability_of_state(0, 1, qubit1, qubit2);
        let p10 = self.probability_of_state(1, 0, qubit1, qubit2);
        let p11 = self.probability_of_state(1, 1, qubit1, qubit2);
        
        // Для незапутанных состояний: p00*p11 = p01*p10
        let product_diff = (p00 * p11 - p01 * p10).abs();
        
        // Если разница значительна, то кубиты запутаны
        product_diff > 1e-6
    }
    
    /// Вычисляет вероятность совместного состояния двух кубитов
    fn probability_of_state(&self, val1: u8, val2: u8, qubit1: usize, qubit2: usize) -> f64 {
        use crate::core::quantum_simulator::QuantumSimulator;
        
        let state_box = self.simulator.get_state();
        let n = state_box.num_qubits();
        let mut total_prob = 0.0;
        
        // Суммируем вероятности по всем состояниям, где указанные кубиты имеют заданные значения
        for i in 0..(1 << n) {
            let bit1 = (i >> qubit1) & 1;
            let bit2 = (i >> qubit2) & 1;
            
            if bit1 == val1 as u64 && bit2 == val2 as u64 {
                total_prob += state_box.probability(i as u64);
            }
        }
        
        total_prob
    }
}

/// Функции для быстрого запуска квантовых алгоритмов
pub mod algorithms {
    use super::*;
    use crate::examples::{generate_random_number, run_deutsch_algorithm, FunctionType};
    use crate::examples::{quantum_teleportation, superdense_coding, grover_search};
    
    /// Генерирует квантовое случайное число заданной битовой длины.
    pub fn random_number(bits: usize) -> u64 {
        generate_random_number(bits) as u64
    }
    
    /// Запускает алгоритм Дойча для определения типа булевой функции.
    pub fn deutsch(function_type: FunctionType) -> bool {
        run_deutsch_algorithm(function_type)
    }
    
    /// Реализует алгоритм квантовой телепортации произвольного состояния.
    pub fn teleport_state(state: (f64, f64)) -> bool {
        quantum_teleportation(state)
    }
    
    /// Реализует сверхплотное кодирование (передача 2 классических битов через 1 кубит).
    pub fn dense_coding(bits: (bool, bool)) -> (bool, bool) {
        superdense_coding(bits)
    }
    
    /// Реализует алгоритм Гровера для поиска в неструктурированных данных.
    pub fn grover(num_qubits: usize, target_state: u64) -> u64 {
        grover_search(num_qubits, target_state)
    }
}

/// Модуль для визуализации квантовых состояний
pub mod visualization {
    use super::QuantumEngine;
    use crate::core::quantum_simulator::QuantumSimulator;
    
    /// Генерирует текстовое представление квантового состояния
    pub fn state_to_string(engine: &QuantumEngine) -> String {
        let state = engine.simulator().get_state();
        let n = state.num_qubits();
        let dim = 1 << n;
        
        let mut result = String::new();
        result.push_str("Квантовое состояние:\n");
        
        // Выводим только состояния с ненулевой амплитудой
        for i in 0..dim {
            let prob = state.probability(i as u64);
            if prob > 1e-10 {
                let binary = format!("{:0width$b}", i, width = n);
                result.push_str(&format!("  |{}⟩: {:6.3}%\n", binary, prob * 100.0));
            }
        }
        
        result
    }
    
    /// Визуализирует запутанность между кубитами в виде графа
    pub fn entanglement_graph(engine: &QuantumEngine) -> String {
        let n = engine.simulator().get_state().num_qubits();
        let mut result = String::new();
        
        result.push_str("Граф запутанности:\n");
        
        // Создаём простую матрицу смежности
        for i in 0..n {
            for j in (i+1)..n {
                if engine.is_entangled(i, j) {
                    result.push_str(&format!("  Кубит {} ⟷ Кубит {}\n", i, j));
                }
            }
        }
        
        if result.lines().count() <= 1 {
            result.push_str("  Запутанность не обнаружена.\n");
        }
        
        result
    }
}

use crate::core::quantum_simulator::{QuantumSimulator, AdvancedQuantumSimulator};
use crate::core::quantum_state::QuantumState;
use std::time::{Duration, Instant};
use std::f64::consts::PI;
use rand::Rng;

/// Тип для представления квантовых функций, используемых в структуре QFT.
pub type QuantumFunction = fn(usize, &mut QuantumEngine);

/// Представляет оракул в алгоритме Гровера.
pub enum Oracle {
    /// Отмечает единственное заданное значение `target_value`.
    SingleValue { target_value: u64 },
    /// Отмечает значения, удовлетворяющие пользовательской функции.
    CustomFunction { function: Box<dyn Fn(u64) -> bool>, max_value: u64 },
} 