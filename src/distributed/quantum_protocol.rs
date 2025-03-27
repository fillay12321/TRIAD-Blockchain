// Протокол квантовой синхронизации для TRIAD
//
// Определяет форматы сообщений для обмена квантовыми состояниями между узлами

use serde::{Serialize, Deserialize};
use num_complex::Complex64;
use crate::quantum::triadvantum::state::QuantumState;

/// Идентификатор кубита в распределенной сети
pub type QubitId = u64;

/// Тип квантовой запутанности
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EntanglementType {
    /// Полностью запутанные состояния (максимальная корреляция)
    Bell,
    /// Частично запутанные состояния
    Partial(f64),
    /// Тройная запутанность GHZ
    GHZ,
    /// Кластерная запутанность для нескольких узлов
    Cluster,
}

/// Квантовое событие для обработки с высшим приоритетом
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumEvent {
    /// Уникальный идентификатор события
    pub event_id: String,
    /// Идентификатор инициатора события
    pub originator: String,
    /// Временная метка создания события
    pub timestamp: u64,
    /// Приоритет события (для телепортации всегда максимальный)
    pub priority: u8,
    /// Полезная нагрузка события
    pub payload: QuantumEventPayload,
}

/// Типы полезной нагрузки квантового события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumEventPayload {
    /// Телепортация квантового состояния
    StateTransfer {
        /// Целевые кубиты
        qubit_ids: Vec<QubitId>,
        /// Телепортируемое состояние
        quantum_state: QuantumState,
        /// Квантовая подпись для проверки целостности
        quantum_signature: [u8; 32],
    },
    
    /// Запутывание удаленных кубитов
    Entanglement {
        /// Локальные кубиты
        local_qubits: Vec<QubitId>,
        /// Идентификатор удаленного узла
        remote_peer: String,
        /// Удаленные кубиты
        remote_qubits: Vec<QubitId>,
        /// Тип запутанности
        entanglement_type: EntanglementType,
    },
    
    /// Разрешение конфликта через интерференцию
    Interference {
        /// Идентификатор интерференционного паттерна
        pattern_id: String,
        /// Конфликтующие кубиты
        conflicting_qubits: Vec<QubitId>,
        /// Фазовые сдвиги для интерференции
        phase_shifts: Vec<Complex64>,
    },
}

/// Сообщения протокола квантовой сети
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumMessage {
    /// Обнаружение и структура сети
    DiscoverPeers { peer_id: String, address: String },
    PeerInfo { peers: Vec<(String, String)> },
    
    /// Квантовые события (обрабатываются с высшим приоритетом)
    QuantumEvent(QuantumEvent),
    QuantumAck { event_id: String, peer_id: String },
    
    /// Транзакции
    ProposeTransaction { 
        tx_id: String, 
        data: String,
        proposer: String,
        timestamp: u64 
    },
    TransactionVote { 
        tx_id: String, 
        voter: String,
        vote: bool, 
        quantum_signature: Vec<u8> 
    },
    
    /// Мониторинг и метрики
    MetricsUpdate { 
        peer_id: String,
        nodes: usize,
        tps: f64,
        latency_ms: f64, 
        entanglement: f64 
    },
}

impl QuantumMessage {
    /// Проверяет, является ли сообщение квантовым событием
    pub fn is_quantum_event(&self) -> bool {
        matches!(self, QuantumMessage::QuantumEvent(_))
    }
    
    /// Получает идентификатор сообщения (если применимо)
    pub fn get_id(&self) -> Option<String> {
        match self {
            QuantumMessage::QuantumEvent(event) => Some(event.event_id.clone()),
            QuantumMessage::QuantumAck { event_id, .. } => Some(event_id.clone()),
            QuantumMessage::ProposeTransaction { tx_id, .. } => Some(tx_id.clone()),
            QuantumMessage::TransactionVote { tx_id, .. } => Some(tx_id.clone()),
            _ => None,
        }
    }
    
    /// Получает приоритет сообщения
    pub fn get_priority(&self) -> u8 {
        match self {
            QuantumMessage::QuantumEvent(_) => 255, // Максимальный приоритет
            QuantumMessage::QuantumAck { .. } => 200,
            QuantumMessage::ProposeTransaction { .. } => 100,
            QuantumMessage::TransactionVote { .. } => 100,
            _ => 50,
        }
    }
} 