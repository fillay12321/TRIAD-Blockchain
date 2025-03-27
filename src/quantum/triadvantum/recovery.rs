use crate::quantum::triadvantum::{
    state::QuantumState, 
    delta::{QuantumDelta, DeltaType}
};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Максимальное количество состояний в истории для восстановления
const MAX_RECOVERY_HISTORY: usize = 10;
/// Максимальное время хранения состояний в истории (в секундах)
const MAX_HISTORY_AGE_SECS: u64 = 60 * 10; // 10 минут

/// Событие восстановления состояния
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryEvent {
    /// Успешное восстановление
    Success,
    /// Ошибка восстановления
    Failed(String),
    /// Частичное восстановление
    Partial { reason: String, success_rate: f64 },
    /// Применена дельта
    DeltaApplied { success: bool, delta_id: String }
}

/// Тип события восстановления
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryEventType {
    /// Успешное восстановление
    Success,
    /// Ошибка восстановления
    Failed(String),
    /// Частичное восстановление
    Partial { reason: String, success_rate: f64 },
    /// Применена дельта
    DeltaApplied { success: bool, delta_id: String },
    /// Ошибка
    Error
}

/// Стратегии восстановления
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Полное восстановление состояния
    Full,
    /// Восстановление по дельтам
    Delta,
    /// Восстановление по чекпоинтам
    Checkpoint,
    /// Адаптивное восстановление
    Adaptive
}

/// Контрольная точка для восстановления
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryCheckpoint {
    /// Идентификатор контрольной точки
    pub id: u64,
    /// Полное квантовое состояние (если есть)
    pub full_state: Option<QuantumState>,
    /// Дельты изменений от предыдущей контрольной точки
    pub deltas: Vec<QuantumDelta>,
    /// Отметка времени
    pub timestamp: u64,
    /// Тип контрольной точки
    pub checkpoint_type: CheckpointType,
    /// Важность контрольной точки (не будет удалена при ротации)
    pub is_important: bool,
}

/// Тип контрольной точки
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckpointType {
    /// Полное квантовое состояние
    Full,
    /// Только дельта изменений
    Delta,
    /// Партитура (последовательность операций)
    Score,
}

/// Состояние для восстановления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryState {
    /// Идентификатор состояния
    pub id: String,
    /// Квантовое состояние
    pub state: QuantumState,
    /// Временная метка
    pub timestamp: u64,
    /// Тип состояния
    pub state_type: RecoveryStateType,
    /// Метаданные
    pub metadata: HashMap<String, String>
}

/// Типы состояний восстановления
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStateType {
    /// Полное состояние
    Full,
    /// Дельта-состояние
    Delta(QuantumDelta),
    /// Чекпоинт
    Checkpoint
}

/// Статистика восстановления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    /// Количество успешных восстановлений
    pub success_count: usize,
    /// Количество неудачных восстановлений
    pub failure_count: usize,
    /// Количество частичных восстановлений
    pub partial_count: usize,
    /// Среднее время восстановления
    pub avg_recovery_time: f64,
    /// История событий
    pub event_history: Vec<(RecoveryEvent, u64)>
}

impl RecoveryStats {
    /// Создает новую статистику
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            partial_count: 0,
            avg_recovery_time: 0.0,
            event_history: Vec::new()
        }
    }

    /// Записывает событие восстановления
    pub fn record_event(&mut self, event: RecoveryEvent) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        match event {
            RecoveryEvent::Success => self.success_count += 1,
            RecoveryEvent::Failed(_) => self.failure_count += 1,
            RecoveryEvent::Partial { .. } => self.partial_count += 1,
            RecoveryEvent::DeltaApplied { success, .. } => {
                if success {
                    self.success_count += 1;
                } else {
                    self.failure_count += 1;
                }
            }
        }
        
        self.event_history.push((event, timestamp));
    }

    /// Вычисляет среднее время восстановления
    pub fn calculate_avg_recovery_time(&mut self) {
        if self.event_history.is_empty() {
            return;
        }

        let mut total_time = 0.0;
        let mut count = 0;

        for i in 1..self.event_history.len() {
            let (_, prev_time) = self.event_history[i - 1];
            let (_, curr_time) = self.event_history[i];
            total_time += (curr_time - prev_time) as f64;
            count += 1;
        }

        if count > 0 {
            self.avg_recovery_time = total_time / count as f64;
        }
    }
}

impl RecoveryState {
    /// Создает новое состояние восстановления
    pub fn new(
        id: String,
        state: QuantumState,
        state_type: RecoveryStateType,
        metadata: HashMap<String, String>
    ) -> Self {
        Self {
            id,
            state,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            state_type,
            metadata
        }
    }

    /// Добавляет чекпоинт
    pub fn add_checkpoint(&mut self, checkpoint: QuantumState) {
        self.state_type = RecoveryStateType::Checkpoint;
        self.state = checkpoint;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Восстанавливает состояние
    pub fn recover(&self) -> Result<QuantumState, String> {
        match &self.state_type {
            RecoveryStateType::Full => Ok(self.state.clone()),
            RecoveryStateType::Delta(delta) => {
                let mut state = self.state.clone();
                delta.apply_to_state(&mut state)?;
                Ok(state)
            },
            RecoveryStateType::Checkpoint => Ok(self.state.clone())
        }
    }
}

/// Протокол восстановления квантовых состояний
pub struct RecoveryProtocol {
    /// Чекпоинты состояний
    state_checkpoints: HashMap<String, QuantumState>,
    /// История дельт
    delta_history: VecDeque<QuantumDelta>,
    /// Журнал событий
    event_log: Vec<RecoveryEvent>,
    /// Максимальная длина истории дельт
    max_history_length: usize,
    /// Максимальное количество чекпоинтов
    max_checkpoints: usize,
    /// ID последнего чекпоинта
    last_checkpoint_id: Option<String>,
}

impl RecoveryProtocol {
    /// Создает новый протокол восстановления
    pub fn new() -> Self {
        Self {
            state_checkpoints: HashMap::new(),
            delta_history: VecDeque::new(),
            event_log: Vec::new(),
            max_history_length: 100,
            max_checkpoints: 10,
            last_checkpoint_id: None,
        }
    }
    
    /// Добавляет чекпоинт состояния
    pub fn add_state_checkpoint(&mut self, id: String, state: QuantumState) {
        // Сохраняем чекпоинт
        self.state_checkpoints.insert(id.clone(), state);
        self.last_checkpoint_id = Some(id.clone());
        
        // Если превышено максимальное количество чекпоинтов, удаляем самый старый
        if self.state_checkpoints.len() > self.max_checkpoints {
            let mut oldest_id = String::new();
            let mut oldest_time = u64::MAX;
            
            for (checkpoint_id, _) in &self.state_checkpoints {
                if let Ok(time) = checkpoint_id.parse::<u64>() {
                    if time < oldest_time {
                        oldest_time = time;
                        oldest_id = checkpoint_id.clone();
                    }
                }
            }
            
            if !oldest_id.is_empty() {
                self.state_checkpoints.remove(&oldest_id);
            }
        }
        
        // Записываем событие
        self.log_event(RecoveryEventType::DeltaApplied {
            success: true,
            delta_id: id.clone(),
        }, Some(&id));
    }
    
    /// Добавляет дельту в историю
    pub fn add_delta(&mut self, delta: QuantumDelta) {
        // Добавляем дельту в историю
        self.delta_history.push_back(delta.clone());
        
        // Если превышена максимальная длина истории, удаляем самую старую дельту
        if self.delta_history.len() > self.max_history_length {
            self.delta_history.pop_front();
        }
        
        // Записываем событие
        self.log_event(RecoveryEventType::DeltaApplied {
            success: true,
            delta_id: delta.id.clone(),
        }, Some(&delta.id));
    }
    
    /// Получает последний чекпоинт
    pub fn get_last_checkpoint(&self) -> Result<QuantumState, String> {
        if let Some(id) = &self.last_checkpoint_id {
            if let Some(state) = self.state_checkpoints.get(id) {
                return Ok(state.clone());
            }
        }
        
        Err("No checkpoint found".to_string())
    }
    
    /// Восстанавливает состояние на указанное время
    pub fn recover_state_at(&self, timestamp: u64) -> Result<QuantumState, String> {
        // Находим ближайший предшествующий чекпоинт
        let mut closest_checkpoint: Option<(&String, &QuantumState)> = None;
        let mut closest_time_diff = u64::MAX;
        
        for (id, state) in &self.state_checkpoints {
            if let Ok(checkpoint_time) = id.parse::<u64>() {
                if checkpoint_time <= timestamp {
                    let time_diff = timestamp - checkpoint_time;
                    if time_diff < closest_time_diff {
                        closest_time_diff = time_diff;
                        closest_checkpoint = Some((id, state));
                    }
                }
            }
        }
        
        // Если чекпоинт не найден, возвращаем ошибку
        let (checkpoint_id, checkpoint_state) = closest_checkpoint
            .ok_or_else(|| "No checkpoint found before the specified time".to_string())?;
            
        // Восстанавливаем состояние из чекпоинта
        let mut state = checkpoint_state.clone();
        
        // Применяем все дельты после чекпоинта и до указанного времени
        let mut applied_deltas = 0;
        for delta in &self.delta_history {
            if delta.timestamp > checkpoint_id.parse::<u64>().unwrap_or(0) 
               && delta.timestamp <= timestamp {
                if let Err(e) = delta.apply_to(&mut state) {
                    return Err(format!("Failed to apply delta {}: {}", delta.id, e));
                }
                applied_deltas += 1;
            }
        }
        
        if applied_deltas > 0 {
            println!("Applied {} deltas during recovery", applied_deltas);
        }
        
        Ok(state)
    }
    
    /// Применяет дельту к состоянию (для обратной совместимости)
    pub fn apply_state_delta(&mut self, delta: &QuantumDelta, state: &mut QuantumState) -> Result<(), String> {
        // Фиксируем событие применения дельты
        let event = RecoveryEvent::new(RecoveryEventType::DeltaApplied {
            success: true,
            delta_id: delta.id.clone(),
        });
        self.event_log.push(event);
        
        // Применяем дельту и добавляем её в историю, если успешно
        let result = delta.apply_to(state);
        
        // Если применение успешно, добавляем дельту в историю
        if result.is_ok() {
            self.add_delta(delta.clone());
        } else {
            // Записываем ошибку
            self.log_event(RecoveryEventType::Error, Some(&format!("Failed to apply delta {}", delta.id)));
        }
        
        result
    }
    
    /// Устанавливает максимальную длину истории дельт
    pub fn set_max_history_length(&mut self, length: usize) {
        self.max_history_length = length;
        
        // Если текущая длина превышает новый максимум, удаляем лишние дельты
        while self.delta_history.len() > self.max_history_length {
            self.delta_history.pop_front();
        }
    }
    
    /// Устанавливает максимальное количество чекпоинтов
    pub fn set_max_checkpoints(&mut self, count: usize) {
        self.max_checkpoints = count;
        
        // Если текущее количество превышает новый максимум, удаляем лишние чекпоинты
        if self.state_checkpoints.len() > self.max_checkpoints {
            let mut checkpoint_times = Vec::new();
            
            for checkpoint_id in self.state_checkpoints.keys() {
                if let Ok(time) = checkpoint_id.parse::<u64>() {
                    checkpoint_times.push((checkpoint_id.clone(), time));
                }
            }
            
            // Сортируем по времени
            checkpoint_times.sort_by(|a, b| a.1.cmp(&b.1));
            
            // Удаляем самые старые
            for i in 0..(self.state_checkpoints.len() - self.max_checkpoints) {
                if i < checkpoint_times.len() {
                    self.state_checkpoints.remove(&checkpoint_times[i].0);
                }
            }
        }
    }
    
    /// Получает журнал событий
    pub fn get_event_log(&self) -> &Vec<RecoveryEvent> {
        &self.event_log
    }
    
    /// Очищает историю дельт
    pub fn clear_delta_history(&mut self) {
        self.delta_history.clear();
    }
    
    /// Очищает чекпоинты
    pub fn clear_checkpoints(&mut self) {
        self.state_checkpoints.clear();
        self.last_checkpoint_id = None;
    }
    
    /// Записывает событие в журнал
    fn log_event(&mut self, event_type: RecoveryEventType, related_id: Option<&str>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut data = HashMap::new();
        if let Some(id) = related_id {
            data.insert("related_id".to_string(), id.to_string());
        }
        
        let event = RecoveryEvent::new(event_type);
        
        self.event_log.push(event);
    }

    /// Добавляет чекпоинт состояния (для обратной совместимости)
    pub fn add_checkpoint(&mut self, id: String, state: QuantumState) {
        self.add_state_checkpoint(id, state)
    }
    
    /// Восстанавливает состояние из последнего чекпоинта для указанного узла (для обратной совместимости)
    pub fn recover_state(&mut self, node_id: &str) -> Result<QuantumState, String> {
        self.get_last_checkpoint()
    }
}

impl RecoveryEvent {
    /// Создает новый объект события восстановления
    pub fn new(event_type: RecoveryEventType) -> Self {
        match event_type {
            RecoveryEventType::Success => Self::Success,
            RecoveryEventType::Failed(message) => Self::Failed(message),
            RecoveryEventType::Partial { reason, success_rate } => Self::Partial { reason, success_rate },
            RecoveryEventType::DeltaApplied { success, delta_id } => Self::DeltaApplied { success, delta_id },
            RecoveryEventType::Error => Self::Failed("Неизвестная ошибка".to_string())
        }
    }
} 