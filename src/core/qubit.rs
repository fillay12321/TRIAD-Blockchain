//! Представление и операции с отдельными кубитами.
//! 
//! Этот модуль содержит типы для работы с отдельными кубитами
//! и их состояниями на низком уровне.

use crate::core::quantum_state::Amplitude;
use num_complex::Complex64;

/// Интерфейс для представления одиночного кубита.
/// Обеспечивает базовые операции с кубитом: установка состояния, измерение и т.д.
#[derive(Debug, Clone)]
pub struct Qubit {
    /// Уникальный идентификатор кубита.
    pub id: usize,
    /// Состояние кубита (если оно определено локально).
    /// Для распределенных кубитов состояние может быть доступно только через симулятор.
    pub state: Option<QubitState>,
}

impl Qubit {
    /// Создает новый кубит с заданным идентификатором.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: Some(QubitState::default()),
        }
    }
    
    /// Вероятность измерения кубита в состоянии |0⟩.
    pub fn prob_zero(&self) -> Option<f64> {
        self.state.as_ref().map(|s| s.prob_zero())
    }
    
    /// Вероятность измерения кубита в состоянии |1⟩.
    pub fn prob_one(&self) -> Option<f64> {
        self.state.as_ref().map(|s| s.prob_one())
    }
}

/// Представление квантового состояния одиночного кубита.
#[derive(Debug, Clone)]
pub struct QubitState {
    /// Амплитуда состояния |0⟩.
    pub alpha: Amplitude,
    /// Амплитуда состояния |1⟩.
    pub beta: Amplitude,
}

impl Default for QubitState {
    /// По умолчанию кубит инициализируется в состоянии |0⟩.
    fn default() -> Self {
        Self {
            alpha: Complex64::new(1.0, 0.0),
            beta: Complex64::new(0.0, 0.0),
        }
    }
}

impl QubitState {
    /// Создает новое состояние кубита с заданными амплитудами.
    pub fn new(alpha: Amplitude, beta: Amplitude) -> Self {
        let mut state = Self { alpha, beta };
        state.normalize();
        state
    }
    
    /// Нормализует состояние, обеспечивая |alpha|² + |beta|² = 1.
    pub fn normalize(&mut self) {
        let norm = (self.alpha.norm_sqr() + self.beta.norm_sqr()).sqrt();
        if norm > 0.0 {
            self.alpha /= norm;
            self.beta /= norm;
        }
    }
    
    /// Вероятность измерения кубита в состоянии |0⟩.
    pub fn prob_zero(&self) -> f64 {
        self.alpha.norm_sqr()
    }
    
    /// Вероятность измерения кубита в состоянии |1⟩.
    pub fn prob_one(&self) -> f64 {
        self.beta.norm_sqr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qubit_state_default() {
        let state = QubitState::default();
        assert_eq!(state.alpha, Complex64::new(1.0, 0.0));
        assert_eq!(state.beta, Complex64::new(0.0, 0.0));
        assert_eq!(state.prob_zero(), 1.0);
        assert_eq!(state.prob_one(), 0.0);
    }
    
    #[test]
    fn test_qubit_state_normalization() {
        let mut state = QubitState::new(
            Complex64::new(2.0, 0.0),
            Complex64::new(0.0, 2.0)
        );
        
        // Проверяем, что состояние нормализовано
        assert_eq!(state.prob_zero() + state.prob_one(), 1.0);
        
        // Ожидаемые нормализованные значения
        let expected_alpha = Complex64::new(2.0, 0.0) / (8.0_f64).sqrt();
        let expected_beta = Complex64::new(0.0, 2.0) / (8.0_f64).sqrt();
        
        assert!((state.alpha - expected_alpha).norm() < 1e-10);
        assert!((state.beta - expected_beta).norm() < 1e-10);
    }
} 