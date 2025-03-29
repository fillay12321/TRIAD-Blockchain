//! Интерфейсы и типы для работы с квантовыми состояниями.
//! 
//! Этот модуль предоставляет интерфейсы для представления 
//! квантовых состояний многокубитных систем.

use std::fmt;
use num_complex::Complex64;

/// Тип для представления квантовой амплитуды (комплексное число).
pub type Amplitude = Complex64;

/// Интерфейс для абстрактного квантового состояния, представляющего
/// систему из нескольких кубитов. Реализации могут использовать различные
/// стратегии хранения состояния в зависимости от бэкенда.
pub trait QuantumState: fmt::Debug {
    /// Получает число кубитов в системе.
    fn num_qubits(&self) -> usize;
    
    /// Получает вероятность измерения всех кубитов в заданном состоянии.
    /// Состояние задается битовой строкой, где каждый бит соответствует одному кубиту.
    fn probability(&self, state: u64) -> f64;
    
    /// Получает амплитуду для заданного базисного состояния.
    fn amplitude(&self, state: u64) -> Amplitude;
    
    /// Применяет унитарный оператор к состоянию.
    fn apply_operator(&mut self, operator: &[Amplitude]);
    
    /// Измеряет заданный кубит и возвращает результат (0 или 1).
    /// Состояние системы коллапсирует в соответствии с результатом измерения.
    fn measure(&mut self, qubit: usize) -> bool;
    
    /// Проверяет, запутано ли состояние (не является ли тензорным произведением).
    fn is_entangled(&self) -> bool;
}

/// Вспомогательные функции для работы с квантовыми состояниями.
pub mod utils {
    use super::*;
    
    /// Вычисляет тензорное произведение двух состояний.
    pub fn tensor_product(state1: &impl QuantumState, state2: &impl QuantumState) -> Vec<Amplitude> {
        let n1 = state1.num_qubits();
        let n2 = state2.num_qubits();
        
        let dim1 = 1 << n1;
        let dim2 = 1 << n2;
        let total_dim = dim1 * dim2;
        
        let mut result = Vec::with_capacity(total_dim);
        
        for i in 0..dim1 {
            let amp1 = state1.amplitude(i as u64);
            
            for j in 0..dim2 {
                let amp2 = state2.amplitude(j as u64);
                result.push(amp1 * amp2);
            }
        }
        
        result
    }
    
    /// Вычисляет вероятность нахождения состояния в указанной подсистеме.
    pub fn subsystem_probability(state: &impl QuantumState, qubits: &[usize], outcome: u64) -> f64 {
        let mut prob = 0.0;
        
        // Суммируем вероятности по всем состояниям, совпадающим с outcome на указанных кубитах
        let n = state.num_qubits();
        let total_states = 1u64 << n;
        
        for i in 0..total_states {
            // Проверяем, совпадает ли состояние i на указанных кубитах с outcome
            let mut matches = true;
            
            for (idx, &q) in qubits.iter().enumerate() {
                let bit_i = (i >> q) & 1;
                let bit_outcome = (outcome >> idx) & 1;
                
                if bit_i != bit_outcome {
                    matches = false;
                    break;
                }
            }
            
            if matches {
                prob += state.probability(i);
            }
        }
        
        prob
    }
} 