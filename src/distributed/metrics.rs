// Метрики распределенной квантовой сети TRIAD
//
// Собирает и анализирует данные о производительности распределенной сети,
// включая квантовую когерентность, задержки при телепортации и эффективность.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::distributed::quantum_protocol::QubitId;

/// Метрики распределенного узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedMetrics {
    /// Идентификатор узла
    pub peer_id: String,
    
    /// Метки времени начала и последнего обновления
    pub start_time: u64,
    pub last_update: u64,
    
    /// Сетевые метрики
    pub network: NetworkMetrics,
    
    /// Квантовые метрики
    pub quantum: QuantumMetrics,
    
    /// Метрики производительности транзакций
    pub transactions: TransactionMetrics,
    
    /// Статистика по соединениям с другими узлами
    pub peer_stats: HashMap<String, PeerStats>,
}

/// Сетевые метрики
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// Общее количество отправленных сообщений
    pub total_messages_sent: usize,
    
    /// Общее количество полученных сообщений
    pub total_messages_received: usize,
    
    /// Объем отправленных данных (байты)
    pub bytes_sent: usize,
    
    /// Объем полученных данных (байты)
    pub bytes_received: usize,
    
    /// Средняя задержка сети (мс)
    pub avg_network_latency_ms: f64,
    
    /// Число активных соединений
    pub active_connections: usize,
    
    /// Пропускная способность (сообщений/сек)
    pub messages_per_second: f64,
}

/// Квантовые метрики
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuantumMetrics {
    /// Количество успешных телепортаций состояний
    pub successful_teleportations: usize,
    
    /// Общее количество попыток телепортации
    pub total_teleportation_attempts: usize,
    
    /// Среднее время телепортации (мкс)
    pub avg_teleportation_time_us: f64,
    
    /// Средний уровень запутанности
    pub avg_entanglement_level: f64,
    
    /// Число запутанных кубитов
    pub entangled_qubits: usize,
    
    /// Распределение запутанных кубитов по узлам
    pub entanglement_distribution: HashMap<String, usize>,
    
    /// Частота когерентности (% времени поддержания когерентного состояния)
    pub coherence_rate: f64,
    
    /// Счетчик конфликтов квантовых состояний
    pub quantum_state_conflicts: usize,
    
    /// Эффективность разрешения конфликтов через интерференцию
    pub interference_resolution_rate: f64,
}

/// Метрики производительности транзакций
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionMetrics {
    /// Транзакций в секунду
    pub tps: f64,
    
    /// Средняя латентность транзакций (мс)
    pub avg_latency_ms: f64,
    
    /// Общее количество обработанных транзакций
    pub processed_transactions: usize,
    
    /// Число транзакций, ожидающих обработки
    pub pending_transactions: usize,
    
    /// Число транзакций, достигших консенсуса
    pub consensus_transactions: usize,
    
    /// Частота достижения консенсуса (%)
    pub consensus_rate: f64,
}

/// Статистика по соединению с другим узлом
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerStats {
    /// Адрес узла
    pub address: String,
    
    /// Время установления соединения
    pub connected_since: u64,
    
    /// Задержка соединения (мс)
    pub latency_ms: f64,
    
    /// Уровень запутанности с этим узлом (0.0 - 1.0)
    pub entanglement_level: f64,
    
    /// Количество телепортаций с этим узлом
    pub teleportation_count: usize,
    
    /// Среднее время телепортации с этим узлом (мкс)
    pub avg_teleportation_time_us: f64,
}

impl DistributedMetrics {
    /// Создает новый экземпляр метрик
    pub fn new(peer_id: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            peer_id,
            start_time: now,
            last_update: now,
            network: NetworkMetrics::default(),
            quantum: QuantumMetrics::default(),
            transactions: TransactionMetrics::default(),
            peer_stats: HashMap::new(),
        }
    }
    
    /// Регистрирует новый узел в статистике
    pub fn register_peer(&mut self, peer_id: String, address: String) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let mut stats = PeerStats::default();
        stats.address = address;
        stats.connected_since = now;
        
        self.peer_stats.insert(peer_id, stats);
        self.network.active_connections = self.peer_stats.len();
    }
    
    /// Регистрирует квантовую телепортацию
    pub fn register_teleportation(&mut self, peer_id: Option<String>, succeeded: bool, duration_us: f64) {
        self.quantum.total_teleportation_attempts += 1;
        
        if succeeded {
            self.quantum.successful_teleportations += 1;
            
            // Обновляем среднее время телепортации
            let total_time = self.quantum.avg_teleportation_time_us 
                * (self.quantum.successful_teleportations - 1) as f64;
            self.quantum.avg_teleportation_time_us = 
                (total_time + duration_us) / self.quantum.successful_teleportations as f64;
                
            // Если известен узел, обновляем его статистику
            if let Some(peer_id) = peer_id {
                if let Some(stats) = self.peer_stats.get_mut(&peer_id) {
                    stats.teleportation_count += 1;
                    
                    let total_time = stats.avg_teleportation_time_us 
                        * (stats.teleportation_count - 1) as f64;
                    stats.avg_teleportation_time_us = 
                        (total_time + duration_us) / stats.teleportation_count as f64;
                }
            }
        }
    }
    
    /// Регистрирует обработку транзакции
    pub fn register_transaction(&mut self, latency_ms: f64, consensus_reached: bool) {
        self.transactions.processed_transactions += 1;
        
        // Обновляем среднюю латентность
        let total_latency = self.transactions.avg_latency_ms 
            * (self.transactions.processed_transactions - 1) as f64;
        self.transactions.avg_latency_ms = 
            (total_latency + latency_ms) / self.transactions.processed_transactions as f64;
            
        // Обновляем статистику консенсуса
        if consensus_reached {
            self.transactions.consensus_transactions += 1;
        }
        
        self.transactions.consensus_rate = 
            self.transactions.consensus_transactions as f64 
            / self.transactions.processed_transactions as f64;
            
        // Обновляем TPS
        let elapsed_sec = (chrono::Utc::now().timestamp_millis() as u64 - self.start_time) as f64 / 1000.0;
        if elapsed_sec > 0.0 {
            self.transactions.tps = self.transactions.processed_transactions as f64 / elapsed_sec;
        }
    }
    
    /// Обновляет метрики на основе текущего состояния
    pub fn update(&mut self) {
        self.last_update = chrono::Utc::now().timestamp_millis() as u64;
    }
    
    /// Получает краткое строковое представление метрик
    pub fn summary(&self) -> String {
        format!(
            "TRIAD Узел {}: TPS={:.2}, Latency={:.2}ms, Teleport={}/{}, Entanglement={:.2}, Peers={}",
            self.peer_id,
            self.transactions.tps,
            self.transactions.avg_latency_ms,
            self.quantum.successful_teleportations,
            self.quantum.total_teleportation_attempts,
            self.quantum.avg_entanglement_level,
            self.peer_stats.len()
        )
    }
} 