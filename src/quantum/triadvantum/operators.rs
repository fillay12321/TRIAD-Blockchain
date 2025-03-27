use serde::{Serialize, Deserialize};
use num_complex::Complex64;
use crate::quantum::triadvantum::state::{QuantumState, QubitState};
use std::f64::consts::PI;
use crate::quantum::triadvantum::gates::{QuantumGate, GateType};

/// Типы квантовых операторов
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OperatorType {
    /// Унитарный оператор
    Unitary,
    /// Оператор измерения
    Measurement,
    /// Общий квантовый канал
    Channel,
    /// Суперпозиция операторов
    Superposition,
}

/// Общий трейт для всех квантовых операторов
pub trait Operator {
    /// Возвращает тип оператора
    fn get_type(&self) -> OperatorType;
    
    /// Возвращает индексы кубитов, на которые действует оператор
    fn get_affected_qubits(&self) -> Vec<usize>;
    
    /// Проверяет, изменяет ли оператор состояние системы (не является ли чистым измерением)
    fn is_state_changing(&self) -> bool;
    
    /// Проверяет, является ли оператор обратимым
    fn is_reversible(&self) -> bool;
}

/// Квантовый оператор для применения к состоянию
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumOperator {
    /// Индексы кубитов, к которым применяется оператор
    pub indices: Vec<usize>,
    /// Матрица оператора
    pub matrix: Vec<Complex64>,
}

impl QuantumOperator {
    /// Создает новый оператор Адамара
    pub fn hadamard(target: usize) -> Self {
        let invsqrt2 = 1.0 / 2.0_f64.sqrt();
        let matrix = vec![
            Complex64::new(invsqrt2, 0.0), Complex64::new(invsqrt2, 0.0),
            Complex64::new(invsqrt2, 0.0), Complex64::new(-invsqrt2, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор Паули-X (NOT)
    pub fn pauli_x(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор Паули-Y
    pub fn pauli_y(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор Паули-Z
    pub fn pauli_z(target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый фазовый оператор
    pub fn phase(target: usize, theta: f64) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(theta.cos(), theta.sin()),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор вращения вокруг оси X
    pub fn rotation_x(target: usize, theta: f64) -> Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(0.0, -sin),
            Complex64::new(0.0, -sin), Complex64::new(cos, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор вращения вокруг оси Y
    pub fn rotation_y(target: usize, theta: f64) -> Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(-sin, 0.0),
            Complex64::new(sin, 0.0), Complex64::new(cos, 0.0),
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор вращения вокруг оси Z
    pub fn rotation_z(target: usize, theta: f64) -> Self {
        let exp_plus = Complex64::new((theta / 2.0).cos(), (theta / 2.0).sin());
        let exp_minus = Complex64::new((theta / 2.0).cos(), -(theta / 2.0).sin());
        
        let matrix = vec![
            exp_minus, Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), exp_plus,
        ];
        
        Self {
            indices: vec![target],
            matrix,
        }
    }
    
    /// Создает новый оператор CNOT
    pub fn cnot(control: usize, target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
        ];
        
        Self {
            indices: vec![control, target],
            matrix,
        }
    }
    
    /// Создает новый оператор CZ
    pub fn cz(control: usize, target: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
        ];
        
        Self {
            indices: vec![control, target],
            matrix,
        }
    }
    
    /// Создает новый оператор SWAP
    pub fn swap(qubit1: usize, qubit2: usize) -> Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
        ];
        
        Self {
            indices: vec![qubit1, qubit2],
            matrix,
        }
    }
    
    /// Создает новый оператор Тоффоли (CCNOT)
    pub fn toffoli(control1: usize, control2: usize, target: usize) -> Self {
        // Создаем 8x8 матрицу для оператора Тоффоли
        let mut matrix = vec![Complex64::new(0.0, 0.0); 64];
        
        // Заполняем матрицу
        for i in 0..8 {
            if i == 7 {
                // Инвертируем бит target, если control1 и control2 установлены
                matrix[i * 8 + 6] = Complex64::new(1.0, 0.0);
                matrix[i * 8 + 7] = Complex64::new(0.0, 0.0);
            } else if i == 6 {
                // Инвертируем бит target, если control1 и control2 установлены
                matrix[i * 8 + 6] = Complex64::new(0.0, 0.0);
                matrix[i * 8 + 7] = Complex64::new(1.0, 0.0);
            } else {
                // Остальные состояния не меняются
                matrix[i * 8 + i] = Complex64::new(1.0, 0.0);
            }
        }
        
        Self {
            indices: vec![control1, control2, target],
            matrix,
        }
    }
    
    /// Создает произвольный оператор с заданной матрицей
    pub fn custom(indices: Vec<usize>, matrix: Vec<Complex64>) -> Self {
        Self { indices, matrix }
    }
    
    /// Применяет оператор к квантовому состоянию
    pub fn apply(&self, state: &mut QuantumState) -> Result<(), String> {
        match self.indices.len() {
            0 => Ok(()), // Пустой оператор
            1 => {
                // Оператор для одного кубита
                let target = self.indices[0];
                if target >= state.qubit_count {
                    return Err(format!("Индекс кубита {} вне диапазона 0-{}", 
                                      target, state.qubit_count - 1));
                }
                
                let gate = QuantumGate::Single(target, self.matrix.clone());
                gate.apply(state)
            },
            2 => {
                // Оператор для двух кубитов
                let control = self.indices[0];
                let target = self.indices[1];
                
                if control >= state.qubit_count || target >= state.qubit_count {
                    return Err(format!("Индексы кубитов {}, {} вне диапазона 0-{}", 
                                      control, target, state.qubit_count - 1));
                }
                
                let gate = QuantumGate::Two(control, target, self.matrix.clone());
                gate.apply(state)
            },
            _ => {
                // Для операторов с большим числом кубитов требуется декомпозиция
                Err("Операторы с более чем 2 кубитами пока не поддерживаются".to_string())
            }
        }
    }
}

impl Operator for QuantumOperator {
    fn get_type(&self) -> OperatorType {
        OperatorType::Unitary
    }
    
    fn get_affected_qubits(&self) -> Vec<usize> {
        self.indices.clone()
    }
    
    fn is_state_changing(&self) -> bool {
        true
    }
    
    fn is_reversible(&self) -> bool {
        true // Унитарные операторы всегда обратимы
    }
}

/// Оператор измерения
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasurementOperator {
    /// Индекс кубита для измерения
    pub qubit_idx: usize,
    /// Сохранять ли результат измерения
    pub store_result: bool,
}

impl MeasurementOperator {
    /// Создает новый оператор измерения
    pub fn new(qubit_idx: usize) -> Self {
        Self {
            qubit_idx,
            store_result: true,
        }
    }
    
    /// Применяет измерение к квантовому состоянию
    /// Возвращает результат измерения (0 или 1)
    pub fn apply(&self, state: &mut QuantumState) -> usize {
        state.measure_qubit(self.qubit_idx)
    }
}

impl Operator for MeasurementOperator {
    fn get_type(&self) -> OperatorType {
        OperatorType::Measurement
    }
    
    fn get_affected_qubits(&self) -> Vec<usize> {
        vec![self.qubit_idx]
    }
    
    fn is_state_changing(&self) -> bool {
        true // Измерение изменяет состояние через коллапс
    }
    
    fn is_reversible(&self) -> bool {
        false // Измерение необратимо
    }
}

/// Конвертирует квантовый оператор в гейт
impl From<QuantumOperator> for QuantumGate {
    fn from(operator: QuantumOperator) -> Self {
        match operator.indices.len() {
            0 => panic!("Пустой оператор не может быть преобразован в гейт"),
            1 => QuantumGate::Single(operator.indices[0], operator.matrix),
            2 => QuantumGate::Two(operator.indices[0], operator.indices[1], operator.matrix),
            _ => panic!("Операторы с более чем 2 кубитами не поддерживаются"),
        }
    }
} 