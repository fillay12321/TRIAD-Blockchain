use crate::quantum::triadvantum::{
    QuantumState,
    QubitState
};
use crate::quantum::triadvantum::delta::QuantumDelta;
use num_complex::Complex64;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

/// Константы для интерференции
const INTERFERENCE_THRESHOLD: f64 = 1e-6;
const MAX_INTERFERENCE_POINTS: usize = 1024;

/// Интерференционный шаблон
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterferencePattern {
    /// Идентификатор шаблона
    pub id: String,
    /// Состояние до интерференции
    pub initial_state: QuantumState,
    /// Состояние после интерференции
    pub final_state: QuantumState,
    /// Матрица интерференции
    pub interference_matrix: HashMap<usize, Complex64>,
    /// Метаданные
    pub metadata: HashMap<String, String>,
}

impl InterferencePattern {
    /// Создает новый шаблон интерференции
    pub fn new(
        id: String, 
        initial_state: QuantumState, 
        final_state: QuantumState
    ) -> Self {
        // Вычисляем матрицу интерференции
        let qubit_count = initial_state.qubit_count();
        let dim = 1 << qubit_count;
        let mut matrix = HashMap::new();
        
        // Заполняем матрицу интерференции
        // Для простоты используем приближение
        let initial_amplitudes = initial_state.get_amplitudes();
        let final_amplitudes = final_state.get_amplitudes();
        
        for i in 0..dim {
            for j in 0..dim {
                if initial_amplitudes[j].norm() > 1e-10 {
                    matrix.insert(i * dim + j, final_amplitudes[i] / initial_amplitudes[j]);
                }
            }
        }
        
        Self {
            id,
            initial_state,
            final_state,
            interference_matrix: matrix,
            metadata: HashMap::new(),
        }
    }
    
    /// Применяет шаблон интерференции к состоянию
    pub fn apply(&self, state: &mut QuantumState) -> Result<(), String> {
        if state.qubit_count() != self.initial_state.qubit_count() {
            return Err("Количество кубитов не совпадает".to_string());
        }
        
        let dim = 1 << state.qubit_count();
        let mut new_amplitudes = vec![Complex64::new(0.0, 0.0); dim];
        let amplitudes = state.get_amplitudes();
        
        // Применяем матрицу интерференции
        for i in 0..dim {
            for j in 0..dim {
                new_amplitudes[i] += self.interference_matrix.get(&(i * dim + j)).cloned().unwrap_or(Complex64::new(0.0, 0.0)) * amplitudes[j];
            }
        }
        
        // Нормализуем результат
        let norm: f64 = new_amplitudes.iter().map(|a| a.norm_sqr()).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for a in &mut new_amplitudes {
                *a /= norm;
            }
        }
        
        // Заменяем амплитуды
        state.set_amplitudes(new_amplitudes);
        
        Ok(())
    }
    
    /// Вычисляет интерференцию между двумя состояниями
    pub fn calculate_interference(state1: &QuantumState, state2: &QuantumState) -> Result<Self, String> {
        if state1.qubit_count() != state2.qubit_count() {
            return Err("Количество кубитов не совпадает".to_string());
        }
        
        let id = format!("interference_{}", state1.qubit_count());
        Ok(Self::new(id, state1.clone(), state2.clone()))
    }
    
    /// Вычисляет фазу между двумя состояниями
    pub fn calculate_phase(&self) -> f64 {
        let mut phase_sum = 0.0;
        let mut count = 0;
        
        let initial_amplitudes = self.initial_state.get_amplitudes();
        let final_amplitudes = self.final_state.get_amplitudes();
        
        for (a1, a2) in initial_amplitudes.iter().zip(final_amplitudes.iter()) {
            if a1.norm() > 1e-10 && a2.norm() > 1e-10 {
                let phase = (a2 / a1).arg();
                phase_sum += phase;
                count += 1;
            }
        }
        
        if count > 0 {
            phase_sum / count as f64
        } else {
            0.0
        }
    }
    
    /// Добавляет метаданные
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Получает метаданные
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Вычисляет силу интерференции между двумя состояниями
    pub fn strength(&self) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;
        
        let initial_amplitudes = self.initial_state.get_amplitudes();
        let final_amplitudes = self.final_state.get_amplitudes();
        
        for (a1, a2) in initial_amplitudes.iter().zip(final_amplitudes.iter()) {
            if a1.norm() > 1e-10 && a2.norm() > 1e-10 {
                let rel_amplitude = (a2.norm() / a1.norm()).abs();
                sum += rel_amplitude;
                count += 1;
            }
        }
        
        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
    
    /// Возвращает фазу интерференции
    pub fn phase(&self) -> f64 {
        self.calculate_phase()
    }
    
    /// Возвращает количество кубитов в паттерне
    pub fn qubit_count(&self) -> usize {
        self.initial_state.qubit_count()
    }
}

/// Калькулятор консенсуса для распределенных квантовых состояний
#[derive(Clone, Debug)]
pub struct ConsensusCalculator {
    /// Собранные паттерны от разных узлов (идентификатор узла -> паттерн)
    node_patterns: HashMap<String, InterferencePattern>,
    /// Весовые коэффициенты узлов для расчета консенсуса
    node_weights: HashMap<String, f64>,
    /// Последний рассчитанный консенсусный паттерн
    consensus_pattern: Option<InterferencePattern>,
    /// Временная метка последнего расчета
    last_calculation: Option<SystemTime>,
    /// Версия консенсуса
    consensus_version: u64,
}

impl ConsensusCalculator {
    /// Создает новый калькулятор консенсуса
    pub fn new() -> Self {
        Self {
            node_patterns: HashMap::new(),
            node_weights: HashMap::new(),
            consensus_pattern: None,
            last_calculation: None,
            consensus_version: 0,
        }
    }
    
    /// Добавляет паттерн от узла
    pub fn add_node_pattern(&mut self, node_id: &str, pattern: InterferencePattern, weight: Option<f64>) {
        self.node_patterns.insert(node_id.to_string(), pattern);
        self.node_weights.insert(node_id.to_string(), weight.unwrap_or(1.0));
        self.consensus_pattern = None; // Сбрасываем кэш
    }
    
    /// Вычисляет консенсусный паттерн
    pub fn calculate_consensus(&mut self) -> Option<InterferencePattern> {
        if self.node_patterns.is_empty() {
            return None;
        }
        
        // Если консенсус уже рассчитан, возвращаем его
        if let Some(ref pattern) = self.consensus_pattern {
            return Some(pattern.clone());
        }
        
        // Определяем размерность (количество кубитов) из первого паттерна
        let qubit_count = self.node_patterns.values().next().unwrap().initial_state.qubit_count();
        
        // Проверяем, что все паттерны имеют одинаковую размерность
        for pattern in self.node_patterns.values() {
            if pattern.initial_state.qubit_count() != qubit_count {
                return None;
            }
        }
        
        // Создаем комбинированный паттерн
        let mut combined_pattern = HashMap::new();
        let mut total_weight = 0.0;
        
        // Комбинируем амплитуды от всех узлов с учетом весов
        for (node_id, pattern) in &self.node_patterns {
            let weight = self.node_weights.get(node_id).cloned().unwrap_or(1.0);
            total_weight += weight;
            
            for (&idx, &amp) in &pattern.interference_matrix {
                *combined_pattern.entry(idx).or_insert(Complex64::new(0.0, 0.0)) += amp * weight;
            }
        }
        
        // Нормализуем с учетом суммы весов
        if total_weight > 0.0 {
            for amp in combined_pattern.values_mut() {
                *amp /= total_weight;
            }
        }
        
        // Создаем консенсусный паттерн
        let mut max_amplitude: f64 = 0.0;
        let mut constructive_probability = 0.0;
        let mut destructive_probability = 0.0;
        
        for &amp in combined_pattern.values() {
            let amp_sqr = amp.norm_sqr();
            max_amplitude = max_amplitude.max(amp_sqr.sqrt());
            
            if amp.re > 0.0 {
                constructive_probability += amp_sqr;
            } else {
                destructive_probability += amp_sqr;
            }
        }
        
        // Нормализуем вероятности
        let total_prob = constructive_probability + destructive_probability;
        if total_prob > 0.0 {
            constructive_probability /= total_prob;
            destructive_probability /= total_prob;
        }
        
        // Создаем и сохраняем результат
        self.consensus_version += 1;
        let consensus = InterferencePattern {
            id: format!("consensus_{}", qubit_count),
            initial_state: QuantumState::new(qubit_count),
            final_state: QuantumState::new(qubit_count),
            interference_matrix: combined_pattern,
            metadata: HashMap::new(),
        };
        
        self.consensus_pattern = Some(consensus.clone());
        self.last_calculation = Some(SystemTime::now());
        
        Some(consensus)
    }
    
    /// Создает дельту на основе консенсусного паттерна
    pub fn create_consensus_delta(&mut self, node_id: &str) -> Option<QuantumDelta> {
        // Вычисляем консенсус, если не рассчитан
        if self.consensus_pattern.is_none() {
            self.calculate_consensus();
        }
        
        if let Some(ref consensus) = self.consensus_pattern {
            // Создаем состояние из консенсусного паттерна
            let mut consensus_state = QuantumState::new(consensus.initial_state.qubit_count());
            consensus_state.set_amplitudes_from_hashmap(consensus.interference_matrix.clone());
            
            // Получаем текущее время для timestamp
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            // Создаем дельту для полной замены состояния
            Some(QuantumDelta::new_amplitude_delta(
                format!("consensus_{}", node_id),
                consensus_state,
                node_id.to_string(),
                timestamp,
                consensus.interference_matrix.clone()
            ))
        } else {
            None
        }
    }
    
    /// Вычисляет уровень согласия между узлами (0.0 - нет согласия, 1.0 - полное согласие)
    pub fn calculate_agreement_level(&self) -> f64 {
        if self.node_patterns.len() <= 1 {
            return 1.0; // Если только один узел, согласие полное
        }
        
        let mut total_agreement = 0.0;
        let mut pair_count = 0;
        
        // Вычисляем попарные перекрытия между всеми паттернами
        let nodes: Vec<&String> = self.node_patterns.keys().collect();
        for i in 0..nodes.len() {
            for j in (i+1)..nodes.len() {
                let pattern1 = &self.node_patterns[nodes[i]];
                let pattern2 = &self.node_patterns[nodes[j]];
                
                let overlap = self.calculate_correlation(pattern1, pattern2);
                total_agreement += overlap;
                pair_count += 1;
            }
        }
        
        if pair_count > 0 {
            total_agreement / pair_count as f64
        } else {
            0.0
        }
    }
    
    /// Обновляет вес узла на основе согласованности его паттерна с консенсусом
    pub fn update_node_weight(&mut self, node_id: &str) {
        if let Some(consensus) = &self.consensus_pattern {
            if let Some(node_pattern) = self.node_patterns.get(node_id) {
                let agreement = self.calculate_correlation(node_pattern, consensus);
                
                // Обновляем вес узла на основе согласованности
                let current_weight = self.node_weights.get(node_id).cloned().unwrap_or(1.0);
                let new_weight = current_weight * 0.8 + agreement * 0.2;
                
                self.node_weights.insert(node_id.to_string(), new_weight);
            }
        }
    }
    
    /// Удаляет паттерн узла
    pub fn remove_node(&mut self, node_id: &str) {
        self.node_patterns.remove(node_id);
        self.node_weights.remove(node_id);
        self.consensus_pattern = None; // Сбрасываем кэш
    }
    
    /// Очищает все данные
    pub fn clear(&mut self) {
        self.node_patterns.clear();
        self.node_weights.clear();
        self.consensus_pattern = None;
        self.last_calculation = None;
        self.consensus_version = 0;
    }
    
    /// Получает версию консенсуса
    pub fn get_consensus_version(&self) -> u64 {
        self.consensus_version
    }
    
    /// Получает количество узлов, участвующих в консенсусе
    pub fn get_node_count(&self) -> usize {
        self.node_patterns.len()
    }
    
    /// Вычисляет корреляцию между двумя паттернами
    fn calculate_correlation(&self, pattern1: &InterferencePattern, pattern2: &InterferencePattern) -> f64 {
        let mut correlation = 0.0;
        let mut count = 0;
        
        // Проходим по всем ключам в обоих паттернах
        let keys1: HashSet<_> = pattern1.interference_matrix.keys().collect();
        let keys2: HashSet<_> = pattern2.interference_matrix.keys().collect();
        
        // Находим общие ключи
        for &idx in keys1.intersection(&keys2) {
            if let (Some(&amp1), Some(&amp2)) = (pattern1.interference_matrix.get(idx), pattern2.interference_matrix.get(idx)) {
                correlation += (amp1 * amp2.conj()).norm();
                count += 1;
            }
        }
        
        if count > 0 {
            correlation / count as f64
        } else {
            0.0
        }
    }
}

/// Детектор интерференционных аномалий
pub struct InterferenceDetector {
    /// История интерференционных паттернов
    pattern_history: Vec<(InterferencePattern, u64)>,
    /// Пороговые значения для детекции аномалий
    threshold: f64,
    /// Максимальный размер истории
    max_history_size: usize,
}

impl InterferenceDetector {
    /// Создает новый детектор интерференционных аномалий
    pub fn new(threshold: f64, max_history_size: usize) -> Self {
        Self {
            pattern_history: Vec::new(),
            threshold,
            max_history_size,
        }
    }
    
    /// Добавляет паттерн в историю
    pub fn add_pattern(&mut self, pattern: InterferencePattern, timestamp: u64) {
        self.pattern_history.push((pattern, timestamp));
        
        // Если история слишком большая, удаляем старые записи
        if self.pattern_history.len() > self.max_history_size {
            self.pattern_history.remove(0);
        }
    }
    
    /// Детектирует аномалии в текущем паттерне
    pub fn detect_anomalies(&self, current_pattern: &InterferencePattern) -> Vec<InterferenceAnomaly> {
        let mut anomalies = Vec::new();
        
        // Если история пуста, нет с чем сравнивать
        if self.pattern_history.is_empty() {
            return anomalies;
        }
        
        // Получаем предыдущий паттерн
        let (prev_pattern, prev_timestamp) = &self.pattern_history[self.pattern_history.len() - 1];
        
        // Вычисляем перекрытие между текущим и предыдущим паттернами
        let overlap = self.calculate_correlation(current_pattern, prev_pattern);
        
        // Если перекрытие меньше порога, детектируем аномалию
        if overlap < self.threshold {
            anomalies.push(InterferenceAnomaly {
                anomaly_type: AnomalyType::PatternChange,
                severity: (self.threshold - overlap) / self.threshold,
                description: format!("Резкое изменение интерференционного паттерна, перекрытие: {:.4}", overlap),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });
        }
        
        // Детектируем изменения в уровне интерференции
        let current_strength = current_pattern.strength();
        let prev_strength = prev_pattern.strength();
        let strength_diff = (current_strength - prev_strength).abs();
                                
        if strength_diff > self.threshold / 2.0 {
            anomalies.push(InterferenceAnomaly {
                anomaly_type: AnomalyType::InterferenceTypeChange,
                severity: strength_diff,
                description: format!("Значительное изменение силы интерференции: {:.4}", strength_diff),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });
        }
        
        anomalies
    }
    
    /// Очищает историю паттернов
    pub fn clear_history(&mut self) {
        self.pattern_history.clear();
    }
    
    /// Устанавливает новый порог детекции
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }
    
    /// Вычисляет корреляцию между двумя паттернами
    fn calculate_correlation(&self, pattern1: &InterferencePattern, pattern2: &InterferencePattern) -> f64 {
        let mut correlation = 0.0;
        let mut count = 0;
        
        // Проходим по всем ключам в обоих паттернах
        let keys1: HashSet<_> = pattern1.interference_matrix.keys().collect();
        let keys2: HashSet<_> = pattern2.interference_matrix.keys().collect();
        
        // Находим общие ключи
        for &idx in keys1.intersection(&keys2) {
            if let (Some(&amp1), Some(&amp2)) = (pattern1.interference_matrix.get(idx), pattern2.interference_matrix.get(idx)) {
                correlation += (amp1 * amp2.conj()).norm();
                count += 1;
            }
        }
        
        if count > 0 {
            correlation / count as f64
        } else {
            0.0
        }
    }
}

/// Типы интерференционных аномалий
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Резкое изменение паттерна
    PatternChange,
    /// Изменение типа интерференции (конструктивная/деструктивная)
    InterferenceTypeChange,
    /// Потеря запутанности
    EntanglementLoss,
    /// Неконсистентность между узлами
    NodeInconsistency,
}

/// Интерференционная аномалия
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterferenceAnomaly {
    /// Тип аномалии
    pub anomaly_type: AnomalyType,
    /// Серьезность аномалии (0.0 - 1.0)
    pub severity: f64,
    /// Описание аномалии
    pub description: String,
    /// Временная метка
    pub timestamp: u64,
}

/// Вычисляет интерференционный паттерн из квантового состояния
pub fn calculate_from_state(state: &QuantumState, significant_qubits: &[usize]) -> InterferencePattern {
    InterferencePattern::new(
        format!("interference_{}", state.qubit_count()),
        state.clone(),
        state.clone()
    )
}

/// Вычисляет интерференцию между двумя квантовыми состояниями
pub fn calculate_interference(state1: &QuantumState, state2: &QuantumState, significant_qubits: &[usize]) -> InterferencePattern {
    if state1.qubit_count() != state2.qubit_count() {
        // Если разное количество кубитов, возвращаем пустой паттерн
        return InterferencePattern::new(
            format!("interference_{}", significant_qubits.len()),
            QuantumState::new(significant_qubits.len()),
            QuantumState::new(significant_qubits.len())
        );
    }
    
    let mut pattern = HashMap::new();
    let qubit_count = significant_qubits.len();
    let subspace_size = 1 << qubit_count;
    
    let mut max_amplitude: f64 = 0.0;
    let mut constructive_count = 0;
    let mut destructive_count = 0;
    let mut total_phase: f64 = 0.0;
    let mut total_strength: f64 = 0.0;
    
    for i in 0..subspace_size {
        // Находим соответствующий индекс в полном пространстве состояний
        let mut state_idx = 0;
        for (j, &qubit) in significant_qubits.iter().enumerate() {
            if (i >> j) & 1 == 1 {
                state_idx |= 1 << qubit;
            }
        }
        
        // Получаем амплитуды для обоих состояний
        let amp1 = if state_idx < state1.amplitudes_len() {
            state1.get_amplitude(state_idx)
        } else {
            Complex64::new(0.0, 0.0)
        };
        
        let amp2 = if state_idx < state2.amplitudes_len() {
            state2.get_amplitude(state_idx)
        } else {
            Complex64::new(0.0, 0.0)
        };
        
        // Вычисляем интерференционную амплитуду
        let interference_amp = amp1 + amp2;
        
        // Сохраняем амплитуду, если она значительна
        if interference_amp.norm() > 1e-10 {
            pattern.insert(i, interference_amp);
            
            // Обновляем максимальную амплитуду
            max_amplitude = max_amplitude.max(interference_amp.norm());
            
            // Анализируем конструктивную/деструктивную интерференцию
            if interference_amp.re > 0.0 {
                constructive_count += 1;
            } else {
                destructive_count += 1;
            }
            
            // Вычисляем фазу и силу
            total_phase += interference_amp.arg();
            total_strength += interference_amp.norm();
        }
    }
    
    // Вычисляем вероятности
    let total_states = constructive_count + destructive_count;
    let constructive_probability = if total_states > 0 {
        constructive_count as f64 / total_states as f64
    } else {
        0.5 // По умолчанию при отсутствии информации
    };
    
    // Нормализуем фазу и силу
    let phase = if !pattern.is_empty() {
        total_phase / pattern.len() as f64
    } else {
        0.0
    };
    
    let strength = if !pattern.is_empty() {
        total_strength / pattern.len() as f64
    } else {
        0.0
    };
    
    InterferencePattern {
        id: format!("interference_{}", qubit_count),
        initial_state: QuantumState::new(qubit_count),
        final_state: QuantumState::new(qubit_count),
        interference_matrix: pattern,
        metadata: HashMap::new(),
    }
} 