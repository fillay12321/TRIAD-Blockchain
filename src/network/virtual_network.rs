use crate::network::node::{Node, NodeState};
use crate::network::NodeId;
use crate::quantum::QuantumField;
use crate::quantum::qubit::QubitDelta;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use rand::Rng;
use serde::{Serialize, Deserialize};

/// Конфигурация виртуальной сети
pub struct NetworkConfig {
    /// Задержка передачи данных между узлами (мс)
    pub network_delay_ms: u64,
    
    /// Вероятность потери пакета (0.0 - 1.0)
    pub packet_loss_probability: f64,
    
    /// Топология сети (который узел с каким соединен)
    pub topology: NetworkTopology,
    
    /// Использовать ли квантовое поле для моделирования запутанности
    pub use_quantum_field: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_delay_ms: 10,
            packet_loss_probability: 0.01,
            topology: NetworkTopology::FullMesh,
            use_quantum_field: true,
        }
    }
}

/// Топология сети
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkTopology {
    /// Полносвязная сеть (каждый с каждым)
    FullMesh,
    
    /// Кольцевая топология
    Ring,
    
    /// Звезда (все подключены к центральному узлу)
    Star { central_node: NodeId },
    
    /// Произвольная топология
    Custom { connections: HashMap<NodeId, HashSet<NodeId>> },
}

/// Виртуальная сеть для симуляции работы узлов TRIAD
pub struct VirtualNetwork {
    /// Узлы в сети
    nodes: HashMap<NodeId, Node>,
    
    /// Соединения между узлами
    connections: HashMap<NodeId, HashSet<NodeId>>,
    
    /// Конфигурация сети
    config: NetworkConfig,
    
    /// Метрики производительности сети
    metrics: NetworkMetrics,
    
    /// Квантовое поле для моделирования запутанности
    quantum_field: Option<QuantumField>,
    
    /// Уровень запутанности в сети (0.0 - 1.0)
    entanglement_level: f64,
}

/// Метрики производительности виртуальной сети
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Общее количество обработанных транзакций
    pub processed_transactions: u64,
    
    /// Среднее время достижения консенсуса (мс)
    pub avg_consensus_time_ms: f64,
    
    /// Среднее количество узлов, достигающих консенсуса
    pub avg_consensus_nodes: f64,
    
    /// Уровень запутанности в сети (0-1)
    pub entanglement_level: f64,
}

/// Результат обработки транзакции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Достигнут ли консенсус
    pub consensus: bool,
    
    /// Результат интерференции (от -1.0 до 1.0)
    pub interference_result: f64,
    
    /// Время обработки транзакции (мс)
    pub processing_time_ms: f64,
    
    /// Количество узлов, достигших консенсуса
    pub consensus_nodes: usize,
    
    /// Общее количество узлов в сети
    pub node_count: usize,
    
    /// Уровень запутанности после транзакции
    pub entanglement_level: f64,
}

impl VirtualNetwork {
    /// Создает новую виртуальную сеть с указанной конфигурацией
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            config: NetworkConfig::default(),
            metrics: NetworkMetrics::default(),
            quantum_field: None,
            entanglement_level: 0.0,
        }
    }
    
    /// Создает виртуальную сеть с заданным количеством узлов и кубитов
    pub fn with_nodes(node_count: usize, qubits_per_node: usize, config: NetworkConfig) -> Self {
        // Проверка на слишком большое количество кубитов
        if qubits_per_node > 30 {
            panic!("Слишком большое количество кубитов на узел: {}. Максимально допустимое значение: 30", qubits_per_node);
        }
    
        let mut network = Self::new();
        network.config = config;

        // Создаем узлы
        for i in 0..node_count {
            let node_id = format!("node_{}", i);
            network.add_node(&node_id, qubits_per_node);
        }

        // Устанавливаем соединения согласно топологии
        network.setup_topology();
        
        // Инициализируем квантовое поле, если оно включено
        if network.config.use_quantum_field {
            network.create_quantum_field(qubits_per_node);
        }

        network
    }
    
    /// Добавляет новый узел в сеть
    pub fn add_node(&mut self, id: &str, qubit_count: usize) {
        let node = Node::new(id, qubit_count);
        self.nodes.insert(id.to_string(), node);
        self.connections.insert(id.to_string(), HashSet::new());
        
        // Обновляем квантовое поле при добавлении узла
        if let Some(_field) = &mut self.quantum_field {
            // В реальной имплементации здесь бы обновлялось квантовое поле
            // Но это требует пересоздания поля, что сложно сделать в этой структуре
        }
    }
    
    /// Устанавливает соединения между узлами согласно выбранной топологии
    fn setup_topology(&mut self) {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        
        // Клонируем конфигурацию топологии для избежания проблем с заимствованием
        match &self.config.topology {
            NetworkTopology::FullMesh => {
                // Каждый узел соединен с каждым
                for i in 0..node_ids.len() {
                    for j in 0..node_ids.len() {
                        if i != j {
                            self.connect(&node_ids[i], &node_ids[j]);
                        }
                    }
                }
            },
            NetworkTopology::Ring => {
                // Узлы соединены в кольцо
                for i in 0..node_ids.len() {
                    let next = (i + 1) % node_ids.len();
                    self.connect(&node_ids[i], &node_ids[next]);
                }
            },
            NetworkTopology::Star { central_node } => {
                // Все узлы соединены с центральным
                let central = central_node.clone();
                for node_id in &node_ids {
                    if node_id != &central {
                        self.connect(node_id, &central);
                    }
                }
            },
            NetworkTopology::Custom { .. } => {
                // Устанавливаем соединения из настроек
                if let NetworkTopology::Custom { connections: ref connections_clone } = self.config.topology.clone() {
                    // Устанавливаем связи в соответствии с заданной конфигурацией
                    let connections_map: HashMap<String, HashSet<String>> = connections_clone.clone();
                    
                    for (node_str, neighbors_set) in connections_map {
                        let node = node_str.clone();
                        for neighbor_str in neighbors_set {
                            let neighbor = neighbor_str.clone();
                            self.connect(&node, &neighbor);
                        }
                    }
                }
            },
        }
    }
    
    /// Соединяет два узла
    fn connect(&mut self, node1: &str, node2: &str) {
        if let Some(connections) = self.connections.get_mut(node1) {
            connections.insert(node2.to_string());
        }
        if let Some(connections) = self.connections.get_mut(node2) {
            connections.insert(node1.to_string());
        }
    }
    
    /// Обрабатывает транзакцию через всю сеть
    pub fn process_transaction(&mut self, tx_data: &str) -> ConsensusResult {
        let start = Instant::now();
        
        // Если используем квантовое поле
        if let Some(field) = &mut self.quantum_field {
            // Выбираем случайный начальный узел
            let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
            if node_ids.is_empty() {
                return ConsensusResult {
                    consensus: false,
                    interference_result: 0.0,
                    processing_time_ms: 0.0,
                    consensus_nodes: 0,
                    node_count: 0,
                    entanglement_level: 0.0,
                };
            }
            
            let mut rng = rand::thread_rng();
            let start_node_idx = rng.gen_range(0..node_ids.len());
            let _start_node_id = node_ids[start_node_idx].parse::<usize>().unwrap_or(0);
            
            // Обрабатываем через квантовое поле
            let interference = field.process_transaction(start_node_idx, tx_data);
            
            // Обновляем уровень запутанности
            self.entanglement_level = interference.consensus_probability;
            
            // Обновляем метрики
            self.metrics.processed_transactions += 1;
            self.metrics.entanglement_level = self.entanglement_level;
            
            // Запускаем процесс распространения интерференции через сеть
            // (это модель квантовой волны, которая проходит через узлы)
            // Используем значение декогеренции из интерференции или установим значение по умолчанию
            let decoherence = 0.1; // Значение по умолчанию
            let all_nodes_reached = self.propagate_wave(node_ids[start_node_idx].as_str(), decoherence);
            
            // Время обработки
            let elapsed = start.elapsed();
            let processing_time_ms = elapsed.as_millis() as f64;
            
            // Возвращаем результат консенсуса
            ConsensusResult {
                consensus: all_nodes_reached >= 0.5,
                interference_result: interference.consensus_probability, // Используем доступное поле
                processing_time_ms,
                consensus_nodes: (all_nodes_reached * self.nodes.len() as f64) as usize,
                node_count: self.nodes.len(),
                entanglement_level: self.entanglement_level,
            }
        } else {
            // Если не используем квантовое поле, просто возвращаем тривиальный результат
            let processing_time_ms = start.elapsed().as_millis() as f64;
            ConsensusResult {
                consensus: true,
                interference_result: 0.0,
                processing_time_ms,
                consensus_nodes: self.nodes.len(),
                node_count: self.nodes.len(),
                entanglement_level: 0.0,
            }
        }
    }
    
    /// Моделирует распространение квантовой волны через сеть
    fn propagate_wave(&mut self, start_node_id: &str, decoherence_level: f64) -> f64 {
        // Моделирование распространения квантовой волны через сеть
        // Каждый узел получает волну с некоторой амплитудой, которая уменьшается при распространении
        // из-за декогеренции
        
        // Инициализируем волну на старте с амплитудой 1.0
        let mut wave_amplitudes: HashMap<String, f64> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        
        wave_amplitudes.insert(start_node_id.to_string(), 1.0);
        visited.insert(start_node_id.to_string());
        
        // Очередь узлов для обхода в ширину
        let mut queue: Vec<String> = vec![start_node_id.to_string()];
        
        // Пока очередь не пуста
        while !queue.is_empty() {
            let current_node_id = queue.remove(0);
            let current_amplitude = *wave_amplitudes.get(&current_node_id).unwrap_or(&0.0);
            
            // Декогеренция (затухание амплитуды)
            let next_amplitude = current_amplitude * (1.0 - decoherence_level);
            
            // Если амплитуда стала слишком маленькой, то дальше не распространяем
            if next_amplitude < 0.01 {
                continue;
            }
            
            // Получаем всех соседей текущего узла
            if let Some(neighbors) = self.connections.get(&current_node_id) {
                for neighbor_id in neighbors {
                    // Если соседа еще не посещали
                    if !visited.contains(neighbor_id) {
                        // Добавляем его в очередь
                        queue.push(neighbor_id.clone());
                        visited.insert(neighbor_id.clone());
                        
                        // Устанавливаем амплитуду волны для этого узла
                        wave_amplitudes.insert(neighbor_id.clone(), next_amplitude);
                    }
                }
            }
        }
        
        // Вычисляем процент узлов, достигших консенсуса
        let total_nodes = self.nodes.len() as f64;
        let consensus_nodes = wave_amplitudes.values().filter(|&&amp| amp > 0.5).count() as f64;
        
        consensus_nodes / total_nodes
    }
    
    /// Распространяет дельты (изменения состояний кубитов) по сети
    fn propagate_deltas(&mut self, start_node_id: &str, initial_deltas: &HashMap<NodeId, HashMap<usize, QubitDelta>>) {
        // Очередь узлов для распространения дельт
        let mut queue: Vec<String> = vec![start_node_id.to_string()];
        // Узлы, которые уже получили дельты
        let mut processed: HashSet<String> = HashSet::new();
        processed.insert(start_node_id.to_string());
        
        // Последние дельты для каждого узла
        let mut current_deltas = initial_deltas.clone();
        
        while !queue.is_empty() {
            let current_node_id = queue.remove(0);
            
            // Применяем дельты к текущему узлу
            if let Some(node) = self.nodes.get_mut(&current_node_id) {
                if let Some(deltas) = current_deltas.get(&current_node_id) {
                    // В реальной имплементации здесь бы применялись дельты к узлу
                    // Используем существующий метод node.apply_deltas вместо несуществующего apply_qubit_delta
                    node.apply_deltas(deltas);
                }
            }
            
            // Получаем соседей текущего узла
            if let Some(neighbors) = self.connections.get(&current_node_id) {
                for neighbor_id in neighbors {
                    // Если сосед еще не получил дельты
                    if !processed.contains(neighbor_id) {
                        queue.push(neighbor_id.clone());
                        processed.insert(neighbor_id.clone());
                        
                        // Передаем дельты соседу с учетом сетевой задержки и возможных искажений
                        // В реальной системе здесь было бы больше логики, но для симуляции просто копируем
                        if let Some(deltas) = current_deltas.get(&current_node_id) {
                            current_deltas.insert(neighbor_id.clone(), deltas.clone());
                        }
                    }
                }
            }
        }
    }
    
    /// Проверяет, достигнут ли консенсус по значениям квантовой волны
    fn check_consensus(&self, wave_values: &HashMap<NodeId, f64>) -> (bool, f64) {
        // Проверяем, все ли узлы имеют значение волны выше порогового
        let threshold = 0.5;
        let mut consensus_count = 0;
        
        for (_node_id, &value) in wave_values {
            if value >= threshold {
                consensus_count += 1;
            }
        }
        
        let consensus_ratio = consensus_count as f64 / wave_values.len() as f64;
        let consensus = consensus_ratio >= 0.5;
        
        (consensus, consensus_ratio)
    }
    
    /// Возвращает метрики сети
    pub fn metrics(&self) -> &NetworkMetrics {
        &self.metrics
    }
    
    /// Измеряет уровень запутанности в сети
    pub fn measure_entanglement_level(&self) -> f64 {
        if let Some(field) = &self.quantum_field {
            // Измеряем запутанность для всех пар узлов
            let mut total_entanglement = 0.0;
            let mut pair_count = 0;
            
            // Получаем все идентификаторы узлов
            let node_ids: Vec<usize> = (0..self.nodes.len()).collect();
            
            // Для каждой пары узлов
            for i in 0..node_ids.len() {
                for j in (i+1)..node_ids.len() {
                    // Вычисляем запутанность
                    let entanglement = field.calculate_entanglement_between_nodes(node_ids[i], node_ids[j]);
                    total_entanglement += entanglement.iter().map(|&(_, _, level)| level).sum::<f64>();
                    pair_count += entanglement.len();
                }
            }
            
            if pair_count > 0 {
                return total_entanglement / pair_count as f64;
            }
        }
        
        0.0
    }
    
    /// Активирует или деактивирует расширенный квантовый симулятор
    pub fn activate_triadvantum(&mut self, activate: bool) {
        if let Some(field) = &mut self.quantum_field {
            // Используем правильный метод set_use_triadvantum вместо несуществующего use_triadvantum
            field.set_use_triadvantum(activate);
        }
    }
    
    /// Создает демо-схему для тестирования
    pub fn create_demo_circuit(&mut self, demo_type: &str) -> bool {
        if let Some(field) = &mut self.quantum_field {
            // Используем существующий метод create_demo_circuit вместо несуществующих методов
            field.create_demo_circuit(demo_type)
        } else {
            false
        }
    }
    
    /// Восстанавливает квантовое состояние
    pub fn recover_quantum_state(&mut self) -> bool {
        if let Some(field) = &mut self.quantum_field {
            return field.recover_state();
        }
        false
    }
    
    /// Находит оптимальный маршрут между узлами
    fn find_optimal_route(&self, source: &str, target: &str) -> Option<Vec<String>> {
        // Алгоритм поиска кратчайшего пути (BFS)
        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        let mut prev: HashMap<String, String> = HashMap::new();
        
        queue.push(source.to_string());
        visited.insert(source.to_string());
        
        while !queue.is_empty() {
            let current = queue.remove(0);
            
            // Если нашли целевой узел
            if current == target {
                // Восстанавливаем путь
                let mut path = Vec::new();
                let mut curr = current;
                path.push(curr.clone());
                
                while let Some(previous) = prev.get(&curr) {
                    path.push(previous.clone());
                    curr = previous.clone();
                }
                
                path.reverse();
                return Some(path);
            }
            
            // Проверяем соседей
            if let Some(neighbors) = self.connections.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        prev.insert(neighbor.clone(), current.clone());
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Создает и инициализирует квантовое поле
    pub fn create_quantum_field(&mut self, qubits_per_node: usize) {
        let node_count = self.nodes.len();
        if node_count > 0 {
            self.quantum_field = Some(QuantumField::new(node_count, qubits_per_node));
        }
    }
} 