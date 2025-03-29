//! Определения и реализации квантовых гейтов.
//! 
//! Этот модуль содержит структуры и перечисления для представления
//! квантовых гейтов и операций над ними.

use crate::core::quantum_state::Amplitude;
use std::f64::consts::PI;
use num_complex::Complex64;

/// Базовый интерфейс для квантового гейта.
pub trait Gate {
    /// Возвращает унитарную матрицу, представляющую гейт.
    fn matrix(&self) -> Vec<Amplitude>;
    
    /// Возвращает число кубитов, на которые действует гейт.
    fn num_qubits(&self) -> usize;
    
    /// Возвращает название гейта.
    fn name(&self) -> &'static str;
}

/// Перечисление базовых квантовых гейтов.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicGate {
    /// Гейт Адамара
    Hadamard,
    /// Гейт Паули-X (NOT)
    PauliX,
    /// Гейт Паули-Y
    PauliY,
    /// Гейт Паули-Z
    PauliZ,
    /// Фазовый гейт S
    S,
    /// Гейт T
    T,
}

impl Gate for BasicGate {
    fn matrix(&self) -> Vec<Amplitude> {
        match self {
            BasicGate::Hadamard => {
                let factor = 1.0 / 2.0_f64.sqrt();
                vec![
                    Complex64::new(factor, 0.0), Complex64::new(factor, 0.0),
                    Complex64::new(factor, 0.0), Complex64::new(-factor, 0.0),
                ]
            },
            BasicGate::PauliX => vec![
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ],
            BasicGate::PauliY => vec![
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
            ],
            BasicGate::PauliZ => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ],
            BasicGate::S => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
            ],
            BasicGate::T => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.5_f64.sqrt(), 0.5_f64.sqrt()),
            ],
        }
    }
    
    fn num_qubits(&self) -> usize {
        1 // Все базовые гейты действуют на 1 кубит
    }
    
    fn name(&self) -> &'static str {
        match self {
            BasicGate::Hadamard => "Hadamard",
            BasicGate::PauliX => "PauliX",
            BasicGate::PauliY => "PauliY",
            BasicGate::PauliZ => "PauliZ",
            BasicGate::S => "S",
            BasicGate::T => "T",
        }
    }
}

/// Гейт вращения вокруг осей X, Y или Z.
#[derive(Debug, Clone, PartialEq)]
pub struct RotationGate {
    /// Ось вращения (X, Y или Z).
    pub axis: Axis,
    /// Угол вращения в радианах.
    pub angle: f64,
}

/// Оси вращения для квантовых гейтов.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Gate for RotationGate {
    fn matrix(&self) -> Vec<Amplitude> {
        let cos_half = (self.angle / 2.0).cos();
        let sin_half = (self.angle / 2.0).sin();
        
        match self.axis {
            Axis::X => vec![
                Complex64::new(cos_half, 0.0), Complex64::new(0.0, -sin_half),
                Complex64::new(0.0, -sin_half), Complex64::new(cos_half, 0.0),
            ],
            Axis::Y => vec![
                Complex64::new(cos_half, 0.0), Complex64::new(-sin_half, 0.0),
                Complex64::new(sin_half, 0.0), Complex64::new(cos_half, 0.0),
            ],
            Axis::Z => vec![
                Complex64::new(cos_half, -sin_half), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(cos_half, sin_half),
            ],
        }
    }
    
    fn num_qubits(&self) -> usize {
        1
    }
    
    fn name(&self) -> &'static str {
        match self.axis {
            Axis::X => "RX",
            Axis::Y => "RY",
            Axis::Z => "RZ",
        }
    }
}

/// Двухкубитовый гейт (CNOT, CZ и т.д.).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoQubitGate {
    /// Контролируемый NOT (CNOT)
    CNOT,
    /// Контролируемый Z
    CZ,
    /// SWAP-гейт
    SWAP,
}

impl Gate for TwoQubitGate {
    fn matrix(&self) -> Vec<Amplitude> {
        match self {
            TwoQubitGate::CNOT => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ],
            TwoQubitGate::CZ => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ],
            TwoQubitGate::SWAP => vec![
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ],
        }
    }
    
    fn num_qubits(&self) -> usize {
        2
    }
    
    fn name(&self) -> &'static str {
        match self {
            TwoQubitGate::CNOT => "CNOT",
            TwoQubitGate::CZ => "CZ",
            TwoQubitGate::SWAP => "SWAP",
        }
    }
}

/// Создает пользовательский гейт с заданной унитарной матрицей.
#[derive(Debug, Clone)]
pub struct CustomGate {
    /// Унитарная матрица гейта.
    pub matrix: Vec<Amplitude>,
    /// Число кубитов, на которые действует гейт.
    pub num_qubits: usize,
    /// Название гейта.
    pub name: String,
}

impl Gate for CustomGate {
    fn matrix(&self) -> Vec<Amplitude> {
        self.matrix.clone()
    }
    
    fn num_qubits(&self) -> usize {
        self.num_qubits
    }
    
    fn name(&self) -> &'static str {
        // Это не совсем правильно, но другого варианта нет из-за ограничений интерфейса
        // Лучше было бы изменить интерфейс, чтобы он возвращал &str вместо &'static str
        "CustomGate"
    }
} 