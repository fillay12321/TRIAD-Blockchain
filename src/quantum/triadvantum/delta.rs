use crate::quantum::triadvantum::{
    state::QuantumState,
};
use num_complex::Complex64;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::fmt;
use std::cmp;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Константы сжатия для оптимизации дельт
const COMPRESSION_THRESHOLD: f64 = 1e-6;
const MAX_SIGNIFICANT_AMPLITUDES: usize = 1024;

/// Типы дельт
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeltaType {
    /// Дельта амплитуд
    Amplitude,
    /// Дельта измерений
    Measurement,
    /// Дельта структуры
    Structural,
    /// Дельта метаданных
    Metadata
}

/// Дельта квантового состояния
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumDelta {
    /// Уникальный идентификатор дельты
    pub id: String,
    /// Исходное состояние (перед изменениями)
    pub old_state: QuantumState,
    /// Новое состояние (после изменений)
    pub new_state: QuantumState,
    /// Идентификатор создателя дельты
    pub creator_id: String,
    /// Временная метка создания дельты
    pub timestamp: u64,
    /// Измененные амплитуды (индекс -> новое значение)
    pub changed_amplitudes: HashMap<usize, Complex64>,
}

impl QuantumDelta {
    /// Создает новую дельту между двумя состояниями
    pub fn new(
        id: String,
        old_state: QuantumState,
        new_state: QuantumState,
        creator_id: String,
        timestamp: u64,
    ) -> Self {
        // Находим измененные амплитуды
        let mut changed_amplitudes = HashMap::new();
        
        // Если количество кубитов различается, считаем все амплитуды измененными
        if old_state.qubit_count() != new_state.qubit_count() {
            for i in 0..new_state.amplitudes_len() {
                changed_amplitudes.insert(i, new_state.get_amplitude(i));
            }
        } else {
            // Сравниваем амплитуды и записываем изменения
            for i in 0..old_state.amplitudes_len() {
                let old_amp = old_state.get_amplitude(i);
                let new_amp = new_state.get_amplitude(i);
                
                // Если амплитуды различаются, добавляем их в дельту
                if (old_amp - new_amp).norm() > 1e-10 {
                    changed_amplitudes.insert(i, new_amp);
                }
            }
        }
        
        Self {
            id,
            old_state,
            new_state,
            creator_id,
            timestamp,
            changed_amplitudes,
        }
    }
    
    /// Создает дельту только для изменения амплитуд
    pub fn new_amplitude_delta(
        id: String,
        base_state: QuantumState,
        creator_id: String,
        timestamp: u64,
        changed_amplitudes: HashMap<usize, Complex64>
    ) -> Self {
        // Создаем копию исходного состояния
        let mut new_state = base_state.clone();
        
        // Применяем изменения
        for (&idx, &amp) in &changed_amplitudes {
            if idx < new_state.amplitudes_len() {
                new_state.set_amplitude(idx, amp);
            }
        }
        
        // Нормализуем новое состояние
        new_state.normalize();
        
        Self {
            id,
            old_state: base_state,
            new_state,
            creator_id,
            timestamp,
            changed_amplitudes,
        }
    }
    
    /// Применяет дельту к указанному состоянию
    pub fn apply_to(&self, state: &mut QuantumState) -> Result<(), String> {
        // Проверяем совместимость состояний
        if state.qubit_count != self.old_state.qubit_count {
            return Err(format!("Несовместимое количество кубитов: {} vs {}", 
                              state.qubit_count, self.old_state.qubit_count));
        }
        
        // Применяем изменения амплитуд
        for (&idx, &amp) in &self.changed_amplitudes {
            if idx < state.amplitudes_len() {
                state.set_amplitude(idx, amp);
            }
        }
        
        // Нормализуем состояние
        state.normalize();
        
        Ok(())
    }
    
    /// Применяет дельту к квантовому состоянию (алиас для apply_to)
    pub fn apply_to_state(&self, state: &mut QuantumState) -> Result<(), String> {
        self.apply_to(state)
    }
    
    /// Получает размер дельты в байтах
    pub fn size_in_bytes(&self) -> usize {
        // Приблизительный расчёт
        let base_size = 24; // Заголовок и базовые поля
        let amplitudes_size = self.changed_amplitudes.len() * 24; // Индекс (8) + Complex64 (16)
        base_size + self.id.len() + self.creator_id.len() + amplitudes_size
    }
    
    /// Вычисляет размер дельты (количество измененных амплитуд)
    pub fn size(&self) -> usize {
        self.changed_amplitudes.len()
    }
    
    /// Вычисляет эффективность дельты (соотношение к полному размеру)
    pub fn efficiency(&self) -> f64 {
        if self.old_state.amplitudes_len() == 0 {
            return 0.0;
        }
        1.0 - self.size() as f64 / self.old_state.amplitudes_len() as f64
    }
    
    /// Объединяет две дельты
    pub fn merge(&self, other: &Self) -> Result<Self, String> {
        // Проверяем совместимость состояний
        if self.new_state.qubit_count != other.old_state.qubit_count {
            return Err(format!("Несовместимые дельты: {} кубитов vs {} кубитов",
                              self.new_state.qubit_count, other.old_state.qubit_count));
        }
        
        // Определяем идентификатор и timestamp
        let id = format!("merged_{}_{}", self.id, other.id);
        let timestamp = std::cmp::max(self.timestamp, other.timestamp);
        
        // Создаем копию измененных амплитуд из первой дельты
        let mut changed_amplitudes = self.changed_amplitudes.clone();
        
        // Добавляем измененные амплитуды из второй дельты
        for (&idx, &amp) in &other.changed_amplitudes {
            changed_amplitudes.insert(idx, amp);
        }
        
        Ok(Self::new_amplitude_delta(
            id,
            self.old_state.clone(),
            self.creator_id.clone(),
            timestamp,
            changed_amplitudes
        ))
    }
}

/// Компрессор дельта-состояний для оптимизации передачи
pub struct DeltaCompressor {
    /// Максимальный размер дельты (в процентах)
    max_delta_size: usize,
    /// Кэш последних состояний
    state_cache: HashMap<String, QuantumState>,
    /// Счетчик дельт
    delta_counter: usize,
}

impl DeltaCompressor {
    /// Создает новый компрессор дельт
    pub fn new(max_delta_size: usize) -> Self {
        Self {
            max_delta_size,
            state_cache: HashMap::new(),
            delta_counter: 0,
        }
    }
    
    /// Создает дельту из одного состояния (для обратной совместимости)
    pub fn create_delta(&mut self, state: &QuantumState) -> QuantumDelta {
        let default_state = QuantumState::new(state.qubit_count);
        self.create_delta_full(&default_state, state, "default".to_string())
    }
    
    /// Создает полную дельту с учетом всех параметров
    pub fn create_delta_full(&mut self, old_state: &QuantumState, new_state: &QuantumState, creator_id: String) -> QuantumDelta {
        // Убеждаемся, что состояния имеют одинаковое количество кубитов
        if old_state.qubit_count() != new_state.qubit_count() {
            panic!("Несовместимое количество кубитов при создании дельты");
        }
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let delta_id = format!("delta_{}_{}", creator_id, timestamp);
        
        // Находим измененные амплитуды
        let mut changed_amplitudes = HashMap::new();
        let old_amplitudes = old_state.get_amplitudes();
        let new_amplitudes = new_state.get_amplitudes();
        
        for idx in 0..new_amplitudes.len() {
            let new_amp = new_amplitudes[idx];
            if idx < old_amplitudes.len() {
                let old_amp = old_amplitudes[idx];
                if (new_amp - old_amp).norm() > COMPRESSION_THRESHOLD {
                    changed_amplitudes.insert(idx, new_amp);
                }
            } else {
                if new_amp.norm() > COMPRESSION_THRESHOLD {
                    changed_amplitudes.insert(idx, new_amp);
                }
            }
        }
        
        // Ограничиваем размер дельты
        if changed_amplitudes.len() > self.max_delta_size {
            // Находим наиболее значимые изменения
            let mut changes: Vec<_> = changed_amplitudes.iter().collect();
            changes.sort_by(|a, b| b.1.norm().partial_cmp(&a.1.norm()).unwrap());
            
            // Оставляем только max_delta_size самых значимых изменений
            changed_amplitudes = changes
                .iter()
                .take(self.max_delta_size)
                .map(|(&idx, &amp)| (idx, amp))
                .collect();
        }
        
        QuantumDelta {
            id: delta_id,
            old_state: old_state.clone(),
            new_state: new_state.clone(),
            creator_id,
            timestamp,
            changed_amplitudes,
        }
    }
    
    /// Сохраняет состояние в кэше
    pub fn cache_state(&mut self, id: String, state: QuantumState) {
        self.state_cache.insert(id, state);
    }
    
    /// Получает состояние из кэша
    pub fn get_cached_state(&self, id: &str) -> Option<&QuantumState> {
        self.state_cache.get(id)
    }
    
    /// Очищает кэш состояний
    pub fn clear_cache(&mut self) {
        self.state_cache.clear();
    }
}

/// Группировщик дельт для оптимизации сетевого трафика
pub struct DeltaBatcher {
    /// Буфер для накопления дельт
    batch_buffer: Vec<QuantumDelta>,
    /// Максимальный размер пакета дельт в байтах
    max_batch_size: usize,
    /// Максимальное количество дельт в пакете
    max_delta_count: usize,
    /// Время создания первой дельты в пакете
    first_delta_time: Option<Instant>,
    /// Максимальное время ожидания перед отправкой неполного пакета (в миллисекундах)
    max_wait_time_ms: u64,
}

impl DeltaBatcher {
    /// Создает новый группировщик дельт
    pub fn new(max_batch_size: usize, max_delta_count: usize, max_wait_time_ms: u64) -> Self {
        Self {
            batch_buffer: Vec::with_capacity(max_delta_count),
            max_batch_size,
            max_delta_count,
            first_delta_time: None,
            max_wait_time_ms,
        }
    }
    
    /// Добавляет дельту в пакет и возвращает готовый пакет, если он заполнен
    pub fn add_delta(&mut self, delta: QuantumDelta) -> Option<Vec<QuantumDelta>> {
        // Если буфер пуст, запоминаем время первой дельты
        if self.batch_buffer.is_empty() {
            self.first_delta_time = Some(Instant::now());
        }
        
        // Добавляем дельту в буфер
        self.batch_buffer.push(delta);
        
        // Если буфер заполнен, возвращаем готовый пакет
        if self.should_send_batch() {
            return Some(self.take_batch());
        }
        
        None
    }
    
    /// Проверяет, готов ли пакет для отправки
    fn should_send_batch(&self) -> bool {
        // Если буфер пуст, пакет не готов
        if self.batch_buffer.is_empty() {
            return false;
        }
        
        // Если достигнут лимит по количеству дельт
        if self.batch_buffer.len() >= self.max_delta_count {
            return true;
        }
        
        // Если достигнут лимит по размеру данных
        let current_size = self.batch_buffer.iter()
                            .map(|delta| delta.size_in_bytes())
                            .sum::<usize>();
        if current_size >= self.max_batch_size {
            return true;
        }
        
        // Если прошло достаточно времени с момента добавления первой дельты
        if let Some(first_time) = self.first_delta_time {
            let elapsed = first_time.elapsed();
            let elapsed_ms = elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64;
            if elapsed_ms >= self.max_wait_time_ms {
                return true;
            }
        }
        
        false
    }
    
    /// Возвращает и очищает текущий пакет дельт
    pub fn take_batch(&mut self) -> Vec<QuantumDelta> {
        let result = std::mem::take(&mut self.batch_buffer);
        self.first_delta_time = None;
        result
    }
    
    /// Проверяет, есть ли готовый к отправке пакет
    pub fn check_pending_batch(&mut self) -> Option<Vec<QuantumDelta>> {
        if self.should_send_batch() {
            Some(self.take_batch())
        } else {
            None
        }
    }
    
    /// Возвращает количество ожидающих отправки дельт
    pub fn pending_count(&self) -> usize {
        self.batch_buffer.len()
    }
    
    /// Возвращает приблизительный размер ожидающих отправки дельт в байтах
    pub fn pending_size(&self) -> usize {
        self.batch_buffer.iter()
            .map(|delta| delta.size_in_bytes())
            .sum()
    }
}

/// Генерирует уникальный идентификатор дельты
fn generate_delta_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let now = current_timestamp();
    let random = rand::random::<u32>();
    
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    random.hash(&mut hasher);
    
    hasher.finish()
}

/// Возвращает текущую временную метку в секундах
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
} 