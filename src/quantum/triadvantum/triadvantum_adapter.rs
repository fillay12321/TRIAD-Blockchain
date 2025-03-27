use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::f64::consts::PI;
use num_complex::Complex64;

use crate::quantum::triadvantum::{
    QuantumState, QubitState,
    gates::QuantumGate,
    circuit::QuantumCircuit,
    simulator::{QrustSimulator, SimulationResult},
    delta::{QuantumDelta, DeltaCompressor},
    recovery::{RecoveryProtocol, RecoveryEvent, RecoveryEventType},
    interference::InterferencePattern
};

/// Адаптер для работы с квантовым симулятором TriadVantum
pub struct TriadVantumAdapter {
    /// Идентификатор ноды
    pub node_id: String,
    /// Количество кубитов
    pub qubit_count: usize,
    /// Квантовый симулятор
    simulator: QrustSimulator,
    /// Квантовая схема
    circuit: QuantumCircuit,
    /// Компрессор дельт
    delta_compressor: DeltaCompressor,
    /// Протокол восстановления
    recovery: RecoveryProtocol,
    /// Последний результат симуляции
    last_result: Option<SimulationResult>,
    /// История состояний
    state_history: VecDeque<QuantumState>,
    /// Максимальный размер истории
    max_history_size: usize,
}

impl TriadVantumAdapter {
    /// Создает новый адаптер для работы с квантовым симулятором
    pub fn new(node_id: String, qubit_count: usize) -> Self {
        let simulator = QrustSimulator::new(node_id.clone(), qubit_count, false)
            .expect("Не удалось создать квантовый симулятор");
        
        let circuit = QuantumCircuit::new(qubit_count);
        let delta_compressor = DeltaCompressor::new(100);
        let recovery = RecoveryProtocol::new();
        
        TriadVantumAdapter {
            node_id,
            qubit_count,
            simulator,
            circuit,
            delta_compressor,
            recovery,
            last_result: None,
            state_history: VecDeque::new(),
            max_history_size: 100,
        }
    }

    /// Устанавливает начальное состояние
    pub fn set_initial_state(&mut self, state: QuantumState) -> Result<(), String> {
        if state.qubit_count() != self.qubit_count {
            return Err(format!("Количество кубитов в состоянии ({}) не соответствует количеству кубитов в симуляторе ({})", 
                state.qubit_count(), self.qubit_count));
        }

        // Очищаем историю и добавляем новое состояние
        self.state_history.clear();
        self.state_history.push_back(state.clone());
        
        // Устанавливаем состояние в симулятор
        self.simulator.set_state(state)?;
        
        // Добавляем чекпоинт
        self.add_checkpoint();
        
        Ok(())
    }

    /// Добавляет чекпоинт текущего состояния
    pub fn add_checkpoint(&mut self) {
        let state = self.simulator.get_state();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let event = RecoveryEvent::new(RecoveryEventType::Success);
        self.recovery.add_checkpoint(format!("checkpoint_{}", timestamp), state.clone());
    }

    /// Восстанавливает состояние по идентификатору
    pub fn recover_state(&mut self, state_id: &str) -> Result<(), String> {
        match self.recovery.recover_state(state_id) {
            Ok(state) => {
                self.simulator.set_state(state.clone())?;
                self.state_history.push_back(state);
                if self.state_history.len() > self.max_history_size {
                    self.state_history.pop_front();
                }
                Ok(())
            },
            Err(err) => Err(err)
        }
    }

    /// Выполняет квантовую схему
    pub fn execute_circuit(&mut self) -> Result<SimulationResult, String> {
        // Добавляем чекпоинт перед выполнением схемы
        self.add_checkpoint();
        
        // Выполняем схему
        let result = self.simulator.run_circuit(&self.circuit)?;
        
        // Сохраняем результат и обновляем историю состояний
        self.last_result = Some(result.clone());
        
        let new_state = self.simulator.get_state().clone();
        
        self.state_history.push_back(new_state);
        if self.state_history.len() > self.max_history_size {
            self.state_history.pop_front();
        }
        
        Ok(result)
    }

    /// Создает дельту между двумя состояниями
    pub fn create_delta(&mut self, from_state_id: &str, to_state_id: &str, max_size: usize) -> Result<QuantumDelta, String> {
        // Получаем исходное состояние
        let from_state = match self.recovery.recover_state(from_state_id) {
            Ok(state) => state,
            Err(e) => return Err(format!("Не удалось восстановить исходное состояние: {}", e)),
        };
        
        // Получаем целевое состояние
        let to_state = match self.recovery.recover_state(to_state_id) {
            Ok(state) => state,
            Err(e) => return Err(format!("Не удалось восстановить целевое состояние: {}", e)),
        };
        
        // Создаем дельту между состояниями
        Ok(self.delta_compressor.create_delta_full(&from_state, &to_state, max_size.to_string()))
    }

    /// Применяет дельту к состоянию
    pub fn apply_delta(&mut self, delta: &QuantumDelta, state_id: &str) -> Result<(), String> {
        let state = match self.recovery.recover_state(state_id) {
            Ok(state) => state,
            Err(err) => return Err(err)
        };
        
        let mut new_state = state.clone();
        delta.apply_to(&mut new_state)?;
        
        self.simulator.set_state(new_state.clone())?;
        self.state_history.push_back(new_state);
        if self.state_history.len() > self.max_history_size {
            self.state_history.pop_front();
        }
        
        Ok(())
    }

    /// Возвращает текущее состояние
    pub fn get_state(&self) -> Option<QuantumState> {
        Some(self.simulator.get_state().clone())
    }

    /// Возвращает последний результат симуляции
    pub fn last_result(&self) -> Option<&SimulationResult> {
        self.last_result.as_ref()
    }

    /// Возвращает количество кубитов
    pub fn get_qubit_count(&self) -> usize {
        self.qubit_count
    }

    /// Очищает текущую схему
    pub fn clear_circuit(&mut self) {
        self.circuit = QuantumCircuit::new(self.qubit_count);
    }
    
    /// Добавляет схему из другой схемы
    pub fn add_circuit(&mut self, circuit: QuantumCircuit) -> Result<(), String> {
        if circuit.qubit_count() != self.qubit_count {
            return Err(format!("Количество кубитов в схеме ({}) не соответствует количеству кубитов в адаптере ({})",
                              circuit.qubit_count(), self.qubit_count));
        }
        
        // Объединяем гейты из новой схемы с текущей
        for gate in circuit.get_gates() {
            self.circuit.add_gate(gate.clone());
        }
        
        Ok(())
    }

    /// Создает схему квантового преобразования Фурье
    pub fn create_qft_circuit(&mut self, target_qubits: &[usize]) -> Result<(), String> {
        self.circuit = QuantumCircuit::new(self.qubit_count);
        
        for i in 0..target_qubits.len() {
            let target = target_qubits[i];
            
            // Адамар на текущем кубите
            self.circuit.h(target);
            
            // Контролируемые фазовые вращения
            for j in 1..(target_qubits.len() - i) {
                let control = target_qubits[i + j];
                let theta = PI / (1 << j) as f64;
                self.circuit.controlled_phase(control, target, theta);
            }
        }
        
        // Меняем местами кубиты (последовательность должна быть обращена)
        for i in 0..target_qubits.len() / 2 {
            let q1 = target_qubits[i];
            let q2 = target_qubits[target_qubits.len() - i - 1];
            self.circuit.swap(q1, q2);
        }
        
        Ok(())
    }

    /// Создает схему для генерации состояния GHZ
    pub fn create_ghz_circuit(&mut self, qubits: &[usize]) -> Result<(), String> {
        if qubits.is_empty() {
            return Err("Список кубитов не может быть пустым".to_string());
        }
        
        self.circuit = QuantumCircuit::new(self.qubit_count);
        
        // Адамар на первом кубите
        self.circuit.h(qubits[0]);
        
        // CNOT на остальных кубитах
        for i in 1..qubits.len() {
            self.circuit.cnot(qubits[0], qubits[i]);
        }
        
        Ok(())
    }

    /// Создает схему для квантовой телепортации
    pub fn create_teleportation_circuit(&mut self) -> Result<(), String> {
        if self.qubit_count < 3 {
            return Err("Для телепортации требуется минимум 3 кубита".to_string());
        }
        
        self.circuit = QuantumCircuit::new(self.qubit_count);
        
        // Создаем запутанную пару (кубиты 1 и 2)
        self.circuit.h(1);
        self.circuit.cnot(1, 2);
        
        // Телепортация
        self.circuit.cnot(0, 1);
        self.circuit.h(0);
        
        // Измерение кубитов 0 и 1
        self.circuit.measure(0);
        self.circuit.measure(1);
        
        // Применяем корректирующие операции на кубите 2
        // в зависимости от результатов измерений
        // Это условные операции, которые в реальном квантовом компьютере
        // выполнялись бы на основе классических битов
        
        Ok(())
    }
} 