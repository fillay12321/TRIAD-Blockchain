//! Интерфейсы для квантовых симуляторов.
//! 
//! Этот модуль предоставляет базовый интерфейс для
//! симуляции квантовых вычислений.

use crate::core::quantum_state::QuantumState;
use crate::core::gates::Gate;

/// Интерфейс для квантового симулятора, способного выполнять квантовые операции.
pub trait QuantumSimulator {
    /// Создает новый симулятор с заданным числом кубитов.
    fn new(num_qubits: usize) -> Self where Self: Sized;
    
    /// Получает текущее квантовое состояние.
    fn get_state(&self) -> Box<dyn QuantumState>;
    
    /// Устанавливает все кубиты в состояние |0⟩.
    fn reset(&mut self);
    
    /// Применяет гейт Адамара к заданному кубиту.
    fn hadamard(&mut self, qubit: usize);
    
    /// Применяет X-гейт (NOT) к заданному кубиту.
    fn x(&mut self, qubit: usize);
    
    /// Применяет Y-гейт к заданному кубиту.
    fn y(&mut self, qubit: usize);
    
    /// Применяет Z-гейт к заданному кубиту.
    fn z(&mut self, qubit: usize);
    
    /// Применяет CNOT-гейт с контрольным и целевым кубитами.
    fn cnot(&mut self, control: usize, target: usize);
    
    /// Измеряет заданный кубит и возвращает результат (0 или 1).
    fn measure(&mut self, qubit: usize) -> bool;
    
    /// Применяет произвольный квантовый гейт.
    fn apply_gate(&mut self, gate: &impl Gate);
    
    /// Применяет гейт вращения.
    fn apply_rotation(&mut self, gate: &impl Gate);
    
    /// Вычисляет ожидаемое значение для произвольного набора операторов Паули.
    fn get_expectation_value(&self, pauli_product: &[(usize, char)]) -> f64;
    
    /// Вычисляет вероятность получения указанного результата при измерении кубита.
    fn probability_of_outcome(&self, qubit: usize, outcome: bool) -> f64 {
        // Значение по умолчанию, конкретные реализации могут переопределить этот метод
        let state = self.get_state();
        let mask = 1u64 << qubit;
        let mut prob = 0.0;
        
        for i in 0..(1u64 << state.num_qubits()) {
            let bit_is_set = (i & mask) != 0;
            if bit_is_set == outcome {
                prob += state.probability(i);
            }
        }
        
        prob
    }
}

/// Расширенный интерфейс для более продвинутых квантовых симуляторов.
/// Предоставляет дополнительные операции сверх базового интерфейса.
pub trait AdvancedQuantumSimulator: QuantumSimulator {
    /// Применяет фазовый S-гейт к заданному кубиту.
    fn s_gate(&mut self, qubit: usize);
    
    /// Применяет T-гейт к заданному кубиту.
    fn t_gate(&mut self, qubit: usize);
    
    /// Применяет контролируемый Z-гейт между указанными кубитами.
    fn cz(&mut self, control: usize, target: usize);
    
    /// Применяет SWAP-гейт, обменивающий состояния двух кубитов.
    fn swap(&mut self, qubit1: usize, qubit2: usize);
    
    /// Применяет произвольный однокубитовый гейт, заданный матрицей 2x2.
    fn apply_unitary(&mut self, qubit: usize, matrix: &[crate::core::quantum_state::Amplitude]);
    
    /// Применяет гейт вращения вокруг оси X на указанный угол (в радианах).
    fn rx(&mut self, qubit: usize, angle: f64);
    
    /// Применяет гейт вращения вокруг оси Y на указанный угол (в радианах).
    fn ry(&mut self, qubit: usize, angle: f64);
    
    /// Применяет гейт вращения вокруг оси Z на указанный угол (в радианах).
    fn rz(&mut self, qubit: usize, angle: f64);
    
    /// Применяет контролируемый унитарный гейт между указанными кубитами.
    fn controlled_unitary(&mut self, control: usize, target: usize, matrix: &[crate::core::quantum_state::Amplitude]);
} 