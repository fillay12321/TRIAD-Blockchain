use crate::quantum::{QrustSimulator, QuantumGate, EntanglementScore};
use crate::quantum::triadvantum::{
    state::QuantumState, 
    circuit::QuantumCircuit, 
    simulator::SimulationResult,
    simulator::SimulationStats,
    operators::QuantumOperator, 
    operators::MeasurementOperator, 
    operators::Operator, 
    delta::QuantumDelta, 
    interference::InterferencePattern
};
use crate::quantum::triadvantum_adapter::TriadVantumAdapter;
use num_complex::Complex64;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

/// Квантовая волновая функция, описывающая состояние поля
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumWaveFunction {
    /// Амплитуды состояний (нормализованные)
    pub amplitudes: Vec<Complex64>,
    /// Размерность пространства (количество кубитов)
    pub dimension: usize,
}

/// Состояние квантового поля
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldState {
    /// Однородное поле без запутанности
    Uniform,
    /// Состояние с областями запутанности
    Entangled {
        /// Группы запутанных узлов
        entanglement_groups: Vec<HashSet<usize>>,
        /// Общий уровень запутанности
        entanglement_level: f64,
    },
    /// Состояние с интерференцией
    Interfering {
        /// Интерференционный паттерн
        pattern: InterferencePattern,
    },
    /// Декогерентное состояние (подверженное шуму)
    Decoherent {
        /// Уровень декогеренции (0-1)
        decoherence_level: f64,
    },
}

/// Представляет квантовую интерференцию между узлами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumInterference {
    /// Узлы, участвующие в интерференции
    pub nodes: Vec<usize>,
    /// Результирующий интерференционный паттерн
    pub pattern: InterferencePattern,
    /// Достигнут ли консенсус
    pub consensus_reached: bool,
    /// Вероятность консенсуса (0-1)
    pub consensus_probability: f64,
}

impl Default for QuantumInterference {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            pattern: InterferencePattern::new(String::new(), QuantumState::new(1), QuantumState::new(1)),
            consensus_reached: false,
            consensus_probability: 0.0,
        }
    }
}

impl QuantumInterference {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn calculate_interference(&self, state1: &QuantumState, state2: &QuantumState) -> f64 {
        // Расчет фактора интерференции между двумя состояниями
        // Простая реализация: проверка на ортогональность состояний
        let mut interference_factor = 0.0;
        let mut total_qubits = 0;
        
        for i in 0..state1.qubit_count.min(state2.qubit_count) {
            let state1_amplitude = state1.get_amplitude(i);
            let state2_amplitude = state2.get_amplitude(i);
            
            if state1_amplitude.norm() > 0.0 && state2_amplitude.norm() > 0.0 {
                total_qubits += 1;
                let amp1 = state1_amplitude;
                let amp2 = state2_amplitude;
                
                // Скалярное произведение комплексных амплитуд (исправлено)
                let dot_product = amp1.re * amp2.re + amp1.im * amp2.im;
                let magnitude = dot_product.abs();  // Используем abs() вместо norm()
                
                // Нормализованный фактор интерференции (0 - полная ортогональность, 1 - полное совпадение)
                interference_factor += magnitude;
            }
        }
        
        if total_qubits > 0 {
            interference_factor /= total_qubits as f64;
        }
        
        interference_factor
    }
    
    pub fn apply_interference(&self, result_state: &QuantumState, incoming_state: &QuantumState, factor: f64) -> QuantumState {
        // Применяем интерференцию, смешивая состояния с весом, зависящим от фактора интерференции
        let mut new_state = result_state.clone();
        
        // Если состояния полностью ортогональны, возвращаем исходное состояние
        if factor < 0.01 {
            return new_state;
        }
        
        for i in 0..result_state.qubit_count.min(incoming_state.qubit_count) {
            let state1_amplitude = result_state.get_amplitude(i);
            let state2_amplitude = incoming_state.get_amplitude(i);
            
            if state1_amplitude.norm() > 0.0 && state2_amplitude.norm() > 0.0 {
                let amp1 = state1_amplitude;
                let amp2 = state2_amplitude;
                
                // Выполняем суперпозицию с весами, зависящими от фактора интерференции
                let weight1 = (1.0 - factor).sqrt();
                let weight2 = factor.sqrt();
                
                // Новые амплитуды как взвешенная суперпозиция
                let new_amp0 = weight1 * amp1.re + weight2 * amp2.re;
                let new_amp1 = weight1 * amp1.im + weight2 * amp2.im;
                
                // Нормализуем
                let norm = (new_amp0 * new_amp0 + new_amp1 * new_amp1).sqrt();
                let norm_amp0 = new_amp0 / norm;
                let norm_amp1 = new_amp1 / norm;
                
                // Устанавливаем новое состояние кубита
                new_state.set_amplitude(i, Complex64::new(norm_amp0, norm_amp1));
            }
        }
        
        new_state
    }
}

/// Распределенное квантовое поле сети TRIAD
pub struct QuantumField {
    /// Общий квантовый симулятор для всего поля
    legacy_simulator: QrustSimulator,
    /// Современный симулятор TriadVantum
    triadvantum_adapter: TriadVantumAdapter,
    /// Отображение физических узлов на кубиты в симуляторе
    node_to_qubits: HashMap<usize, Vec<usize>>,
    /// Текущее состояние поля
    state: FieldState,
    /// Запутанные пары кубитов (между узлами)
    entangled_pairs: Vec<(usize, usize)>,
    /// Волновая функция поля
    wave_function: QuantumWaveFunction,
    /// Флаг для выбора симулятора (true - использовать TriadVantum)
    use_triadvantum: bool,
}

impl QuantumField {
    /// Создает новое квантовое поле с указанным количеством узлов
    /// и кубитов на каждый узел
    pub fn new(node_count: usize, qubits_per_node: usize) -> Self {
        // Общее количество кубитов в системе
        let total_qubits = node_count * qubits_per_node;
        
        // Создаем симуляторы
        let legacy_simulator = QrustSimulator::new(total_qubits);
        let triadvantum_adapter = TriadVantumAdapter::new(total_qubits, "main_field".to_string());
        
        // Отображаем узлы на диапазоны кубитов
        let mut node_to_qubits = HashMap::new();
        for node_idx in 0..node_count {
            let start_qubit = node_idx * qubits_per_node;
            let qubits = (start_qubit..(start_qubit + qubits_per_node)).collect();
            node_to_qubits.insert(node_idx, qubits);
        }
        
        // Начальная волновая функция - все в состоянии |0...0>
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); 1 << total_qubits];
        amplitudes[0] = Complex64::new(1.0, 0.0);
        
        Self {
            legacy_simulator,
            triadvantum_adapter,
            node_to_qubits,
            state: FieldState::Uniform,
            entangled_pairs: Vec::new(),
            wave_function: QuantumWaveFunction {
                amplitudes,
                dimension: total_qubits,
            },
            use_triadvantum: true, // По умолчанию используем новый симулятор
        }
    }
    
    /// Установить использование симулятора TriadVantum
    pub fn set_use_triadvantum(&mut self, use_it: bool) {
        self.use_triadvantum = use_it;
    }
    
    /// Применяет вентиль Адамара к указанному кубиту указанного узла
    pub fn apply_hadamard(&mut self, node_idx: usize, qubit_local_idx: usize) {
        if let Some(qubits) = self.node_to_qubits.get(&node_idx) {
            if qubit_local_idx < qubits.len() {
                let global_qubit_idx = qubits[qubit_local_idx];
                
                if self.use_triadvantum {
                    // Используем новый симулятор TriadVantum
                    self.triadvantum_adapter.clear_circuit();
                    self.triadvantum_adapter.circuit.h(global_qubit_idx);
                    self.triadvantum_adapter.execute_circuit();
                } else {
                    // Используем старый симулятор
                    self.legacy_simulator.apply_gate(QuantumGate::Hadamard, &[global_qubit_idx]);
                }
                
                self.update_state();
            }
        }
    }
    
    /// Применяет вентиль NOT (X) к указанному кубиту указанного узла
    pub fn apply_not(&mut self, node_idx: usize, qubit_local_idx: usize) {
        if let Some(qubits) = self.node_to_qubits.get(&node_idx) {
            if qubit_local_idx < qubits.len() {
                let global_qubit_idx = qubits[qubit_local_idx];
                
                if self.use_triadvantum {
                    // Используем новый симулятор TriadVantum
                    self.triadvantum_adapter.clear_circuit();
                    self.triadvantum_adapter.circuit.x(global_qubit_idx);
                    self.triadvantum_adapter.execute_circuit();
                } else {
                    // Используем старый симулятор
                    self.legacy_simulator.apply_gate(QuantumGate::PauliX, &[global_qubit_idx]);
                }
                
                self.update_state();
            }
        }
    }
    
    /// Создает запутанность между кубитами разных узлов
    pub fn entangle_nodes(&mut self, node1: usize, qubit1: usize, node2: usize, qubit2: usize) {
        let global_qubit1 = self.get_global_qubit_idx(node1, qubit1);
        let global_qubit2 = self.get_global_qubit_idx(node2, qubit2);
        
        if let (Some(q1), Some(q2)) = (global_qubit1, global_qubit2) {
            if self.use_triadvantum {
                // Используем новый симулятор для создания пары Белла
                self.triadvantum_adapter.clear_circuit();
                self.triadvantum_adapter.circuit.h(q1);
                self.triadvantum_adapter.circuit.cnot(q1, q2);
                self.triadvantum_adapter.execute_circuit();
            } else {
                // Используем старый симулятор для создания пары Белла
                self.legacy_simulator.apply_gate(QuantumGate::Hadamard, &[q1]);
                self.legacy_simulator.apply_gate(QuantumGate::CNOT, &[q1, q2]);
            }
            
            // Добавляем пару в список запутанных
            self.entangled_pairs.push((q1, q2));
            
            // Обновляем состояние поля
            self.update_state();
        }
    }
    
    /// Возвращает глобальный индекс кубита по индексу узла и локальному индексу кубита
    fn get_global_qubit_idx(&self, node_idx: usize, qubit_local_idx: usize) -> Option<usize> {
        self.node_to_qubits.get(&node_idx).and_then(|qubits| {
            if qubit_local_idx < qubits.len() {
                Some(qubits[qubit_local_idx])
            } else {
                None
            }
        })
    }
    
    /// Обновляет состояние квантового поля на основе текущего состояния симулятора
    fn update_state(&mut self) {
        if self.use_triadvantum {
            // Получаем текущее состояние из симулятора TriadVantum
            if let Some(current_state) = self.triadvantum_adapter.get_state() {
                // Обновляем волновую функцию
                self.wave_function.amplitudes = current_state.get_amplitudes().to_vec();
                
                // Определяем новое состояние поля
                let entanglement_score = self.legacy_simulator.measure_entanglement();
                if entanglement_score.score > 0.1 {
                    // Есть существенная запутанность
                    let mut groups: Vec<HashSet<usize>> = Vec::new();
                    for pair in &entanglement_score.qubit_pairs {
                        let (q1, q2) = *pair;
                        let mut found = false;
                        
                        // Ищем существующую группу, куда входит хотя бы один из кубитов
                        for group in &mut groups {
                            if group.contains(&q1) || group.contains(&q2) {
                                group.insert(q1);
                                group.insert(q2);
                                found = true;
                                break;
                            }
                        }
                        
                        // Если не нашли, создаем новую группу
                        if !found {
                            let mut new_group = HashSet::new();
                            new_group.insert(q1);
                            new_group.insert(q2);
                            groups.push(new_group);
                        }
                    }
                    
                    self.state = FieldState::Entangled {
                        entanglement_groups: groups,
                        entanglement_level: entanglement_score.score,
                    };
                } else {
                    // Проверяем на интерференцию
                    let interference = self.calculate_interference_level();
                    if interference > 0.1 {
                        // Есть существенная интерференция
                        let pattern = self.calculate_interference_pattern(0, 0);
                        
                        self.state = FieldState::Interfering { pattern };
                    } else {
                        // Проверяем на декогеренцию
                        let decoherence = self.calculate_decoherence_level();
                        if decoherence > 0.2 {
                            // Есть существенная декогеренция
                            self.state = FieldState::Decoherent {
                                decoherence_level: decoherence,
                            };
                        } else {
                            // Однородное поле
                            self.state = FieldState::Uniform;
                        }
                    }
                }
            }
        } else {
            // Используем старый симулятор (логика аналогична)
            // ...
        }
    }
    
    /// Вычисляет интерференционный паттерн на основе текущего состояния поля
    fn calculate_interference_pattern(&self, qubit1: usize, qubit2: usize) -> InterferencePattern {
        // Получаем состояние из адаптера
        let current_state = self.triadvantum_adapter.get_state().unwrap_or_else(|| QuantumState::new(1));
        
        // Создаем паттерн интерференции
        InterferencePattern::new(
            format!("interference_{}", SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()),
            current_state.clone(),
            current_state.clone()
        )
    }
    
    /// Применяет операцию обработки транзакции к полю
    pub fn process_transaction(&mut self, node_idx: usize, tx_data: &str) -> QuantumInterference {
        use sha2::{Sha256, Digest};
        
        // Используем хеш транзакции для выбора кубитов и операций
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        hasher.update(&node_idx.to_le_bytes());
        let hash = hasher.finalize();
        
        if let Some(qubits) = self.node_to_qubits.get(&node_idx) {
            // Выбираем кубиты для операции на основе хеша
            let qubit1_idx = (hash[0] as usize) % qubits.len();
            let _qubit2_idx = (hash[1] as usize) % qubits.len();
            let global_qubit1 = qubits[qubit1_idx];
            
            if self.use_triadvantum {
                // Используем TriadVantum
                self.triadvantum_adapter.clear_circuit();
                
                // Применяем Адамар к первому кубиту
                self.triadvantum_adapter.circuit.h(global_qubit1);
                
                // Если есть другие узлы, выбираем случайный для запутывания
                if self.node_to_qubits.len() > 1 {
                    let mut other_node_idx = node_idx;
                    let mut attempts = 0;
                    // Ограничиваем количество попыток, чтобы избежать бесконечного цикла
                    while other_node_idx == node_idx && attempts < 10 {
                        other_node_idx = (hash[2] as usize) % self.node_to_qubits.len();
                        attempts += 1;
                    }
                    
                    // Если после всех попыток не удалось выбрать другой узел, используем
                    // первый отличный от текущего или просто следующий
                    if other_node_idx == node_idx {
                        for (idx, _) in &self.node_to_qubits {
                            if *idx != node_idx {
                                other_node_idx = *idx;
                                break;
                            }
                        }
                    }
                    
                    if let Some(other_qubits) = self.node_to_qubits.get(&other_node_idx) {
                        let other_qubit_idx = (hash[3] as usize) % other_qubits.len();
                        let global_qubit2 = other_qubits[other_qubit_idx];
                        
                        // Запутываем кубиты разных узлов через CNOT
                        self.triadvantum_adapter.circuit.cnot(global_qubit1, global_qubit2);
                    }
                }
                
                // Выполняем схему
                let result = self.triadvantum_adapter.execute_circuit().unwrap_or_else(|_| {
                    // Создаем пустой результат с нужными полями final_state и stats
                    SimulationResult {
                        measurements: HashMap::new(),
                        probabilities: HashMap::new(),
                        final_state: QuantumState::new(1),
                        stats: SimulationStats {
                            gates_applied: 0,
                            measurements: 0,
                            errors: 0,
                            execution_time: 0.0,
                            state_history: Vec::new()
                        }
                    }
                });
                
                // Получаем состояние после выполнения
                let last_state = match self.triadvantum_adapter.get_state() {
                    Some(state) => state,
                    None => QuantumState::new(1)
                };
                
                // Обновляем состояние поля
                self.update_state();
                
                // Определяем, достигнут ли консенсус
                let consensus_reached = self.check_consensus();
                let consensus_probability = self.calculate_consensus_probability();
                
                // Вычисляем паттерн интерференции
                let pattern = match &self.state {
                    FieldState::Interfering { pattern } => pattern.clone(),
                    _ => {
                        if !self.entangled_pairs.is_empty() {
                            self.calculate_interference_pattern(self.entangled_pairs[0].0, self.entangled_pairs[0].1)
                        } else {
                            self.calculate_interference_pattern(0, 0)
                        }
                    }
                };
                
                // Определяем узлы, участвующие в интерференции
                let mut nodes = vec![node_idx];
                for (q1, q2) in &self.entangled_pairs {
                    // Находим, к каким узлам принадлежат эти кубиты
                    for (n_idx, n_qubits) in &self.node_to_qubits {
                        if n_qubits.contains(q1) || n_qubits.contains(q2) {
                            if !nodes.contains(n_idx) {
                                nodes.push(*n_idx);
                            }
                        }
                    }
                }
                
                QuantumInterference {
                    nodes,
                    pattern,
                    consensus_reached,
                    consensus_probability,
                }
            } else {
                // Используем старый симулятор
                // Применяем Адамар к первому кубиту
                self.legacy_simulator.apply_gate(QuantumGate::Hadamard, &[global_qubit1]);
                
                // Если есть другие узлы, выбираем случайный для запутывания
                if self.node_to_qubits.len() > 1 {
                    let mut other_node_idx = node_idx;
                    let mut attempts = 0;
                    // Ограничиваем количество попыток, чтобы избежать бесконечного цикла
                    while other_node_idx == node_idx && attempts < 10 {
                        other_node_idx = (hash[2] as usize) % self.node_to_qubits.len();
                        attempts += 1;
                    }
                    
                    // Если после всех попыток не удалось выбрать другой узел, используем
                    // первый отличный от текущего или просто следующий
                    if other_node_idx == node_idx {
                        for (idx, _) in &self.node_to_qubits {
                            if *idx != node_idx {
                                other_node_idx = *idx;
                                break;
                            }
                        }
                    }
                    
                    if let Some(other_qubits) = self.node_to_qubits.get(&other_node_idx) {
                        let other_qubit_idx = (hash[3] as usize) % other_qubits.len();
                        let global_qubit2 = other_qubits[other_qubit_idx];
                        
                        // Запутываем кубиты разных узлов через CNOT
                        self.legacy_simulator.apply_gate(QuantumGate::CNOT, &[global_qubit1, global_qubit2]);
                    }
                }
                
                // Обновляем состояние поля
                self.update_state();
                
                // Определяем, достигнут ли консенсус
                let consensus_reached = self.check_consensus();
                let consensus_probability = self.calculate_consensus_probability();
                
                // Вычисляем паттерн интерференции
                let pattern = self.calculate_interference_pattern(self.entangled_pairs[0].0, self.entangled_pairs[0].1);
                
                // Определяем узлы, участвующие в интерференции
                let mut nodes = vec![node_idx];
                for (q1, q2) in &self.entangled_pairs {
                    // Находим, к каким узлам принадлежат эти кубиты
                    for (n_idx, n_qubits) in &self.node_to_qubits {
                        if n_qubits.contains(q1) || n_qubits.contains(q2) {
                            if !nodes.contains(n_idx) {
                                nodes.push(*n_idx);
                            }
                        }
                    }
                }
                
                QuantumInterference {
                    nodes,
                    pattern,
                    consensus_reached,
                    consensus_probability,
                }
            }
        } else {
            // Узел не найден, возвращаем пустую интерференцию
            QuantumInterference {
                nodes: vec![],
                pattern: InterferencePattern::new(
                    "empty".to_string(),
                    QuantumState::new(1),
                    QuantumState::new(1)
                ),
                consensus_reached: false,
                consensus_probability: 0.0,
            }
        }
    }
    
    /// Проверяет, достигнут ли консенсус в текущем состоянии поля
    fn check_consensus(&self) -> bool {
        match &self.state {
            FieldState::Entangled { entanglement_level, .. } => {
                // Консенсус достигается при высоком уровне запутанности
                *entanglement_level > 0.8
            },
            FieldState::Interfering { pattern } => {
                // Консенсус достигается при сильной интерференции
                pattern.strength() > 0.75
            },
            _ => false,
        }
    }
    
    /// Вычисляет вероятность достижения консенсуса
    fn calculate_consensus_probability(&self) -> f64 {
        match &self.state {
            FieldState::Entangled { entanglement_level, .. } => {
                // Вероятность пропорциональна уровню запутанности
                *entanglement_level
            },
            FieldState::Interfering { pattern } => {
                // Вероятность пропорциональна силе интерференции
                pattern.strength()
            },
            FieldState::Decoherent { decoherence_level } => {
                // При высокой декогеренции вероятность консенсуса низкая
                1.0 - *decoherence_level
            },
            FieldState::Uniform => 0.5, // Нейтральная вероятность
        }
    }
    
    /// Возвращает текущее состояние квантового поля
    pub fn get_state(&self) -> &FieldState {
        &self.state
    }
    
    /// Возвращает список запутанных пар кубитов
    pub fn get_entangled_pairs(&self) -> &[(usize, usize)] {
        &self.entangled_pairs
    }
    
    /// Возвращает количество узлов в квантовом поле
    pub fn node_count(&self) -> usize {
        self.node_to_qubits.len()
    }
    
    /// Вычисляет "волну консенсуса" - это амплитуда вероятности, с которой
    /// все узлы приходят к согласию
    pub fn calculate_consensus_wave(&self) -> f64 {
        match &self.state {
            FieldState::Uniform => 0.5, // Нейтральное значение
            FieldState::Entangled { entanglement_level, .. } => {
                // Волна консенсуса пропорциональна запутанности
                *entanglement_level
            },
            FieldState::Interfering { pattern } => {
                // Волна консенсуса модулируется интерференцией
                // Вычисляем среднее абсолютное значение амплитуд
                let interference_matrix = pattern.interference_matrix.iter()
                    .map(|(_, amp)| amp.norm())
                    .sum::<f64>();
                
                if pattern.interference_matrix.is_empty() {
                    0.0
                } else {
                    pattern.strength() * interference_matrix / pattern.interference_matrix.len() as f64
                }
            },
            FieldState::Decoherent { decoherence_level } => {
                // При высокой декогеренции волна консенсуса затухает
                0.5 * (1.0 - *decoherence_level)
            },
        }
    }
    
    /// Создает демонстрационную схему используя TriadVantum
    pub fn create_demo_circuit(&mut self, demo_type: &str) -> bool {
        if !self.use_triadvantum {
            // Эта функция доступна только при использовании TriadVantum
            return false;
        }
        
        // Создаем соответствующую схему в зависимости от типа
        match demo_type {
            "qft" => {
                if let Ok(_) = self.triadvantum_adapter.create_qft_circuit() {
                    return true;
                }
            },
            "bell" => {
                self.triadvantum_adapter.clear_circuit();
                self.triadvantum_adapter.circuit.h(0);
                self.triadvantum_adapter.circuit.cnot(0, 1);
                if let Ok(_) = self.triadvantum_adapter.execute_circuit() {
                    return true;
                }
            },
            _ => return false
        }
        
        false
    }
    
    /// Применяет квантовую дельту к состоянию поля
    pub fn apply_quantum_delta(&mut self, delta: &QuantumDelta) -> bool {
        if !self.use_triadvantum {
            // Эта функция доступна только при использовании TriadVantum
            return false;
        }
        
        // Получаем текущее состояние
        if let Ok(result) = self.triadvantum_adapter.execute_circuit() {
            // Извлекаем состояние из результата
            if let Some(mut current_state) = self.triadvantum_adapter.get_state() {
                // Применяем дельту
                if let Ok(_) = self.triadvantum_adapter.apply_delta(delta, &mut current_state) {
                    // Обновляем состояние поля
                    self.update_state();
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Вычисляет запутанность между указанными узлами
    pub fn calculate_entanglement(&mut self, node1: usize, node2: usize) -> f64 {
        let node_total = self.node_to_qubits.len();
        if node1 >= node_total || node2 >= node_total {
            return 0.0;
        }
        
        // Общая оценка запутанности
        let mut entanglement_sum = 0.0;
        let mut pair_count = 0;
        
        // Проверяем что оба узла существуют в карте
        if !self.node_to_qubits.contains_key(&node1) || !self.node_to_qubits.contains_key(&node2) {
            return 0.0;
        }
        
        // Рассчитываем запутанность для каждой пары кубитов
        for i in 0..self.node_to_qubits[&node1].len() {
            for j in 0..self.node_to_qubits[&node2].len() {
                let global_qubit1 = self.node_to_qubits[&node1][i];
                let global_qubit2 = self.node_to_qubits[&node2][j];
                
                let entanglement = self.calculate_pair_entanglement(global_qubit1, global_qubit2);
                entanglement_sum += entanglement;
                pair_count += 1;
            }
        }
        
        // Средняя запутанность между всеми парами кубитов
        if pair_count > 0 {
            entanglement_sum / pair_count as f64
        } else {
            0.0
        }
    }

    /// Выполняет восстановление состояния при ошибке
    pub fn recover_state(&mut self) -> bool {
        if !self.use_triadvantum {
            // Эта функция доступна только при использовании TriadVantum
            return false;
        }
        
        if let Some(state) = self.triadvantum_adapter.recover_state() {
            // Если успешно восстановили состояние, выполняем пустую схему,
            // чтобы обновить результат
            self.triadvantum_adapter.clear_circuit();
            self.triadvantum_adapter.circuit.h(0); // Просто для проверки
            self.triadvantum_adapter.execute_circuit();
            
            // Обновляем состояние поля
            self.update_state();
            true
        } else {
            false
        }
    }
    
    /// Рассчитывает запутанность между двумя узлами
    pub fn calculate_entanglement_between_nodes(&self, node1: usize, node2: usize) -> Vec<(usize, usize, f64)> {
        let mut entanglement_scores = Vec::new();
        
        // Получаем кубиты для каждого узла
        if let (Some(qubits1), Some(qubits2)) = (self.node_to_qubits.get(&node1), self.node_to_qubits.get(&node2)) {
            // Для каждой пары кубитов проверяем запутанность
            for &q1 in qubits1 {
                for &q2 in qubits2 {
                    // Проверяем, является ли эта пара запутанной
                    let mut entanglement_level = 0.0;
                    
                    if self.use_triadvantum {
                        if let Some(state) = self.triadvantum_adapter.get_state() {
                            entanglement_level = state.calculate_pair_entanglement(q1, q2);
                        }
                    } else {
                        let entanglement = self.legacy_simulator.measure_entanglement();
                        for &(pair_q1, pair_q2) in &entanglement.qubit_pairs {
                            if (pair_q1 == q1 && pair_q2 == q2) || (pair_q1 == q2 && pair_q2 == q1) {
                                entanglement_level = entanglement.score;
                                break;
                            }
                        }
                    }
                    
                    if entanglement_level > 0.01 {
                        entanglement_scores.push((q1, q2, entanglement_level));
                    }
                }
            }
        }
        
        entanglement_scores
    }
    
    /// Добавляю метод для расчета запутанности между парой кубитов
    fn calculate_pair_entanglement(&self, qubit1: usize, qubit2: usize) -> f64 {
        if self.use_triadvantum {
            if let Some(state) = self.triadvantum_adapter.get_state() {
                return state.calculate_pair_entanglement(qubit1, qubit2);
            }
        } else {
            let entanglement = self.legacy_simulator.measure_entanglement();
            for &(q1, q2) in &entanglement.qubit_pairs {
                if (q1 == qubit1 && q2 == qubit2) || (q1 == qubit2 && q2 == qubit1) {
                    return entanglement.score;
                }
            }
        }
        0.0
    }
    
    /// Проверяет наличие запутанности между глобальными кубитами
    pub fn check_entanglement(&self, global_qubit1: usize, global_qubit2: usize) -> Option<f64> {
        let entanglement = self.legacy_simulator.measure_entanglement();
        // Проверяем наличие запутанности между указанными кубитами
        for &(q1, q2) in &entanglement.qubit_pairs {
            if (q1 == global_qubit1 && q2 == global_qubit2) || (q1 == global_qubit2 && q2 == global_qubit1) {
                return Some(entanglement.score);
            }
        }
        None
    }

    /// Добавляем новый метод для расчета уровня интерференции
    fn calculate_interference_level(&self) -> f64 {
        // Простая реализация - можно улучшить
        if let Some(state) = self.triadvantum_adapter.get_state() {
            let amplitudes = state.get_amplitudes();
            let mut sum = 0.0;
            for amp in amplitudes {
                sum += amp.norm_sqr();
            }
            if amplitudes.len() > 1 {
                // Мера интерференции - насколько распределение отличается от равномерного
                1.0 - (sum / amplitudes.len() as f64)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Добавляем новый метод для расчета уровня декогеренции
    fn calculate_decoherence_level(&self) -> f64 {
        // Простая реализация - можно улучшить
        if let Some(state) = self.triadvantum_adapter.get_state() {
            let amplitudes = state.get_amplitudes();
            let mut sum = 0.0;
            for amp in amplitudes {
                // В декогерентном состоянии мнимая часть обычно уменьшается
                sum += amp.im.abs() / (amp.norm() + 1e-10);
            }
            if amplitudes.len() > 1 {
                // Нормализуем
                sum / amplitudes.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
} 