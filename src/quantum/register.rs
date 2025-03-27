use crate::quantum::Qubit;
use crate::quantum::qubit::QubitDelta;
use serde::{Deserialize, Serialize};

/// Представляет набор кубитов (квантовый регистр)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumRegister {
    /// Вектор кубитов в регистре
    qubits: Vec<Qubit>,
}

impl QuantumRegister {
    /// Создает новый квантовый регистр с заданным количеством кубитов
    pub fn new(qubit_count: usize) -> Self {
        let mut qubits = Vec::with_capacity(qubit_count);
        for _ in 0..qubit_count {
            qubits.push(Qubit::new());
        }
        Self { qubits }
    }

    /// Возвращает количество кубитов в регистре
    pub fn qubit_count(&self) -> usize {
        self.qubits.len()
    }

    /// Возвращает ссылку на кубит по индексу
    pub fn get_qubit(&self, index: usize) -> &Qubit {
        &self.qubits[index]
    }

    /// Возвращает изменяемую ссылку на кубит по индексу
    pub fn get_qubit_mut(&mut self, index: usize) -> &mut Qubit {
        &mut self.qubits[index]
    }

    /// Применяет вентиль Адамара к указанному кубиту
    pub fn apply_hadamard(&mut self, index: usize) {
        if index < self.qubits.len() {
            self.qubits[index].apply_hadamard();
        }
    }

    /// Применяет вентиль NOT (X) к указанному кубиту
    pub fn apply_x(&mut self, index: usize) {
        if index < self.qubits.len() {
            // Меняем местами амплитуды |0⟩ и |1⟩
            let qubit = self.get_qubit_mut(index);
            let temp = qubit.probability_zero();
            let alpha = qubit.probability_one().sqrt();
            let beta = temp.sqrt();
            *qubit = Qubit::new();
            qubit.apply_delta(&QubitDelta {
                delta_alpha: num_complex::Complex64::new(alpha, 0.0),
                delta_beta: num_complex::Complex64::new(beta, 0.0),
            });
        }
    }

    /// Получает дельту для указанного кубита между текущим и предыдущим состояниями
    pub fn get_qubit_delta(&self, index: usize, previous: &Self) -> QubitDelta {
        if index < self.qubits.len() && index < previous.qubits.len() {
            self.qubits[index].get_delta(&previous.qubits[index])
        } else {
            QubitDelta {
                delta_alpha: num_complex::Complex64::new(0.0, 0.0),
                delta_beta: num_complex::Complex64::new(0.0, 0.0),
            }
        }
    }

    /// Применяет дельту к указанному кубиту
    pub fn apply_delta_to_qubit(&mut self, index: usize, delta: &QubitDelta) {
        if index < self.qubits.len() {
            self.qubits[index].apply_delta(delta);
        }
    }

    /// Вычисляет "волну" для консенсуса на основе всего регистра
    /// Возвращает значение от -1.0 до 1.0
    pub fn calculate_consensus_wave(&self) -> f64 {
        let mut total_zero = 0.0;
        let mut total_one = 0.0;
        
        for qubit in &self.qubits {
            total_zero += qubit.probability_zero();
            total_one += qubit.probability_one();
        }
        
        let avg_zero = total_zero / self.qubits.len() as f64;
        let avg_one = total_one / self.qubits.len() as f64;
        
        // Возвращаем разницу вероятностей как "волну"
        avg_zero - avg_one
    }
} 