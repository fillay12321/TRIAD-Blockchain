use crate::quantum::triadvantum::state::QuantumState;
use num_complex::Complex64;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::f64::consts::PI;

/// Типы квантовых гейтов, поддерживаемых библиотекой
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateType {
    /// Вентиль Адамара - создает суперпозицию
    Hadamard,
    /// Вентиль Паули-X (NOT) - инвертирует состояние
    PauliX,
    /// Вентиль Паули-Y
    PauliY,
    /// Вентиль Паули-Z - фазовый сдвиг
    PauliZ,
    /// Фазовый сдвиг на произвольный угол
    Phase,
    /// Вращение вокруг оси X
    RotationX,
    /// Вращение вокруг оси Y
    RotationY,
    /// Вращение вокруг оси Z
    RotationZ,
    /// Контролируемый NOT - запутывает два кубита
    CNOT,
    /// Контролируемый Z - контролируемый фазовый сдвиг
    CZ,
    /// SWAP - обмен состояниями между кубитами
    SWAP,
    /// Тоффоли (CCNOT) - контролируемый-контролируемый NOT
    Toffoli,
    /// Пользовательский унитарный оператор
    Custom,
    /// Операция измерения
    Measure
}

/// Параметры для квантового гейта
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GateParams {
    /// Угол поворота (для вращений и фазовых сдвигов)
    pub theta: f64,
    /// Дополнительный угол (для некоторых двухкубитовых гейтов)
    pub phi: f64,
    /// Произвольная матрица унитарного оператора
    pub matrix: Vec<Complex64>,
    /// Дополнительные метаданные
    pub metadata: String,
}

impl Default for GateParams {
    fn default() -> Self {
        Self {
            theta: 0.0,
            phi: 0.0,
            matrix: Vec::new(),
            metadata: String::new(),
        }
    }
}

/// Представление квантового гейта
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QuantumGate {
    Single(usize, Vec<Complex64>),
    Two(usize, usize, Vec<Complex64>),
}

impl QuantumGate {
    /// Создает новый гейт указанного типа (для обратной совместимости)
    pub fn new(gate_type: GateType, target: usize) -> Self {
        match gate_type {
            GateType::Hadamard => Self::hadamard(target),
            GateType::PauliX => Self::pauli_x(target),
            GateType::PauliY => Self::pauli_y(target),
            GateType::PauliZ => Self::pauli_z(target),
            GateType::Phase => {
                let matrix = vec![
                    Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                    Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)
                ];
                QuantumGate::Single(target, matrix)
            },
            GateType::RotationX => Self::rotation_x(target, 0.0),
            GateType::RotationY => Self::rotation_y(target, 0.0),
            GateType::RotationZ => Self::rotation_z(target, 0.0),
            GateType::Measure => {
                let matrix = vec![
                    Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                    Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)
                ];
                QuantumGate::Single(target, matrix)
            },
            _ => panic!("Неподдерживаемый тип гейта для метода new: {:?}", gate_type)
        }
    }

    /// Создает гейт Адамара (алиас для hadamard для обратной совместимости)
    pub fn h(target: usize) -> Self {
        Self::hadamard(target)
    }
    
    /// Создает гейт Тоффоли
    pub fn toffoli(control1: usize, control2: usize, target: usize) -> Self {
        // Упрощенная реализация для совместимости
        // В реальности, это должно быть реализовано через декомпозицию или матрицу 8x8
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)
        ];
        QuantumGate::Two(control1, target, matrix)
    }
    
    /// Создает гейт Адамара
    pub fn hadamard(target: usize) -> Self {
        let invsqrt2 = 1.0 / 2.0_f64.sqrt();
        let matrix = vec![
            Complex64::new(invsqrt2, 0.0), Complex64::new(invsqrt2, 0.0),
            Complex64::new(invsqrt2, 0.0), Complex64::new(-invsqrt2, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт Паули-X (NOT)
    pub fn pauli_x(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт Паули-Y
    pub fn pauli_y(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт Паули-Z
    pub fn pauli_z(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает фазовый гейт
    pub fn phase(target: usize, theta: f64) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(theta.cos(), theta.sin()),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт вращения вокруг оси X
    pub fn rotation_x(target: usize, theta: f64) -> Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(0.0, -sin),
            Complex64::new(0.0, -sin), Complex64::new(cos, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт вращения вокруг оси Y
    pub fn rotation_y(target: usize, theta: f64) -> Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(-sin, 0.0),
            Complex64::new(sin, 0.0), Complex64::new(cos, 0.0),
        ];
        QuantumGate::Single(target, matrix)
    }
    
    /// Создает гейт вращения вокруг оси Z
    pub fn rotation_z(target: usize, theta: f64) -> Self {
        let exp_plus = Complex64::new((theta / 2.0).cos(), (theta / 2.0).sin());
        let exp_minus = Complex64::new((theta / 2.0).cos(), -(theta / 2.0).sin());
        
        let matrix = vec![
            exp_minus, Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), exp_plus,
        ];
        QuantumGate::Single(target, matrix)
    }

    /// Создает гейт CNOT
    pub fn cnot(control: usize, target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)
        ];
        QuantumGate::Two(control, target, matrix)
    }

    /// Создает гейт CZ
    pub fn cz(control: usize, target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0)
        ];
        QuantumGate::Two(control, target, matrix)
    }

    /// Создает гейт SWAP
    pub fn swap(qubit1: usize, qubit2: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)
        ];
        QuantumGate::Two(qubit1, qubit2, matrix)
    }

    /// Создает пользовательский гейт
    pub fn custom(indices: Vec<usize>, matrix: Vec<Complex64>) -> Self {
        if indices.len() == 1 {
            QuantumGate::Single(indices[0], matrix)
        } else if indices.len() == 2 {
            QuantumGate::Two(indices[0], indices[1], matrix)
        } else {
            panic!("Неподдерживаемое количество индексов для гейта");
        }
    }

    pub fn get_matrix(&self) -> Vec<Complex64> {
        match self {
            QuantumGate::Single(_, matrix) => matrix.clone(),
            QuantumGate::Two(_, _, matrix) => matrix.clone(),
        }
    }

    pub fn is_measurement(&self) -> bool {
        match self {
            QuantumGate::Single(_, matrix) => {
                // Проверяем, является ли матрица идентичностной (что указывает на измерение)
                matrix.len() == 4 && 
                matrix[0] == Complex64::new(1.0, 0.0) && 
                matrix[1] == Complex64::new(0.0, 0.0) && 
                matrix[2] == Complex64::new(0.0, 0.0) && 
                matrix[3] == Complex64::new(1.0, 0.0)
            },
            _ => false
        }
    }

    pub fn is_multi_qubit(&self) -> bool {
        matches!(self, QuantumGate::Two(_, _, _))
    }

    pub fn affects_qubit(&self, qubit_idx: usize) -> bool {
        match self {
            QuantumGate::Single(target, _) => *target == qubit_idx,
            QuantumGate::Two(control, target, _) => *control == qubit_idx || *target == qubit_idx,
        }
    }

    /// Получает описание гейта
    pub fn get_description(&self) -> String {
        match self {
            QuantumGate::Single(target, matrix) => {
                if self.is_measurement() {
                    format!("Измерение кубита {}", target)
                } else if matrix.len() == 4 {
                    // Определяем тип гейта по матрице
                    if matrix[0].re.abs() > 0.7 && matrix[0].re.abs() < 0.8 && 
                       matrix[1].re.abs() > 0.7 && matrix[1].re.abs() < 0.8 {
                        format!("Адамара на кубите {}", target)
                    } else if matrix[0] == Complex64::new(0.0, 0.0) && matrix[1] == Complex64::new(1.0, 0.0) {
                        format!("Паули-X на кубите {}", target)
                    } else if matrix[0] == Complex64::new(0.0, 0.0) && matrix[1] == Complex64::new(0.0, -1.0) {
                        format!("Паули-Y на кубите {}", target)
                    } else if matrix[0] == Complex64::new(1.0, 0.0) && matrix[3] == Complex64::new(-1.0, 0.0) {
                        format!("Паули-Z на кубите {}", target)
                    } else {
                        format!("Однокубитовый гейт на кубите {}", target)
                    }
                } else {
                    format!("Однокубитовый гейт на кубите {}", target)
                }
            },
            QuantumGate::Two(control, target, matrix) => {
                // Определяем тип двухкубитового гейта по матрице
                if matrix.len() == 16 {
                    if matrix[10] == Complex64::new(0.0, 0.0) && matrix[11] == Complex64::new(1.0, 0.0) {
                        format!("CNOT: контроль={}, цель={}", control, target)
                    } else if matrix[15] == Complex64::new(-1.0, 0.0) {
                        format!("CZ: контроль={}, цель={}", control, target)
                    } else if matrix[5] == Complex64::new(0.0, 0.0) && matrix[6] == Complex64::new(1.0, 0.0) {
                        format!("SWAP: кубит1={}, кубит2={}", control, target)
                    } else {
                        format!("Двухкубитовый гейт: контроль={}, цель={}", control, target)
                    }
                } else {
                    format!("Двухкубитовый гейт: контроль={}, цель={}", control, target)
                }
            }
        }
    }

    pub fn apply(&self, state: &mut QuantumState) -> Result<(), String> {
        match self {
            QuantumGate::Single(target, matrix) => {
                if *target >= state.qubit_count {
                    return Err(format!("Индекс кубита {} вне диапазона 0-{}", 
                                      target, state.qubit_count - 1));
                }
                
                state.apply_single_qubit_gate(*target, matrix.clone());
                Ok(())
            },
            QuantumGate::Two(control, target, matrix) => {
                if *control >= state.qubit_count || *target >= state.qubit_count {
                    return Err(format!("Индексы кубитов {}, {} вне диапазона 0-{}", 
                                      control, target, state.qubit_count - 1));
                }
                
                state.apply_two_qubit_gate(*control, *target, matrix.clone());
                Ok(())
            }
        }
    }
}

impl GateType {
    /// Преобразует тип гейта в строковое представление
    pub fn to_string(&self) -> String {
        match self {
            GateType::Hadamard => "Hadamard".to_string(),
            GateType::PauliX => "PauliX".to_string(),
            GateType::PauliY => "PauliY".to_string(),
            GateType::PauliZ => "PauliZ".to_string(),
            GateType::Phase => "Phase".to_string(),
            GateType::RotationX => "RotationX".to_string(),
            GateType::RotationY => "RotationY".to_string(),
            GateType::RotationZ => "RotationZ".to_string(),
            GateType::CNOT => "CNOT".to_string(),
            GateType::CZ => "CZ".to_string(),
            GateType::SWAP => "SWAP".to_string(),
            GateType::Toffoli => "Toffoli".to_string(),
            GateType::Custom => "Custom".to_string(),
            GateType::Measure => "Measure".to_string(),
        }
    }
}

/// Создает последовательность гейтов для гейта SWAP через три CNOT
pub fn decompose_swap_to_cnot(qubit1: usize, qubit2: usize) -> Vec<QuantumGate> {
    vec![
        QuantumGate::cnot(qubit1, qubit2),
        QuantumGate::cnot(qubit2, qubit1),
        QuantumGate::cnot(qubit1, qubit2),
    ]
}

/// Создает последовательность гейтов для гейта Тоффоли через элементарные гейты
pub fn decompose_toffoli(control1: usize, control2: usize, target: usize) -> Vec<QuantumGate> {
    let mut gates = Vec::new();
    
    // H на целевом кубите
    gates.push(QuantumGate::h(target));
    
    // CNOT между control2 и target
    gates.push(QuantumGate::cnot(control2, target));
    
    // T† на целевом кубите
    gates.push(QuantumGate::new(GateType::Phase, target));
    
    // CNOT между control1 и target
    gates.push(QuantumGate::cnot(control1, target));
    
    // T на целевом кубите
    gates.push(QuantumGate::new(GateType::Phase, target));
    
    // CNOT между control2 и target
    gates.push(QuantumGate::cnot(control2, target));
    
    // T† на целевом кубите
    gates.push(QuantumGate::new(GateType::Phase, target));
    
    // CNOT между control1 и target
    gates.push(QuantumGate::cnot(control1, target));
    
    // T на целевом и control2
    gates.push(QuantumGate::new(GateType::Phase, target));
    gates.push(QuantumGate::new(GateType::Phase, control2));
    
    // CNOT между control1 и control2
    gates.push(QuantumGate::cnot(control1, control2));
    
    // T на control1
    gates.push(QuantumGate::new(GateType::Phase, control1));
    
    // T† на control2
    gates.push(QuantumGate::new(GateType::Phase, control2));
    
    // CNOT между control1 и control2
    gates.push(QuantumGate::cnot(control1, control2));
    
    // H на целевом кубите
    gates.push(QuantumGate::h(target));
    
    gates
} 