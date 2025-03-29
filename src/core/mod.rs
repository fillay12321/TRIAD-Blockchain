//! Ядро системы TRIAD: базовые типы и интерфейсы для квантовой симуляции.
//! Обеспечивает общие абстракции, которые могут использоваться различными бэкендами.

/// Типы и интерфейсы для работы с квантовыми состояниями
pub mod quantum_state;

/// Интерфейс для квантовых симуляторов
pub mod quantum_simulator;

/// Представление и операции с отдельными кубитами
pub mod qubit;

/// Определение типов и абстракций для квантовых гейтов
pub mod gates;

// Реэкспорт основных типов для удобства использования
pub use quantum_state::{QuantumState, Amplitude};
pub use quantum_simulator::QuantumSimulator;
pub use qubit::{Qubit, QubitState};
pub use gates::Gate; 