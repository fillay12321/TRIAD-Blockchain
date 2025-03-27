use std::vec::Vec;
use serde::{Serialize, Deserialize};
use crate::quantum::triadvantum::gates::{QuantumGate, GateType, GateParams};
use crate::quantum::triadvantum::operators::{QuantumOperator, MeasurementOperator};
use std::f64::consts::PI;
use std::fmt;
use std::collections::HashMap;
use num_complex::Complex64;
use crate::quantum::triadvantum::state::QuantumState;
use std::collections::HashSet;
use num_complex::Complex64 as OldComplex64;

/// Квантовая цепь, представляющая последовательность квантовых гейтов
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumCircuit {
    /// Количество кубитов в цепи
    pub qubit_count: usize,
    /// Последовательность квантовых гейтов
    pub gates: Vec<QuantumGate>,
    /// Название цепи (для отладки)
    pub name: String,
    /// Результаты измерений, индексированные по индексу кубита и времени измерения
    pub measurement_results: HashMap<(usize, usize), usize>,
    /// Метаданные схемы
    pub metadata: String,
}

impl QuantumCircuit {
    /// Создает новую пустую квантовую цепь с указанным количеством кубитов
    pub fn new(qubit_count: usize) -> Self {
        Self {
            qubit_count,
            gates: Vec::new(),
            name: format!("Circuit[{}]", qubit_count),
            measurement_results: HashMap::new(),
            metadata: String::new(),
        }
    }
    
    /// Создает новую квантовую цепь с заданным именем
    pub fn new_named(qubit_count: usize, name: &str) -> Self {
        Self {
            qubit_count,
            gates: Vec::new(),
            name: name.to_string(),
            measurement_results: HashMap::new(),
            metadata: String::new(),
        }
    }
    
    /// Добавляет гейт в конец цепи
    pub fn add_gate(&mut self, gate: QuantumGate) -> &mut Self {
        // Проверяем, что гейт использует только кубиты, которые есть в цепи
        match gate {
            QuantumGate::Single(target, _) => {
                if target >= self.qubit_count {
                    panic!("Индекс кубита {} вне диапазона цепи с {} кубитами", target, self.qubit_count);
                }
            },
            QuantumGate::Two(control, target, _) => {
                if control >= self.qubit_count || target >= self.qubit_count {
                    panic!("Индекс кубита вне диапазона цепи с {} кубитами", self.qubit_count);
                }
            }
        }
        
        self.gates.push(gate);
        self
    }
    
    /// Алиас для add_gate, добавлен для совместимости с UnitaryOperator
    pub fn add_operator(&mut self, operator: impl Into<QuantumGate>) -> &mut Self {
        self.add_gate(operator.into())
    }
    
    /// Добавляет гейт Адамара на указанный кубит
    pub fn h(&mut self, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::hadamard(target))
    }
    
    /// Добавляет гейт Паули-X (NOT) на указанный кубит
    pub fn x(&mut self, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::pauli_x(target))
    }
    
    /// Добавляет гейт Паули-Y на указанный кубит
    pub fn y(&mut self, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::pauli_y(target))
    }
    
    /// Добавляет гейт Паули-Z на указанный кубит
    pub fn z(&mut self, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::pauli_z(target))
    }
    
    /// Добавляет фазовый гейт на указанный кубит
    pub fn phase(&mut self, target: usize, theta: f64) -> &mut Self {
        let invsqrt2 = 1.0 / 2.0_f64.sqrt();
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(theta.cos(), theta.sin())
        ];
        self.add_gate(QuantumGate::Single(target, matrix))
    }
    
    /// Добавляет гейт вращения вокруг оси X
    pub fn rx(&mut self, target: usize, theta: f64) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(0.0, -sin),
            Complex64::new(0.0, -sin), Complex64::new(cos, 0.0)
        ];
        self.add_gate(QuantumGate::Single(target, matrix))
    }
    
    /// Добавляет гейт вращения вокруг оси Y
    pub fn ry(&mut self, target: usize, theta: f64) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        
        let matrix = vec![
            Complex64::new(cos, 0.0), Complex64::new(-sin, 0.0),
            Complex64::new(sin, 0.0), Complex64::new(cos, 0.0)
        ];
        self.add_gate(QuantumGate::Single(target, matrix))
    }
    
    /// Добавляет гейт вращения вокруг оси Z
    pub fn rz(&mut self, target: usize, theta: f64) -> &mut Self {
        let exp_plus = Complex64::new((theta / 2.0).cos(), (theta / 2.0).sin());
        let exp_minus = Complex64::new((theta / 2.0).cos(), -(theta / 2.0).sin());
        
        let matrix = vec![
            exp_minus, Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), exp_plus
        ];
        self.add_gate(QuantumGate::Single(target, matrix))
    }
    
    /// Добавляет гейт CNOT
    pub fn cnot(&mut self, control: usize, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::cnot(control, target))
    }
    
    /// Добавляет гейт CZ
    pub fn cz(&mut self, control: usize, target: usize) -> &mut Self {
        self.add_gate(QuantumGate::cz(control, target))
    }
    
    /// Добавляет контролируемый фазовый гейт
    pub fn controlled_phase(&mut self, control: usize, target: usize, angle: f64) -> &mut Self {
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(angle.cos(), angle.sin())
        ];
        self.add_gate(QuantumGate::Two(control, target, matrix))
    }
    
    /// Добавляет гейт SWAP
    pub fn swap(&mut self, qubit1: usize, qubit2: usize) -> &mut Self {
        self.add_gate(QuantumGate::swap(qubit1, qubit2))
    }
    
    /// Добавляет гейт Тоффоли
    pub fn toffoli(&mut self, control1: usize, control2: usize, target: usize) -> &mut Self {
        let operator = QuantumOperator::toffoli(control1, control2, target);
        self.add_operator(operator)
    }
    
    /// Добавляет операцию измерения
    pub fn measure(&mut self, target: usize) -> &mut Self {
        // Идентичностная матрица для измерения
        let matrix = vec![
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)
        ];
        self.add_gate(QuantumGate::Single(target, matrix))
    }
    
    /// Добавляет все гейты из другой цепи
    pub fn extend(&mut self, other: &QuantumCircuit) -> &mut Self {
        // Проверяем совместимость
        if other.qubit_count > self.qubit_count {
            panic!("Невозможно объединить цепи: исходная цепь имеет {} кубитов, а добавляемая - {}", 
                  self.qubit_count, other.qubit_count);
        }
        
        // Копируем гейты
        for gate in &other.gates {
            self.add_gate(gate.clone());
        }
        
        self
    }
    
    /// Применяет отдельный гейт к квантовому состоянию
    fn apply_gate(&self, gate: &QuantumGate, state: &mut QuantumState) {
        match gate {
            QuantumGate::Single(target, matrix) => {
                state.apply_single_qubit_gate(*target, matrix.clone());
            },
            QuantumGate::Two(control, target, matrix) => {
                state.apply_two_qubit_gate(*control, *target, matrix.clone());
            }
        }
    }
    
    /// Выполняет квантовую цепь на указанном состоянии
    pub fn execute(&mut self, state: &mut QuantumState) -> HashMap<usize, usize> {
        // Результаты измерений, индексированные по индексу кубита
        let mut results = HashMap::new();
        
        // Применяем гейты последовательно
        for (gate_idx, gate) in self.gates.iter().enumerate() {
            match gate {
                QuantumGate::Single(target, matrix) => {
                    // Если это измерение (идентичностная матрица)
                    let is_measure = matrix[0].re == 1.0 && matrix[0].im == 0.0 &&
                                    matrix[1].re == 0.0 && matrix[1].im == 0.0 &&
                                    matrix[2].re == 0.0 && matrix[2].im == 0.0 &&
                                    matrix[3].re == 1.0 && matrix[3].im == 0.0;
                    
                    if is_measure {
                        // Измеряем кубит
                        let result = state.measure_qubit(*target);
                        results.insert(*target, result);
                        self.measurement_results.insert((*target, gate_idx), result);
                    } else {
                        // Применяем одиночный гейт
                        state.apply_single_qubit_gate(*target, matrix.clone());
                    }
                },
                QuantumGate::Two(control, target, matrix) => {
                    // Применяем двухкубитный гейт
                    state.apply_two_qubit_gate(*control, *target, matrix.clone());
                }
            }
        }
        
        results
    }
    
    /// Получает вероятность измерения указанной комбинации битов
    pub fn get_probability(&self, state: &QuantumState, bit_string: &str) -> f64 {
        // Проверяем совместимость
        if bit_string.len() > self.qubit_count {
            panic!("Строка битов длиннее, чем количество кубитов в цепи");
        }
        
        // Для получения вероятности используем более простой подход:
        // получаем вероятность каждого кубита быть в состоянии, указанном в шаблоне,
        // и умножаем их, что верно для несвязанных кубитов
        let mut probability = 1.0;
        
        for (qubit_idx, bit_char) in bit_string.chars().enumerate() {
            let expected_bit = match bit_char {
                '0' => 0,
                '1' => 1,
                _ => panic!("Недопустимый символ в строке битов: {}", bit_char),
            };
            
            let qubit_state = state.get_qubit_state(qubit_idx);
            let bit_prob = if expected_bit == 0 { 
                qubit_state.prob_zero() 
            } else { 
                qubit_state.prob_one() 
            };
            
            probability *= bit_prob;
        }
        
        probability
    }
    
    /// Клонирует цепь и добавляет указанный гейт
    pub fn with_gate(&self, gate: QuantumGate) -> Self {
        let mut new_circuit = self.clone();
        new_circuit.add_gate(gate);
        new_circuit
    }
    
    /// Создает строковое представление цепи
    pub fn to_string(&self) -> String {
        let mut result = format!("QuantumCircuit \"{}\" с {} кубитами:\n", self.name, self.qubit_count);
        
        for (i, gate) in self.gates.iter().enumerate() {
            result.push_str(&format!("  {}: {}\n", i, gate.get_description()));
        }
        
        result
    }

    /// Проверяет валидность схемы
    pub fn validate(&self) -> Result<(), String> {
        // Проверяем количество кубитов
        if self.qubit_count == 0 {
            return Err("Circuit must have at least one qubit".to_string());
        }

        // Проверяем каждый гейт
        for gate in &self.gates {
            match gate {
                QuantumGate::Single(target, _) => {
                    if *target >= self.qubit_count {
                        return Err(format!("Invalid target qubit index: {}", target));
                    }
                },
                QuantumGate::Two(control, target, _) => {
                    if *control >= self.qubit_count || *target >= self.qubit_count {
                        return Err(format!("Invalid qubit indices: control={}, target={}", control, target));
                    }
                }
            }
        }

        Ok(())
    }

    /// Применяет схему к квантовому состоянию
    pub fn apply_to_state(&self, state: &mut QuantumState) -> Result<(), String> {
        // Проверяем валидность схемы
        self.validate()?;

        // Применяем каждый гейт
        for gate in &self.gates {
            self.apply_gate(gate, state);
        }

        Ok(())
    }

    /// Получает глубину схемы
    pub fn depth(&self) -> usize {
        self.gates.len()
    }

    /// Получает количество гейтов
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Получает количество кубитов
    pub fn qubit_count(&self) -> usize {
        self.qubit_count
    }

    /// Получает метаданные схемы
    pub fn metadata(&self) -> &str {
        &self.metadata
    }

    /// Устанавливает метаданные схемы
    pub fn set_metadata(&mut self, metadata: String) {
        self.metadata = metadata;
    }

    /// Получает список гейтов в схеме
    pub fn get_gates(&self) -> &Vec<QuantumGate> {
        &self.gates
    }
}

/// Создает квантовую цепь для телепортации кубита
pub fn create_teleportation_circuit() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new_named(3, "Телепортация");
    
    // Инициализация запутанной пары (кубиты 1 и 2)
    circuit.h(1)
           .cnot(1, 2);
    
    // Операции Алисы (кубиты 0 и 1)
    circuit.cnot(0, 1)
           .h(0)
           .measure(0)
           .measure(1);
    
    // Операции Боба на основе измерений Алисы (кубит 2)
    // В реальности эти операции будут применяться условно,
    // в зависимости от результатов измерений
    circuit.x(2) // Применяется, если результат измерения кубита 1 равен 1
           .z(2); // Применяется, если результат измерения кубита 0 равен 1
    
    circuit
}

/// Создает квантовую цепь для алгоритма Гровера поиска в несортированной базе данных
pub fn create_grover_circuit(qubit_count: usize, marked_state: usize) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new_named(qubit_count, "Алгоритм Гровера");
    
    // Инициализация суперпозиции на всех кубитах
    for i in 0..qubit_count {
        circuit.h(i);
    }
    
    // Количество итераций Гровера (оптимальное)
    let iterations = (std::f64::consts::PI / 4.0 * (1 << qubit_count) as f64).sqrt() as usize;
    
    for _ in 0..iterations {
        // Оракул - инвертирует амплитуду для отмеченного состояния
        // Здесь реализуем через многоконтролируемый Z с дополнительными X
        
        // Применяем X к кубитам, где в отмеченном состоянии стоит 0
        for i in 0..qubit_count {
            if (marked_state >> i) & 1 == 0 {
                circuit.x(i);
            }
        }
        
        // Многоконтролируемый Z (реализован через декомпозицию)
        // В реальности для этого потребуется дополнительная логика
        
        // Применяем X обратно
        for i in 0..qubit_count {
            if (marked_state >> i) & 1 == 0 {
                circuit.x(i);
            }
        }
        
        // Диффузия - инвертирует амплитуды относительно среднего
        
        // H на всех кубитах
        for i in 0..qubit_count {
            circuit.h(i);
        }
        
        // X на всех кубитах
        for i in 0..qubit_count {
            circuit.x(i);
        }
        
        // Многоконтролируемый Z (через декомпозицию)
        // В реальности для этого потребуется дополнительная логика
        
        // X на всех кубитах
        for i in 0..qubit_count {
            circuit.x(i);
        }
        
        // H на всех кубитах
        for i in 0..qubit_count {
            circuit.h(i);
        }
    }
    
    // Измеряем все кубиты
    for i in 0..qubit_count {
        circuit.measure(i);
    }
    
    circuit
}

/// Создает квантовую цепь для алгоритма Шора
/// Упрощенная версия, только для демонстрационных целей
pub fn create_shor_circuit(a: usize, n: usize) -> QuantumCircuit {
    // Определяем количество кубитов в регистрах
    // Для полной реализации потребуется больше логики и кубитов
    let counting_qubits = (2.0 * (n as f64).log2().ceil()) as usize;
    let work_qubits = (n as f64).log2().ceil() as usize;
    let total_qubits = counting_qubits + work_qubits;
    
    let circuit_name = format!("Алгоритм Шора для факторизации {}", n);
    let mut circuit = QuantumCircuit::new_named(total_qubits, &circuit_name);
    
    // Инициализация регистра счетчика в суперпозицию
    for i in 0..counting_qubits {
        circuit.h(i);
    }
    
    // Здесь должна быть реализация модульного возведения в степень
    // a^x mod N, где x - значение в регистре счетчика
    // Для упрощения этот шаг опущен
    
    // Квантовое преобразование Фурье на регистре счетчика
    for i in 0..counting_qubits/2 {
        circuit.swap(i, counting_qubits - i - 1);
    }
    
    for i in 0..counting_qubits {
        circuit.h(i);
        
        for j in i+1..counting_qubits {
            let theta = std::f64::consts::PI / (1 << (j - i)) as f64;
            circuit.phase(j, theta);
        }
    }
    
    // Измеряем регистр счетчика
    for i in 0..counting_qubits {
        circuit.measure(i);
    }
    
    circuit
}

/// Создает квантовую цепь для квантового преобразования Фурье
pub fn create_qft_circuit(qubit_count: usize) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new_named(qubit_count, "Квантовое преобразование Фурье");
    
    // Применяем QFT
    for i in 0..qubit_count {
        // H на i-й кубит
        circuit.h(i);
        
        // Контролируемые вращения фазы
        for j in i+1..qubit_count {
            // Угол вращения для j-го кубита
            let angle = std::f64::consts::PI / (1u64 << (j - i)) as f64;
            circuit.controlled_phase(i, j, angle);
        }
    }
    
    // Инвертируем порядок кубитов
    for i in 0..qubit_count/2 {
        circuit.swap(i, qubit_count - i - 1);
    }
    
    circuit
} 