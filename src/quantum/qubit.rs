use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::FRAC_1_SQRT_2;

/// Представляет один кубит в симуляции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qubit {
    /// Амплитуда состояния |0⟩
    alpha: Complex64,
    /// Амплитуда состояния |1⟩
    beta: Complex64,
}

impl Qubit {
    /// Создает новый кубит в состоянии |0⟩
    pub fn new() -> Self {
        Self {
            alpha: Complex64::new(1.0, 0.0),
            beta: Complex64::new(0.0, 0.0),
        }
    }

    /// Создает кубит с заданными амплитудами вероятности
    pub fn from_amplitudes(alpha: Complex64, beta: Complex64) -> Self {
        // Нормализуем амплитуды, чтобы |alpha|^2 + |beta|^2 = 1
        let norm = (alpha.norm_sqr() + beta.norm_sqr()).sqrt();
        let alpha_norm = alpha / norm;
        let beta_norm = beta / norm;
        
        Self {
            alpha: alpha_norm,
            beta: beta_norm,
        }
    }

    /// Создает новый кубит в состоянии |1⟩
    pub fn one() -> Self {
        Self {
            alpha: Complex64::new(0.0, 0.0),
            beta: Complex64::new(1.0, 0.0),
        }
    }

    /// Создает кубит в состоянии суперпозиции |+⟩ = (|0⟩+|1⟩)/√2
    pub fn plus() -> Self {
        Self {
            alpha: Complex64::new(FRAC_1_SQRT_2, 0.0),
            beta: Complex64::new(FRAC_1_SQRT_2, 0.0),
        }
    }

    /// Применяет вентиль Адамара для создания суперпозиции
    pub fn apply_hadamard(&mut self) {
        let new_alpha = (self.alpha + self.beta) * FRAC_1_SQRT_2;
        let new_beta = (self.alpha - self.beta) * FRAC_1_SQRT_2;
        self.alpha = new_alpha;
        self.beta = new_beta;
    }

    /// Вычисляет и возвращает дельту между текущим и предыдущим состояниями
    pub fn get_delta(&self, previous: &Self) -> QubitDelta {
        QubitDelta {
            delta_alpha: self.alpha - previous.alpha,
            delta_beta: self.beta - previous.beta,
        }
    }

    /// Применяет дельту к состоянию кубита
    pub fn apply_delta(&mut self, delta: &QubitDelta) {
        self.alpha += delta.delta_alpha;
        self.beta += delta.delta_beta;
        self.normalize(); // Сохраняем нормализацию
    }

    /// Нормализация состояния кубита (|α|² + |β|² = 1)
    pub fn normalize(&mut self) {
        let norm = (self.alpha.norm_sqr() + self.beta.norm_sqr()).sqrt();
        if norm > 1e-10 {  // Избегаем деления на очень маленькие числа
            self.alpha /= norm;
            self.beta /= norm;
        }
    }

    /// Вероятность состояния |0⟩
    pub fn probability_zero(&self) -> f64 {
        self.alpha.norm_sqr()
    }

    /// Вероятность состояния |1⟩
    pub fn probability_one(&self) -> f64 {
        self.beta.norm_sqr()
    }

    /// Возвращает амплитуду состояния |0⟩
    pub fn alpha(&self) -> Complex64 {
        self.alpha
    }
    
    /// Возвращает амплитуду состояния |1⟩
    pub fn beta(&self) -> Complex64 {
        self.beta
    }
}

/// Разница между двумя состояниями кубита для эффективной передачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QubitDelta {
    pub delta_alpha: Complex64,
    pub delta_beta: Complex64,
}
