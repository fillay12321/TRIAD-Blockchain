use crate::quantum::{
    Qubit as LegacyQubit, 
    QuantumRegister as LegacyRegister, 
    quantum_integration::{QuantumGate as LegacyGate, QuantumOperation as LegacyOperation}
};
use crate::quantum::triadvantum::{
    self, 
    state::QuantumState, 
    state::QubitState,
    circuit::QuantumCircuit, 
    simulator::{QrustSimulator, SimulationResult},
    delta::{QuantumDelta, DeltaCompressor},
    recovery::{RecoveryProtocol, RecoveryEventType},
};
use num_complex::Complex64;
use crate::quantum::quantum_integration::{
    QrustSimulator as LegacyQrustSimulator, EntanglementScore
};

/// Адаптер для интеграции библиотеки TriadVantum с существующим кодом TRIAD
pub struct TriadVantumAdapter {
    /// Внутренний симулятор TriadVantum
    pub simulator: QrustSimulator,
    /// Текущая квантовая схема
    pub circuit: QuantumCircuit,
    /// Компрессор дельт для эффективной передачи изменений
    delta_compressor: DeltaCompressor,
    /// Протокол восстановления
    recovery: RecoveryProtocol,
    /// Идентификатор узла
    node_id: String,
    /// Количество кубитов
    qubit_count: usize,
    /// Внутреннее состояние
    state: Option<QuantumState>,
}

impl TriadVantumAdapter {
    /// Создает новый адаптер с указанным количеством кубитов
    pub fn new(qubit_count: usize, node_id: String) -> Self {
        // Проверяем, что количество кубитов не превышает разумный предел
        if qubit_count > 30 {
            panic!("Слишком большое количество кубитов: {}. Максимально допустимое значение: 30", qubit_count);
        }
        
        Self {
            state: Some(QuantumState::new(qubit_count)),
            circuit: QuantumCircuit::new(qubit_count),
            simulator: QrustSimulator::new(node_id.clone(), qubit_count, false).expect("Ошибка создания симулятора"),
            recovery: RecoveryProtocol::new(),
            delta_compressor: DeltaCompressor::new(1000), // Максимальный размер дельты 1000
            node_id,
            qubit_count
        }
    }
    
    /// Конвертирует устаревший квантовый регистр в формат TriadVantum
    pub fn from_legacy_register(register: &LegacyRegister, node_id: &str) -> Self {
        let qubit_count = register.qubit_count();
        let mut adapter = Self::new(qubit_count, node_id.to_string());
        
        // Создаем состояние на основе кубитов из старого регистра
        let mut qubit_states = Vec::with_capacity(qubit_count);
        
        for i in 0..qubit_count {
            let legacy_qubit = register.get_qubit(i);
            // Заглушка: создаем кубит в состоянии |0⟩, так как доступа к амплитудам нет
            let alpha = Complex64::new(1.0, 0.0); // |0⟩
            let beta = Complex64::new(0.0, 0.0);  // |1⟩
            
            qubit_states.push(QubitState::with_amplitudes(alpha, beta));
        }
        
        // Создаем квантовое состояние из отдельных кубитов
        let quantum_state = QuantumState::from_qubit_states(&qubit_states);
        
        // Создаем контрольную точку для восстановления
        adapter.recovery.add_checkpoint("initial".to_string(), quantum_state.clone());
        adapter.state = Some(quantum_state);
        
        // Создаем схему для текущего состояния
        adapter.circuit = QuantumCircuit::new(qubit_count);
        adapter.circuit.name = "Legacy Circuit".to_string();
        
        adapter
    }
    
    /// Конвертирует устаревший гейт в формат TriadVantum
    pub fn convert_legacy_gate(&self, gate: LegacyGate, qubits: &[usize]) -> Option<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.circuit.qubit_count);
        circuit.name = "Converted Gate".to_string();
        
        match gate {
            LegacyGate::Hadamard => {
                if qubits.len() == 1 {
                    circuit.h(qubits[0]);
                    Some(circuit)
                } else {
                    None
                }
            },
            LegacyGate::PauliX => {
                if qubits.len() == 1 {
                    circuit.x(qubits[0]);
                    Some(circuit)
                } else {
                    None
                }
            },
            LegacyGate::PauliY => {
                if qubits.len() == 1 {
                    circuit.y(qubits[0]);
                    Some(circuit)
                } else {
                    None
                }
            },
            LegacyGate::PauliZ => {
                if qubits.len() == 1 {
                    circuit.z(qubits[0]);
                    Some(circuit)
                } else {
                    None
                }
            },
            LegacyGate::CNOT => {
                if qubits.len() == 2 {
                    circuit.cnot(qubits[0], qubits[1]);
                    Some(circuit)
                } else {
                    None
                }
            },
            LegacyGate::Measure => {
                if qubits.len() == 1 {
                    circuit.measure(qubits[0]);
                    Some(circuit)
                } else {
                    None
                }
            }
        }
    }
    
    /// Конвертирует устаревшую операцию в формат TriadVantum
    pub fn convert_legacy_operation(&self, operation: &LegacyOperation) -> Option<QuantumCircuit> {
        match operation {
            LegacyOperation::SingleQubit { gate, target } => {
                self.convert_legacy_gate(*gate, &[*target])
            },
            LegacyOperation::CNOT { control, target } => {
                self.convert_legacy_gate(LegacyGate::CNOT, &[*control, *target])
            }
        }
    }
    
    /// Выполняет квантовую схему и возвращает результат
    pub fn execute_circuit(&mut self) -> Result<SimulationResult, String> {
        // Создаем начальный чекпоинт
        if let Some(ref state) = self.state {
            self.recovery.add_checkpoint("pre_execute".to_string(), state.clone());
        }

        // Выполняем схему
        let result = self.simulator.run_circuit(&self.circuit)?;

        // Устанавливаем новое состояние
        self.state = Some(result.final_state.clone());

        // Добавляем финальный чекпоинт
        self.recovery.add_checkpoint("post_execute".to_string(), result.final_state.clone());

        Ok(result)
    }
    
    /// Применяет устаревшую операцию к текущей схеме
    pub fn apply_legacy_operation(&mut self, operation: &LegacyOperation) -> bool {
        if let Some(circuit) = self.convert_legacy_operation(operation) {
            // Добавляем гейты из конвертированной схемы в текущую
            for gate in circuit.gates {
                self.circuit.add_gate(gate);
            }
            true
        } else {
            false
        }
    }
    
    /// Восстанавливает состояние в случае ошибки
    pub fn recover_state(&mut self) -> Option<QuantumState> {
        // Пытаемся восстановить последний чекпоинт
        if let Ok(state) = self.recovery.recover_state("post_execute") {
            self.state = Some(state.clone());
            return Some(state);
        } else if let Ok(state) = self.recovery.recover_state("pre_execute") {
            self.state = Some(state.clone());
            return Some(state);
        } else if let Ok(state) = self.recovery.recover_state("initial") {
            self.state = Some(state.clone());
            return Some(state);
        } else {
            None
        }
    }
    
    /// Конвертирует квантовое состояние TriadVantum в устаревший регистр
    pub fn to_legacy_register(&self, state: &QuantumState) -> LegacyRegister {
        let mut register = LegacyRegister::new(state.qubit_count);
        
        // В этой реализации мы создаем новые кубиты с состоянием |0⟩,
        // так как нет прямой возможности установить амплитуды
        
        // Конвертируем каждый кубит
        for i in 0..state.qubit_count {
            // Здесь должна быть логика конвертации,
            // но в текущей имплементации LegacyQubit нет нужных методов
            // Оставляем заглушку
        }
        
        register
    }
    
    /// Создает дельту изменений для передачи по сети
    pub fn create_delta(&mut self, state: &QuantumState) -> Result<QuantumDelta, String> {
        if let Some(ref current_state) = self.state {
            Ok(self.delta_compressor.create_delta_full(
                current_state,
                state,
                self.node_id.clone() // Используем ID узла вместо строки "1000"
            ))
        } else {
            Err("Текущее состояние не инициализировано".to_string())
        }
    }
    
    /// Возвращает последний результат симуляции, если он существует
    pub fn last_result(&self) -> Option<&SimulationResult> {
        None // В новой реализации этот метод не поддерживается напрямую
    }
    
    /// Применяет дельту к состоянию
    pub fn apply_delta(&mut self, delta: &QuantumDelta, state: &mut QuantumState) -> Result<(), String> {
        delta.apply_to(state)
    }
    
    /// Очищает текущую квантовую схему
    pub fn clear_circuit(&mut self) {
        self.circuit = QuantumCircuit::new(self.qubit_count);
    }
    
    /// Возвращает текущее состояние
    pub fn get_state(&self) -> Option<QuantumState> {
        self.state.clone()
    }
    
    /// Устанавливает новое состояние
    pub fn set_state(&mut self, state: QuantumState) -> Result<(), String> {
        if state.qubit_count != self.qubit_count {
            return Err(format!("Неправильное количество кубитов: ожидается {}, получено {}", 
                self.qubit_count, state.qubit_count));
        }
        
        self.simulator.set_state(state.clone())?;
        self.state = Some(state);
        Ok(())
    }
    
    /// Создает квантовую схему QFT
    pub fn create_qft_circuit(&mut self) -> Result<(), String> {
        if self.qubit_count < 2 {
            return Err("Для QFT требуется как минимум 2 кубита".to_string());
        }
        
        self.clear_circuit();
        
        // Создаем схему для квантового преобразования Фурье
        for i in 0..self.qubit_count {
            // Применяем гейт Адамара к текущему кубиту
            self.circuit.h(i);
            
            // Применяем контролируемые фазовые вращения
            for j in 1..(self.qubit_count - i) {
                let theta = std::f64::consts::PI / (1_f64 * (1 << j) as f64);
                self.circuit.controlled_phase(i, i + j, theta);
            }
        }
        
        // Меняем порядок кубитов (swap)
        for i in 0..(self.qubit_count / 2) {
            self.circuit.swap(i, self.qubit_count - i - 1);
        }
        
        self.circuit.name = "Quantum Fourier Transform".to_string();
        
        Ok(())
    }
} 