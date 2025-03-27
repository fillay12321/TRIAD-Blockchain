use std::time::Instant;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use rand::Rng;

/// Результаты сравнения алгоритмов консенсуса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusComparisonResults {
    pub node_counts: Vec<usize>,
    pub quantum_times: Vec<f64>,
    pub pow_times: Vec<f64>,
    pub pos_times: Vec<f64>,
    // Новые метрики для PoW
    pub pow_energy_consumption: Vec<f64>,      // В условных единицах энергии
    pub pow_block_propagation_time: Vec<f64>,  // В миллисекундах
    pub pow_real_life_estimation: Vec<f64>,    // Оценка реального времени в секундах
    // Информация о запутанности для квантового консенсуса
    pub quantum_entanglement: Vec<f64>,        // Уровень квантовой запутанности
}

/// Симулирует консенсус Proof of Work с более реалистичным подходом
pub fn simulate_pow_consensus(node_count: usize, difficulty: u32) -> (f64, f64, f64, f64) {
    let start = Instant::now();
    
    // Более реалистичная сложность
    // Bitcoin использует сложность, которая обеспечивает нахождение блока примерно раз в 10 минут
    // Здесь мы используем оценочную формулу, учитывая, что каждое увеличение difficulty на 1
    // увеличивает количество необходимых хешей в 2 раза
    
    // Целевое значение хеша (чем меньше, тем сложнее)
    let target = 2u128.pow(128 - difficulty);
    
    // Размер блока (в KB) - влияет на время распространения
    let block_size_kb = 1.0 + (node_count as f64 * 0.05); // Оценка: базовый размер + данные от узлов
    
    // Оценка количества хешей, которое нужно перебрать
    // В реальности это триллионы хешей
    let estimated_hashes = 2u128.pow(difficulty) / node_count as u128;
    
    // Симуляция с ограничением времени
    let mut found = false;
    let mut total_hashes = 0u128;
    let max_hashes_to_simulate = 1_000_000u128; // Ограничение для симуляции
    
    while !found && total_hashes < max_hashes_to_simulate {
        for _ in 0..node_count {
            let mut rng = rand::thread_rng();
            let nonce = rng.gen::<u64>();
            
            let mut hasher = Sha256::new();
            hasher.update(nonce.to_le_bytes());
            let hash = hasher.finalize();
            
            // Первые 16 байт как u128
            let hash_value = u128::from_le_bytes([
                hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
                hash[8], hash[9], hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]
            ]);
            
            if hash_value < target {
                found = true;
                break;
            }
            
            total_hashes += 1;
        }
    }
    
    // Время симуляции (в миллисекундах)
    let simulation_time = start.elapsed().as_secs_f64() * 1000.0;
    
    // Расчет метрик
    
    // 1. Реальное время в секундах (на основе сложности и мощности сети)
    // Bitcoin: ~10 минут на блок при текущей сложности 
    // Эта формула дает очень приблизительную оценку
    let real_hash_rate_per_node = 10.0e9; // 10 GH/s на узел (оценка средней мощности)
    let network_hash_rate = real_hash_rate_per_node * node_count as f64;
    let real_time_seconds = (estimated_hashes as f64) / network_hash_rate;
    
    // 2. Энергопотребление в кВт*ч (оценка)
    // Значения приблизительны и используются для относительного сравнения
    let energy_per_hash = 1.0e-10; // условных единиц энергии на хеш
    let energy_consumption = (estimated_hashes as f64) * energy_per_hash;
    
    // 3. Время распространения блока (зависит от размера блока и количества узлов)
    // Формула: базовое время + дополнительное время на каждый узел
    let base_propagation_time = 100.0; // 100 мс базовая задержка
    let node_factor = 1.5; // мс на узел
    let size_factor = 0.5; // мс на KB
    let block_propagation_time = base_propagation_time + 
                                (node_count as f64 * node_factor) + 
                                (block_size_kb * size_factor);
    
    // Возвращаем время симуляции и дополнительные метрики
    (simulation_time, energy_consumption, block_propagation_time, real_time_seconds)
}

/// Симулирует консенсус Proof of Stake (упрощенно)
pub fn simulate_pos_consensus(node_count: usize) -> f64 {
    // В PoS основное время занимает сбор и верификация подписей 
    // от валидаторов, а также сетевые задержки
    
    // Базовая задержка сети
    let base_latency_ms = 50.0;
    
    // Время на проверку подписи от каждого валидатора
    let signature_verification_ms = 2.0;
    
    // Общее время для PoS
    base_latency_ms + (signature_verification_ms * node_count as f64)
}

/// Сравнивает время консенсуса для различных алгоритмов
pub fn compare_consensus_mechanisms(node_counts: &[usize]) -> ConsensusComparisonResults {
    let mut quantum_times = Vec::with_capacity(node_counts.len());
    let mut pow_times = Vec::with_capacity(node_counts.len());
    let mut pos_times = Vec::with_capacity(node_counts.len());
    let mut pow_energy = Vec::with_capacity(node_counts.len());
    let mut pow_propagation = Vec::with_capacity(node_counts.len());
    let mut pow_real_time = Vec::with_capacity(node_counts.len());
    
    for &node_count in node_counts {
        // Для квантового консенсуса используем оценку
        // Реальный тест проводится отдельно
        let quantum_time = 1.0 + 10.0 + (0.1 * node_count as f64); // 1мс вычисление + 10мс сеть + 0.1мс на узел
        quantum_times.push(quantum_time);
        
        // Симуляция PoW с более реалистичными параметрами
        // Сложность увеличивается с размером сети для более реалистичной модели
        let scaling_difficulty = 16 + (node_count as f64 / 50.0).floor() as u32;
        let (pow_time, energy, propagation, real_time) = simulate_pow_consensus(node_count, scaling_difficulty);
        pow_times.push(pow_time);
        pow_energy.push(energy);
        pow_propagation.push(propagation);
        pow_real_time.push(real_time);
        
        // Симуляция PoS
        let pos_time = simulate_pos_consensus(node_count);
        pos_times.push(pos_time);
    }
    
    ConsensusComparisonResults {
        node_counts: node_counts.to_vec(),
        quantum_times,
        pow_times,
        pos_times,
        pow_energy_consumption: pow_energy,
        pow_block_propagation_time: pow_propagation,
        pow_real_life_estimation: pow_real_time,
        quantum_entanglement: Vec::new(),
    }
} 