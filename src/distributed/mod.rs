// Модуль распределенной квантовой сети TRIAD
//
// Этот модуль реализует компоненты для работы TRIAD в распределенном режиме:
// 1. P2P сеть с полной децентрализацией
// 2. Механизмы квантовой синхронизации между узлами
// 3. Сбор метрик производительности распределенной сети

// Подмодули
pub mod quantum_peer;
pub mod quantum_protocol;
pub mod metrics;

// Реэкспорт основных компонентов для удобства
pub use quantum_peer::{QuantumPeer, TriadQuantumPeer};
pub use quantum_protocol::{QuantumMessage, EntanglementType, QubitId};
pub use metrics::DistributedMetrics;

/// Ошибки распределенного режима
#[derive(Debug, thiserror::Error)]
pub enum DistributedError {
    #[error("Ошибка соединения: {0}")]
    ConnectionError(String),
    
    #[error("Ошибка квантовой телепортации: {0}")]
    TeleportationError(String),
    
    #[error("Узел не найден: {0}")]
    PeerNotFound(String),
    
    #[error("Кубит не запутан: {0}")]
    QubitNotEntangled(QubitId),
    
    #[error("Несовместимые квантовые состояния")]
    IncompatibleQuantumStates,
    
    #[error("Ошибка десериализации: {0}")]
    DeserializationError(String),
    
    #[error("Непредвиденная ошибка: {0}")]
    UnexpectedError(String),
}

// Псевдоним для результата с распределенной ошибкой
pub type DistributedResult<T> = Result<T, DistributedError>; 