use crate::quantum::triadvantum::{
    QuantumState,
    gates::QuantumGate,
    operators::QuantumOperator,
    circuit::QuantumCircuit,
    recovery::{RecoveryProtocol, RecoveryEvent},
    delta::QuantumDelta,
    interference::InterferencePattern
};
use num_complex::Complex64;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use rand;

/// Результат симуляции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Измеренные значения кубитов
    pub measurements: HashMap<usize, usize>,
    /// Вероятности до коллапса
    pub probabilities: HashMap<usize, f64>,
    /// Финальное состояние
    pub final_state: QuantumState,
    /// Статистика симуляции
    pub stats: SimulationStats
}

/// Статистика симуляции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    /// Количество примененных гейтов
    pub gates_applied: usize,
    /// Количество измерений
    pub measurements: usize,
    /// Количество ошибок
    pub errors: usize,
    /// Время выполнения
    pub execution_time: f64,
    /// История состояний
    pub state_history: Vec<(String, u64)>
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self {
            gates_applied: 0,
            measurements: 0,
            errors: 0,
            execution_time: 0.0,
            state_history: Vec::new()
        }
    }
}

/// Симулятор квантовых вычислений
pub struct QrustSimulator {
    /// Текущее квантовое состояние
    state: QuantumState,
    /// Протокол восстановления
    recovery: RecoveryProtocol,
    /// Кэш состояний
    state_cache: HashMap<String, QuantumState>,
    /// Статистика симуляции
    stats: SimulationStats,
    /// Флаг отладки
    debug_mode: bool,
    /// Квантовая схема
    circuit: QuantumCircuit,
    /// ID ноды (для создания дельт)
    node_id: String
}

impl QrustSimulator {
    /// Создает новый симулятор
    pub fn new(node_id: String, qubit_count: usize, debug_mode: bool) -> Result<Self, String> {
        Ok(Self {
            state: QuantumState::new(qubit_count),
            recovery: RecoveryProtocol::new(),
            state_cache: HashMap::new(),
            stats: SimulationStats::default(),
            debug_mode,
            circuit: QuantumCircuit::new(qubit_count),
            node_id
        })
    }

    /// Применяет квантовый гейт
    pub fn apply_gate(&mut self, gate: &QuantumGate) -> Result<(), String> {
        match gate {
            QuantumGate::Single(target, matrix) => {
                if *target >= self.state.qubit_count {
                    return Err(format!("Индекс кубита {} вне диапазона 0-{}", target, self.state.qubit_count - 1));
                }
                self.state.apply_single_qubit_gate(*target, matrix.clone());
                self.stats.gates_applied += 1;
                Ok(())
            },
            QuantumGate::Two(control, target, matrix) => {
                if *control >= self.state.qubit_count || *target >= self.state.qubit_count {
                    return Err(format!("Индексы кубитов {}, {} вне диапазона 0-{}", 
                                     control, target, self.state.qubit_count - 1));
                }
                self.state.apply_two_qubit_gate(*control, *target, matrix.clone());
                self.stats.gates_applied += 1;
                Ok(())
            }
        }
    }

    /// Применяет квантовый оператор
    pub fn apply_operator(&mut self, operator: &QuantumOperator) -> Result<(), String> {
        operator.apply(&mut self.state)
    }

    /// Выполняет квантовую схему
    pub fn run_circuit(&mut self, circuit: &QuantumCircuit) -> Result<SimulationResult, String> {
        let start_time = std::time::Instant::now();
        let mut measurements = HashMap::new();
        let mut probabilities = HashMap::new();
        
        for (_gate_idx, gate) in circuit.gates.iter().enumerate() {
            match gate {
                QuantumGate::Single(target, _matrix) => {
                    // Проверяем, является ли гейт измерением
                    if gate.is_measurement() {
                        let prob = self.state.get_probability_one(*target);
                        let rand_val: f64 = rand::random();
                        let result = if rand_val < prob { 1 } else { 0 };
                        
                        measurements.insert(*target, result);
                        probabilities.insert(*target, prob);
                        
                        // Коллапсируем состояние в соответствии с результатом
                        self.collapse_qubit(*target, result == 1)?;
                    } else {
                        gate.apply(&mut self.state)?;
                    }
                },
                QuantumGate::Two(_, _, _) => {
                    gate.apply(&mut self.state)?;
                }
            }
        }
        
        let execution_time = start_time.elapsed();
        // Сохраняем количество измерений перед перемещением
        let measurements_count = measurements.len();
        
        let result = SimulationResult {
            measurements,
            probabilities,
            final_state: self.state.clone(),
            stats: SimulationStats::default(),
        };
        
        self.stats.execution_time += execution_time.as_secs_f64();
        self.stats.measurements += measurements_count;
        
        Ok(result)
    }

    /// Измеряет кубит
    pub fn measure_qubit(&mut self, qubit: usize) -> Result<usize, String> {
        if qubit >= self.state.qubit_count {
            return Err(format!("Индекс кубита {} вне диапазона 0-{}", qubit, self.state.qubit_count - 1));
        }
        
        let result = self.state.measure_qubit(qubit);
        self.stats.measurements += 1;
        Ok(result)
    }

    /// Создает дельту с предыдущим состоянием
    pub fn create_delta(&self, old_state: &QuantumState) -> Result<QuantumDelta, String> {
        if old_state.qubit_count != self.state.qubit_count {
            return Err("Количество кубитов в состояниях не совпадает".to_string());
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let delta = QuantumDelta::new(
            format!("delta_{}", timestamp),
            old_state.clone(),
            self.state.clone(),
            self.node_id.clone(),
            timestamp
        );
        
        Ok(delta)
    }

    /// Применяет дельту к текущему состоянию
    pub fn apply_delta(&mut self, delta: &QuantumDelta) -> Result<(), String> {
        if delta.old_state.qubit_count != self.state.qubit_count {
            return Err("Количество кубитов в дельте и текущем состоянии не совпадает".to_string());
        }
        
        // Применяем изменения из дельты
        for (idx, amplitude) in &delta.changed_amplitudes {
            self.state.set_amplitude(*idx, *amplitude);
        }
        
        // Нормализуем состояние
        self.state.normalize();
        
        Ok(())
    }

    /// Добавляет чекпоинт текущего состояния
    pub fn add_checkpoint(&mut self, state_id: String) -> Result<(), String> {
        self.recovery.add_state_checkpoint(state_id, self.state.clone());
        Ok(())
    }

    /// Восстанавливает состояние
    pub fn recover_state(&mut self) -> Result<(), String> {
        let last_checkpoint = self.recovery.get_last_checkpoint()?;
        self.state = last_checkpoint;
        self.stats.errors += 1;
        
        Ok(())
    }

    /// Получает текущее состояние
    pub fn get_state(&self) -> &QuantumState {
        &self.state
    }

    /// Получает статистику симуляции
    pub fn get_stats(&self) -> &SimulationStats {
        &self.stats
    }

    /// Очищает кэш состояний
    pub fn clear_cache(&mut self) {
        self.state_cache.clear();
    }

    /// Устанавливает режим отладки
    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    /// Устанавливает состояние симулятора
    pub fn set_state(&mut self, state: QuantumState) -> Result<(), String> {
        if state.qubit_count != self.state.qubit_count {
            return Err(format!("Несовместимое количество кубитов: {} vs {}", 
                             state.qubit_count, self.state.qubit_count));
        }
        self.state = state;
        Ok(())
    }

    /// Коллапсирует кубит в указанное состояние
    fn collapse_qubit(&mut self, qubit_idx: usize, to_one: bool) -> Result<(), String> {
        if qubit_idx >= self.state.qubit_count {
            return Err(format!("Индекс кубита {} вне диапазона 0-{}", 
                             qubit_idx, self.state.qubit_count - 1));
        }
        
        // Устанавливаем все амплитуды с противоположным состоянием кубита в 0
        let n = 1 << self.state.qubit_count;
        let mask = 1 << qubit_idx;
        
        let mut norm_squared = 0.0;
        
        for i in 0..n {
            let is_one = (i & mask) != 0;
            if is_one != to_one {
                self.state.set_amplitude(i, Complex64::new(0.0, 0.0));
            } else {
                norm_squared += self.state.get_amplitude(i).norm_sqr();
            }
        }
        
        // Нормализуем оставшиеся амплитуды
        let norm = norm_squared.sqrt();
        if norm > 0.0 {
            for i in 0..n {
                let amp = self.state.get_amplitude(i);
                if amp.norm() > 0.0 {
                    self.state.set_amplitude(i, amp / norm);
                }
            }
        }
        
        Ok(())
    }
} 