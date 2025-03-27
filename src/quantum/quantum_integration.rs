use crate::quantum::{Qubit, QuantumRegister};
use crate::quantum::triadvantum::{
    state::QuantumState, 
    circuit::QuantumCircuit, 
    simulator::QrustSimulator as TriadVantumSimulator,
    operators::QuantumOperator
};
use num_complex::Complex64;
use serde::{Serialize, Deserialize};
use std::f64::consts::PI;
use rand;

/// Типы квантовых гейтов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuantumGate {
    /// Вентиль Адамара - создает суперпозицию
    Hadamard,
    /// Вентиль Паули-X (NOT) - инвертирует состояние
    PauliX,
    /// Вентиль Паули-Y
    PauliY,
    /// Вентиль Паули-Z - фазовый сдвиг
    PauliZ,
    /// Контролируемый NOT - запутывает два кубита
    CNOT,
    /// Операция измерения
    Measure
}

/// Квантовая операция с указанием задействованных кубитов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumOperation {
    /// Операция с одним кубитом
    SingleQubit { 
        /// Тип вентиля
        gate: QuantumGate, 
        /// Индекс целевого кубита
        target: usize 
    },
    /// Операция CNOT с двумя кубитами
    CNOT { 
        /// Индекс управляющего кубита
        control: usize, 
        /// Индекс целевого кубита
        target: usize 
    },
}

/// Оценка запутанности между кубитами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementScore {
    /// Степень запутанности от 0.0 до 1.0
    pub score: f64,
    /// Между какими кубитами измерена запутанность
    pub qubit_pairs: Vec<(usize, usize)>,
}

/// Обертка над TriadVantum для квантовых вычислений
pub struct QrustSimulator {
    /// Количество кубитов в симуляторе
    qubit_count: usize,
    /// Внутреннее состояние - квантовый симулятор TriadVantum
    simulator: TriadVantumSimulator,
    /// Текущее квантовое состояние
    state: QuantumState,
}

impl QrustSimulator {
    /// Создает новый симулятор с указанным количеством кубитов
    pub fn new(qubit_count: usize) -> Self {
        // Инициализируем симулятор и состояние |0...0⟩
        // Передаем все требуемые аргументы: node_id, qubit_count, debug_mode
        let simulator = TriadVantumSimulator::new(format!("node_0"), qubit_count, false)
            .expect("Не удалось создать симулятор");
        let state = QuantumState::new(qubit_count);
        
        Self {
            qubit_count,
            simulator,
            state,
        }
    }
    
    /// Создает симулятор из существующего квантового регистра TRIAD
    pub fn from_register(register: &QuantumRegister) -> Self {
        let qubit_count = register.qubit_count();
        let mut simulator = Self::new(qubit_count);
        
        // Конвертируем состояние каждого кубита
        for i in 0..qubit_count {
            let qubit = register.get_qubit(i);
            
            // Применяем соответствующие гейты для приведения к нужному состоянию
            let prob_zero = qubit.probability_zero();
            let prob_one = qubit.probability_one();
            
            if prob_zero < 1.0 || prob_one > 0.0 {
                // Применяем вентиль Адамара, если кубит не в состоянии |0⟩
                simulator.apply_gate(QuantumGate::Hadamard, &[i]);
                
                // Здесь можно было бы точнее настроить амплитуды
                // TODO: Реализовать точную установку амплитуд через TriadVantum
            }
        }
        
        simulator
    }
    
    /// Применяет указанный квантовый вентиль к указанным кубитам
    pub fn apply_gate(&mut self, gate: QuantumGate, qubits: &[usize]) {
        // Создаем временную схему для применения гейта
        let mut circuit = QuantumCircuit::new(self.qubit_count);
        
        match gate {
            QuantumGate::Hadamard => {
                if qubits.len() == 1 {
                    circuit.h(qubits[0]);
                }
            },
            QuantumGate::PauliX => {
                if qubits.len() == 1 {
                    circuit.x(qubits[0]);
                }
            },
            QuantumGate::PauliY => {
                if qubits.len() == 1 {
                    circuit.y(qubits[0]);
                }
            },
            QuantumGate::PauliZ => {
                if qubits.len() == 1 {
                    circuit.z(qubits[0]);
                }
            },
            QuantumGate::CNOT => {
                if qubits.len() == 2 {
                    circuit.cnot(qubits[0], qubits[1]);
                }
            },
            QuantumGate::Measure => {
                if qubits.len() == 1 {
                    circuit.measure(qubits[0]);
                }
            }
        }
        
        // Выполняем схему на текущем состоянии
        if let Ok(result) = self.simulator.run_circuit(&circuit) {
            // Обрабатываем результат правильно, учитывая что final_state может отсутствовать
            if let Some(state) = result.measurements.get(&qubits[0]) {
                // Обновляем состояние на основе измерений
                // Этот код является заглушкой и требует более точной реализации
                // в зависимости от структуры SimulationResult
            }
        }
    }
    
    /// Измеряет вероятность того, что кубит находится в состоянии |1⟩
    pub fn get_qubit_probability_one(&self, qubit: usize) -> f64 {
        let qubit_state = self.state.get_qubit_state(qubit);
        qubit_state.prob_one()
    }
    
    /// Измеряет вероятность того, что кубит находится в состоянии |0⟩
    pub fn get_qubit_probability_zero(&self, qubit: usize) -> f64 {
        let qubit_state = self.state.get_qubit_state(qubit);
        qubit_state.prob_zero()
    }
    
    /// Получает состояние кубита в виде амплитуд
    pub fn get_qubit_state(&self, qubit: usize) -> Option<(Complex64, Complex64)> {
        if qubit >= self.qubit_count {
            return None;
        }
        
        let qubit_state = self.state.get_qubit_state(qubit);
        Some((qubit_state.alpha, qubit_state.beta))
    }
    
    /// Устанавливает состояние кубита
    pub fn set_qubit_state(&mut self, qubit: usize, alpha: Complex64, beta: Complex64) {
        // В TriadVantum пока нет прямого метода установки состояния кубита
        // Поэтому реализуем через систему гейтов
        
        // Сначала сбрасываем кубит в |0⟩
        let prob_one = self.get_qubit_probability_one(qubit);
        if prob_one > 0.0 {
            // Если вероятность |1⟩ не нулевая, применяем X для сброса в |0⟩
            self.apply_gate(QuantumGate::PauliX, &[qubit]);
        }
        
        // Теперь устанавливаем нужное состояние
        // Для этого нам нужно применить соответствующее вращение
        // (для упрощения пока реализуем только применение H, если нужна суперпозиция)
        if alpha.norm_sqr() < 1.0 && beta.norm_sqr() > 0.0 {
            self.apply_gate(QuantumGate::Hadamard, &[qubit]);
            
            // Если нужно точное состояние, здесь должны быть дополнительные вращения
            // TODO: Реализовать точную установку амплитуд через последовательность вращений
        }
    }
    
    /// Измеряет запутанность между всеми парами кубитов
    pub fn measure_entanglement(&self) -> EntanglementScore {
        let mut total_score = 0.0;
        let mut pairs = Vec::new();
        
        // Перебираем все пары кубитов
        for i in 0..self.qubit_count {
            for j in i+1..self.qubit_count {
                let pair_score = self.calculate_pair_entanglement(i, j);
                if pair_score > 0.01 { // Порог существенной запутанности
                    total_score += pair_score;
                    pairs.push((i, j));
                }
            }
        }
        
        // Нормализуем суммарную оценку
        let pair_count = self.qubit_count * (self.qubit_count - 1) / 2;
        if pair_count > 0 {
            total_score /= pair_count as f64;
        }
        
        EntanglementScore {
            score: total_score,
            qubit_pairs: pairs,
        }
    }
    
    /// Вычисляет запутанность между парой кубитов
    fn calculate_pair_entanglement(&self, qubit1: usize, qubit2: usize) -> f64 {
        // Вычисляем запутанность как меру корреляции
        let prob00 = self.get_basis_state_probability(vec![(qubit1, 0), (qubit2, 0)]);
        let prob01 = self.get_basis_state_probability(vec![(qubit1, 0), (qubit2, 1)]);
        let prob10 = self.get_basis_state_probability(vec![(qubit1, 1), (qubit2, 0)]);
        let prob11 = self.get_basis_state_probability(vec![(qubit1, 1), (qubit2, 1)]);
        
        // Используем взаимную информацию как меру запутанности
        let mut entanglement = 0.0;
        
        if prob00 > 0.0 && prob11 > 0.0 {
            // Есть корреляция между |00⟩ и |11⟩
            entanglement += (prob00 * prob11).sqrt();
        }
        
        if prob01 > 0.0 && prob10 > 0.0 {
            // Есть корреляция между |01⟩ и |10⟩
            entanglement += (prob01 * prob10).sqrt();
        }
        
        entanglement
    }
    
    /// Получает вероятность конкретного базисного состояния
    fn get_basis_state_probability(&self, qubit_values: Vec<(usize, u8)>) -> f64 {
        // Создаем маску для указанных кубитов
        let mut full_state_idx = 0;
        
        for (qubit, value) in qubit_values {
            if qubit >= self.qubit_count {
                return 0.0;
            }
            
            if value == 1 {
                full_state_idx |= 1 << qubit;
            }
        }
        
        // Получаем амплитуду для этого состояния
        let amplitude = self.state.get_amplitude(full_state_idx);
        amplitude.norm_sqr()
    }
    
    /// Измеряет интерференцию состояния
    pub fn measure_interference(&self) -> f64 {
        // Оценка интерференции на основе амплитуд
        // Для простой оценки используем стандартное отклонение амплитуд
        
        // Получаем распределение вероятностей для всех базисных состояний
        let mut sum_prob = 0.0;
        let mut sum_prob_sq = 0.0;
        
        for i in 0..self.qubit_count {
            let prob = self.get_qubit_probability_one(i);
            sum_prob += prob;
            sum_prob_sq += prob * prob;
        }
        
        let mean = sum_prob / self.qubit_count as f64;
        let variance = sum_prob_sq / self.qubit_count as f64 - mean * mean;
        
        // Стандартное отклонение как мера интерференции
        variance.sqrt()
    }
    
    /// Конвертирует текущее состояние в квантовый регистр TRIAD
    pub fn to_register(&self) -> QuantumRegister {
        let mut register = QuantumRegister::new(self.qubit_count);
        
        // Конвертируем каждый кубит
        for i in 0..self.qubit_count {
            if let Some((alpha, beta)) = self.get_qubit_state(i) {
                let mut qubit = Qubit::new();
                // TODO: Обновить API Qubit для установки амплитуд
                // qubit.set_amplitudes(alpha, beta);
                // register.set_qubit(i, qubit);
            }
        }
        
        register
    }
    
    /// Применяет квантовую операцию
    pub fn apply_operation(&mut self, operation: &QuantumOperation) {
        match operation {
            QuantumOperation::SingleQubit { gate, target } => {
                self.apply_gate(*gate, &[*target]);
            },
            QuantumOperation::CNOT { control, target } => {
                self.apply_gate(QuantumGate::CNOT, &[*control, *target]);
            }
        }
    }
    
    /// Измеряет кубит и возвращает результат
    pub fn measure_qubit(&mut self, qubit: usize) -> u8 {
        // Создаем схему для измерения
        let mut circuit = QuantumCircuit::new(self.qubit_count);
        circuit.measure(qubit);
        
        // Выполняем измерение
        let result = self.simulator.run_circuit(&circuit);
        
        if let Ok(result) = result {
            // Получаем результат измерения из maps measurements, а не из final_state
            if let Some(&outcome) = result.measurements.get(&qubit) {
                return outcome as u8;
            }
        }
        
        // Возвращаем 0 по умолчанию, если не удалось измерить
        0
    }
    
    /// Возвращает вероятность измерения |0⟩ для указанного кубита
    pub fn probability_of_zero(&self, qubit: usize) -> f64 {
        self.get_qubit_probability_zero(qubit)
    }
    
    /// Возвращает вероятность измерения |1⟩ для указанного кубита
    pub fn probability_of_one(&self, qubit: usize) -> f64 {
        self.get_qubit_probability_one(qubit)
    }
    
    /// Получает текущее состояние квантовой системы
    pub fn get_state(&self) -> &QuantumState {
        &self.state
    }
    
    /// Установка нового состояния
    pub fn set_state(&mut self, state: QuantumState) {
        self.state = state;
    }
}

/// Билдер для создания квантовых схем
pub struct QuantumCircuitBuilder {
    /// Количество кубитов в схеме
    qubit_count: usize,
    /// Последовательность операций
    operations: Vec<QuantumOperation>,
}

impl QuantumCircuitBuilder {
    /// Создает новый билдер с указанным количеством кубитов
    pub fn new(qubit_count: usize) -> Self {
        Self {
            qubit_count,
            operations: Vec::new(),
        }
    }
    
    /// Добавляет вентиль Адамара на указанный кубит
    pub fn hadamard(&mut self, qubit: usize) -> &mut Self {
        if qubit < self.qubit_count {
            self.operations.push(QuantumOperation::SingleQubit { 
                gate: QuantumGate::Hadamard, 
                target: qubit 
            });
        }
        self
    }
    
    /// Добавляет вентиль Паули-X (NOT) на указанный кубит
    pub fn x(&mut self, qubit: usize) -> &mut Self {
        if qubit < self.qubit_count {
            self.operations.push(QuantumOperation::SingleQubit { 
                gate: QuantumGate::PauliX, 
                target: qubit 
            });
        }
        self
    }
    
    /// Добавляет вентиль CNOT между двумя кубитами
    pub fn cnot(&mut self, control: usize, target: usize) -> &mut Self {
        if control < self.qubit_count && target < self.qubit_count && control != target {
            self.operations.push(QuantumOperation::CNOT { 
                control, 
                target 
            });
        }
        self
    }
    
    /// Добавляет операцию измерения кубита
    pub fn measure(&mut self, qubit: usize) -> &mut Self {
        if qubit < self.qubit_count {
            self.operations.push(QuantumOperation::SingleQubit { 
                gate: QuantumGate::Measure, 
                target: qubit 
            });
        }
        self
    }
    
    /// Создает пару Белла (запутанную пару) между двумя кубитами
    pub fn bell_pair(&mut self, qubit1: usize, qubit2: usize) -> &mut Self {
        self.hadamard(qubit1).cnot(qubit1, qubit2)
    }
    
    /// Возвращает список операций
    pub fn operations(&self) -> &[QuantumOperation] {
        &self.operations
    }
    
    /// Выполняет схему на указанном симуляторе
    pub fn execute(&self, simulator: &mut QrustSimulator) {
        for op in &self.operations {
            simulator.apply_operation(op);
        }
    }
}

impl QuantumOperation {
    /// Создает операцию с одним кубитом
    pub fn SingleQubit(gate: QuantumGate, target: usize) -> Self {
        QuantumOperation::SingleQubit {
            gate,
            target
        }
    }
    
    /// Создает операцию CNOT с двумя кубитами
    pub fn CNOT(control: usize, target: usize) -> Self {
        QuantumOperation::CNOT {
            control,
            target
        }
    }
} 