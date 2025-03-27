use crate::quantum::{Qubit, QrustSimulator, QuantumGate, QuantumOperation};
use crate::quantum::qubit::QubitDelta;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use num_complex::Complex64;

/// Уникальный идентификатор узла
pub type NodeId = String;

/// Метрики производительности узла
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    /// Количество обработанных транзакций
    pub processed_transactions: u64,
    
    /// Среднее время обработки транзакции (мс)
    pub avg_processing_time_ms: f64,
    
    /// Средняя вероятность успешной обработки
    pub avg_success_probability: f64,
    
    /// Средний уровень интерференции
    pub avg_interference: f64,
    
    /// Средний уровень запутанности узла
    pub avg_entanglement_level: f64,
}

/// Состояние узла TRIAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeState {
    /// Узел готов к обработке транзакций
    Ready,
    
    /// Узел в процессе достижения консенсуса
    ConsensusInProgress,
    
    /// Узел в запутанном состоянии с другими узлами
    Entangled { 
        /// С какими узлами запутан
        with_nodes: Vec<NodeId>,
        
        /// Уровень запутанности (0.0 - 1.0)
        entanglement_level: f64,
    },
    
    /// Узел выполняет квантовую операцию
    QuantumOperation { 
        /// Тип выполняемой операции
        operation: String,
        
        /// Прогресс выполнения (0.0 - 1.0)
        progress: f64,
    },
    
    /// Узел недоступен
    Unavailable,
}

/// Узел сети TRIAD
pub struct Node {
    /// Уникальный идентификатор узла
    id: NodeId,
    
    /// Набор кубитов узла
    qubits: Vec<Qubit>,
    
    /// Текущее состояние узла
    state: NodeState,
    
    /// Квантовый симулятор для более точного моделирования
    simulator: QrustSimulator,
    
    /// Метрики производительности узла
    metrics: NodeMetrics,
    
    /// Текущий уровень запутанности (0.0 - 1.0)
    entanglement_level: f64,
    
    /// Связи с другими узлами (nodeId -> индекс кубита)
    entangled_with: HashMap<NodeId, usize>,
}

impl Node {
    /// Создает новый узел с указанным идентификатором и количеством кубитов
    pub fn new(id: &str, qubit_count: usize) -> Self {
        let mut qubits = Vec::with_capacity(qubit_count);
        
        // Инициализируем кубиты в чистом состоянии |0⟩
        for _ in 0..qubit_count {
            qubits.push(Qubit::new());
        }
        
        // Инициализируем квантовый симулятор
        let simulator = QrustSimulator::new(qubit_count);
        
        Self {
            id: id.to_string(),
            qubits,
            state: NodeState::Ready,
            simulator,
            metrics: NodeMetrics::default(),
            entanglement_level: 0.0,
            entangled_with: HashMap::new(),
        }
    }
    
    /// Возвращает идентификатор узла
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Возвращает текущее состояние узла
    pub fn state(&self) -> &NodeState {
        &self.state
    }
    
    /// Возвращает копию текущего состояния узла
    pub fn get_state(&self) -> NodeState {
        self.state.clone()
    }
    
    /// Устанавливает новое состояние узла
    pub fn set_state(&mut self, state: NodeState) {
        self.state = state;
    }
    
    /// Обрабатывает транзакцию и возвращает результирующую "волну" и дельты кубитов
    pub fn process_transaction(&mut self, tx_data: &str) -> (f64, HashMap<usize, QubitDelta>) {
        // Измеряем начальное время
        let start = std::time::Instant::now();
        
        // Получаем хеш транзакции
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        let tx_hash = hasher.finalize();
        
        // Дельты для изменения состояния кубитов
        let mut deltas = HashMap::new();
        
        // Устанавливаем состояние в процессе консенсуса
        self.state = NodeState::ConsensusInProgress;
        
        // Применяем квантовые операции на основе хеша транзакции
        for (i, byte) in tx_hash.iter().enumerate().take(self.qubits.len()) {
            // Определяем операцию на основе значения байта
            let operation = match *byte % 4 {
                0 => QuantumOperation::SingleQubit { 
                    gate: QuantumGate::Hadamard,
                    target: i % self.qubits.len(),
                },
                1 => QuantumOperation::SingleQubit { 
                    gate: QuantumGate::PauliX,
                    target: i % self.qubits.len(),
                },
                2 => QuantumOperation::SingleQubit { 
                    gate: QuantumGate::PauliZ,
                    target: i % self.qubits.len(),
                },
                _ => {
                    // Если в наличие имеется хотя бы два кубита, выполняем CNOT
                    if self.qubits.len() >= 2 {
                        let control = i % self.qubits.len();
                        let target = (i + 1) % self.qubits.len();
                        QuantumOperation::CNOT { control, target }
                    } else {
                        QuantumOperation::SingleQubit { 
                            gate: QuantumGate::PauliY,
                            target: i % self.qubits.len(),
                        }
                    }
                }
            };
            
            // Применяем операцию через симулятор
            self.simulator.apply_operation(&operation);
            
            // Обновляем состояние узла
            self.state = NodeState::QuantumOperation { 
                operation: format!("{:?}", operation),
                progress: (i as f64 + 1.0) / (self.qubits.len() as f64),
            };
            
            // Устанавливаем состояние кубитов на основе симулятора
            if let QuantumOperation::SingleQubit { target, .. } = operation {
                let amplitudes = self.simulator.get_qubit_state(target);
                if let Some(amplitudes) = amplitudes {
                    // Вычисляем дельту относительно текущего состояния кубита
                    let current = &self.qubits[target];
                    let delta_alpha = amplitudes.0 - current.alpha();
                    let delta_beta = amplitudes.1 - current.beta();
                    
                    // Сохраняем дельту для возврата
                    deltas.insert(target, QubitDelta { 
                        delta_alpha,
                        delta_beta,
                    });
                    
                    // Обновляем кубит
                    self.qubits[target] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
                }
            } else if let QuantumOperation::CNOT { control, target } = operation {
                // Для CNOT обновляем оба кубита
                let control_amplitudes = self.simulator.get_qubit_state(control);
                let target_amplitudes = self.simulator.get_qubit_state(target);
                
                if let Some(amplitudes) = control_amplitudes {
                    let current = &self.qubits[control];
                    let delta_alpha = amplitudes.0 - current.alpha();
                    let delta_beta = amplitudes.1 - current.beta();
                    
                    deltas.insert(control, QubitDelta { 
                        delta_alpha,
                        delta_beta,
                    });
                    
                    self.qubits[control] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
                }
                
                if let Some(amplitudes) = target_amplitudes {
                    let current = &self.qubits[target];
                    let delta_alpha = amplitudes.0 - current.alpha();
                    let delta_beta = amplitudes.1 - current.beta();
                    
                    deltas.insert(target, QubitDelta { 
                        delta_alpha,
                        delta_beta,
                    });
                    
                    self.qubits[target] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
                }
            }
        }
        
        // Измеряем запутанность
        let entanglement_score = self.simulator.measure_entanglement();
        self.entanglement_level = entanglement_score.score;
        
        // Если запутанность выше порога, меняем состояние узла
        if self.entanglement_level > 0.5 {
            // Получаем список запутанных узлов (пока просто используем текущие запутанные узлы)
            let entangled_with: Vec<NodeId> = self.entangled_with.keys().cloned().collect();
            
            self.state = NodeState::Entangled { 
                with_nodes: entangled_with,
                entanglement_level: self.entanglement_level,
            };
        } else {
            // Возвращаем готовность узла
            self.state = NodeState::Ready;
        }
        
        // Рассчитываем результирующую "волну" для консенсуса
        let wave = self.calculate_wave_value(tx_data);
        
        // Обновляем метрики
        let elapsed = start.elapsed();
        let processing_time_ms = elapsed.as_secs_f64() * 1000.0;
        
        self.metrics.processed_transactions += 1;
        let total_time = self.metrics.avg_processing_time_ms * (self.metrics.processed_transactions - 1) as f64;
        self.metrics.avg_processing_time_ms = (total_time + processing_time_ms) / self.metrics.processed_transactions as f64;
        
        let total_entanglement = self.metrics.avg_entanglement_level * (self.metrics.processed_transactions - 1) as f64;
        self.metrics.avg_entanglement_level = (total_entanglement + self.entanglement_level) / self.metrics.processed_transactions as f64;
        
        (wave, deltas)
    }
    
    /// Применяет дельты от других узлов к своим кубитам (моделирует запутанность)
    pub fn apply_deltas(&mut self, deltas: &HashMap<usize, QubitDelta>) {
        // Обновляем состояние кубитов
        for (idx, delta) in deltas {
            if *idx < self.qubits.len() {
                let qubit = &mut self.qubits[*idx];
                
                // Применяем дельты к амплитудам кубита
                let new_alpha = qubit.alpha() + delta.delta_alpha;
                let new_beta = qubit.beta() + delta.delta_beta;
                
                // Заменяем кубит на новый с обновленными амплитудами
                *qubit = Qubit::from_amplitudes(new_alpha, new_beta);
                
                // Также обновляем состояние в симуляторе
                self.simulator.set_qubit_state(*idx, Complex64::new(new_alpha.re, new_alpha.im), Complex64::new(new_beta.re, new_beta.im));
            }
        }
        
        // Повторно измеряем запутанность после применения дельт
        let entanglement_score = self.simulator.measure_entanglement();
        self.entanglement_level = entanglement_score.score;
        
        // Обновляем состояние узла в зависимости от уровня запутанности
        if self.entanglement_level > 0.5 {
            // В реальной системе здесь бы проверялось, с какими именно узлами произошла запутанность
            let entangled_nodes = Vec::new(); // Заглушка
            
            self.state = NodeState::Entangled {
                with_nodes: entangled_nodes,
                entanglement_level: self.entanglement_level,
            };
        }
    }
    
    /// Рассчитывает значение "волны" для консенсуса на основе состояния кубитов
    fn calculate_wave_value(&self, tx_data: &str) -> f64 {
        // Рассчитываем интерференцию между кубитами
        let mut interference = 0.0;
        
        // Если мы используем квантовый симулятор, получаем более точное значение
        let simulator_interference = self.simulator.measure_interference();
        if simulator_interference != 0.0 {
            return simulator_interference;
        }
        
        // Если симулятор не дал результата, делаем упрощенный расчет
        for i in 0..self.qubits.len() {
            for j in (i+1)..self.qubits.len() {
                let q1 = &self.qubits[i];
                let q2 = &self.qubits[j];
                
                // Простая модель интерференции между двумя кубитами
                let phase_diff = (q1.alpha() * q2.alpha().conj() + q1.beta() * q2.beta().conj()).arg();
                
                // Вычисляем интерференцию
                interference += phase_diff.cos();
            }
        }
        
        // Нормализуем интерференцию
        if self.qubits.len() > 1 {
            let pairs = (self.qubits.len() * (self.qubits.len() - 1)) / 2;
            interference /= pairs as f64;
        } else {
            // Если всего один кубит, используем его вероятность |1⟩
            interference = self.qubits[0].probability_one();
        }
        
        // Добавляем вклад от данных транзакции
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        let tx_hash = hasher.finalize();
        
        // Используем первый байт хеша для модуляции результата
        let modulation = (tx_hash[0] as f64) / 255.0;
        
        // Окончательная волна является комбинацией интерференции и модуляции от транзакции
        (interference * 0.8 + modulation * 0.2) * 2.0 - 1.0  // Нормализуем к [-1, 1]
    }
    
    /// Создает запутанность с другим узлом
    pub fn entangle_with(&mut self, other_node_id: &str, qubit_idx: usize) {
        if qubit_idx < self.qubits.len() {
            // Сохраняем информацию о запутанности
            self.entangled_with.insert(other_node_id.to_string(), qubit_idx);
            
            // Применяем операцию Адамара к кубиту для создания суперпозиции
            let operation = QuantumOperation::SingleQubit { 
                gate: QuantumGate::Hadamard,
                target: qubit_idx,
            };
            self.simulator.apply_operation(&operation);
            
            // Обновляем состояние кубита
            if let Some(amplitudes) = self.simulator.get_qubit_state(qubit_idx) {
                self.qubits[qubit_idx] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
            }
            
            // Обновляем состояние узла
            let entanglement_score = self.simulator.measure_entanglement();
            self.entanglement_level = entanglement_score.score;
            if self.entanglement_level > 0.0 {
                let entangled_with: Vec<NodeId> = self.entangled_with.keys().cloned().collect();
                self.state = NodeState::Entangled {
                    with_nodes: entangled_with,
                    entanglement_level: self.entanglement_level,
                };
            }
        }
    }
    
    /// Применяет гейт Адамара к указанному кубиту
    pub fn apply_hadamard(&mut self, qubit_idx: usize) {
        if qubit_idx < self.qubits.len() {
            let operation = QuantumOperation::SingleQubit { 
                gate: QuantumGate::Hadamard,
                target: qubit_idx,
            };
            self.simulator.apply_operation(&operation);
            
            // Обновляем состояние кубита
            if let Some(amplitudes) = self.simulator.get_qubit_state(qubit_idx) {
                self.qubits[qubit_idx] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
            }
        }
    }
    
    /// Применяет гейт NOT (PauliX) к указанному кубиту
    pub fn apply_not(&mut self, qubit_idx: usize) {
        if qubit_idx < self.qubits.len() {
            let operation = QuantumOperation::SingleQubit { 
                gate: QuantumGate::PauliX,
                target: qubit_idx,
            };
            self.simulator.apply_operation(&operation);
            
            // Обновляем состояние кубита
            if let Some(amplitudes) = self.simulator.get_qubit_state(qubit_idx) {
                self.qubits[qubit_idx] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
            }
        }
    }
    
    /// Применяет гейт CNOT между двумя кубитами
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        if control < self.qubits.len() && target < self.qubits.len() {
            let operation = QuantumOperation::CNOT { control, target };
            self.simulator.apply_operation(&operation);
            
            // Обновляем состояния кубитов
            if let Some(amplitudes) = self.simulator.get_qubit_state(control) {
                self.qubits[control] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
            }
            if let Some(amplitudes) = self.simulator.get_qubit_state(target) {
                self.qubits[target] = Qubit::from_amplitudes(amplitudes.0, amplitudes.1);
            }
        }
    }
    
    /// Возвращает копию кубита по индексу
    pub fn get_qubit(&self, idx: usize) -> Option<Qubit> {
        if idx < self.qubits.len() {
            Some(self.qubits[idx].clone())
        } else {
            None
        }
    }
    
    /// Возвращает ссылку на метрики узла
    pub fn metrics(&self) -> &NodeMetrics {
        &self.metrics
    }
    
    /// Возвращает текущий уровень запутанности узла
    pub fn entanglement_level(&self) -> f64 {
        self.entanglement_level
    }
    
    /// Измеряет интерференцию между кубитами узла
    pub fn measure_interference(&self) -> f64 {
        self.simulator.measure_interference()
    }
} 