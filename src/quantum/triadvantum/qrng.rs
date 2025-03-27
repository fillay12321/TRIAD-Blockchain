use crate::quantum::triadvantum::{
    QuantumState,
    gates::QuantumGate,
    circuit::QuantumCircuit,
    simulator::QrustSimulator
};
use num_complex::Complex64;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Квантовый генератор случайных чисел
pub struct QuantumRNG {
    /// Внутренний симулятор
    simulator: QrustSimulator,
    /// Количество кубитов
    qubit_count: usize,
    /// Количество сгенерированных чисел
    generated_count: usize,
    /// Статистика
    stats: RNGStats,
    /// Схема для генерации случайных чисел
    circuit: QuantumCircuit
}

/// Статистика генератора случайных чисел
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RNGStats {
    /// Количество сгенерированных чисел
    pub generated_count: usize,
    /// Средняя энтропия (0-1)
    pub avg_entropy: f64,
    /// Среднее время генерации в микросекундах
    pub avg_generation_time: f64,
    /// Время последней генерации в микросекундах
    pub last_generation_time: u64,
    /// Распределение битов (0/1)
    pub bit_distribution: [usize; 2]
}

impl QuantumRNG {
    /// Создает новый генератор случайных чисел
    pub fn new(qubit_count: usize) -> Result<Self, String> {
        if qubit_count < 1 || qubit_count > 32 {
            return Err("Количество кубитов должно быть от 1 до 32".to_string());
        }
        
        // Создаем симулятор
        let simulator = QrustSimulator::new("qrng".to_string(), qubit_count, false)?;
        
        // Создаем схему для генерации (гейты Адамара на все кубиты)
        let mut circuit = QuantumCircuit::new(qubit_count);
        for i in 0..qubit_count {
            circuit.h(i);
        }
        
        Ok(Self {
            simulator,
            qubit_count,
            generated_count: 0,
            stats: RNGStats {
                generated_count: 0,
                avg_entropy: 0.0,
                avg_generation_time: 0.0,
                last_generation_time: 0,
                bit_distribution: [0, 0]
            },
            circuit
        })
    }
    
    /// Генерирует случайное целое число от 0 до 2^qubit_count - 1
    pub fn generate_int(&mut self) -> Result<u64, String> {
        let start_time = SystemTime::now();
        
        // Выполняем схему
        self.simulator.run_circuit(&self.circuit)?;
        
        // Измеряем все кубиты
        let mut result: u64 = 0;
        let mut bit_counts = [0, 0];
        
        for i in 0..self.qubit_count {
            let bit = self.simulator.measure_qubit(i)? as u64;
            result |= bit << i;
            bit_counts[bit as usize] += 1;
        }
        
        // Обновляем статистику
        self.generated_count += 1;
        self.stats.generated_count += 1;
        
        // Обновляем распределение битов
        self.stats.bit_distribution[0] += bit_counts[0];
        self.stats.bit_distribution[1] += bit_counts[1];
        
        // Вычисляем энтропию
        let p0 = bit_counts[0] as f64 / self.qubit_count as f64;
        let p1 = bit_counts[1] as f64 / self.qubit_count as f64;
        let entropy = if p0 > 0.0 && p1 > 0.0 {
            -p0 * p0.log2() - p1 * p1.log2()
        } else {
            0.0
        };
        
        // Обновляем среднюю энтропию
        self.stats.avg_entropy = (self.stats.avg_entropy * (self.stats.generated_count - 1) as f64 
                                 + entropy) / self.stats.generated_count as f64;
        
        // Вычисляем время генерации
        let elapsed = start_time.elapsed().unwrap_or_default();
        let elapsed_micros = elapsed.as_micros() as u64;
        
        // Обновляем время генерации
        self.stats.last_generation_time = elapsed_micros;
        self.stats.avg_generation_time = (self.stats.avg_generation_time * (self.stats.generated_count - 1) as f64 
                                         + elapsed_micros as f64) / self.stats.generated_count as f64;
        
        Ok(result)
    }
    
    /// Генерирует случайное число от 0 до 1
    pub fn generate_float(&mut self) -> Result<f64, String> {
        let max_value = (1u64 << self.qubit_count) as f64;
        let rand_int = self.generate_int()?;
        
        Ok(rand_int as f64 / max_value)
    }
    
    /// Генерирует случайное число в заданном диапазоне [min, max)
    pub fn generate_in_range(&mut self, min: f64, max: f64) -> Result<f64, String> {
        if min >= max {
            return Err("Минимальное значение должно быть меньше максимального".to_string());
        }
        
        let rand_float = self.generate_float()?;
        Ok(min + rand_float * (max - min))
    }
    
    /// Генерирует случайное целое число в заданном диапазоне [min, max)
    pub fn generate_int_in_range(&mut self, min: i64, max: i64) -> Result<i64, String> {
        if min >= max {
            return Err("Минимальное значение должно быть меньше максимального".to_string());
        }
        
        let range = (max - min) as u64;
        let max_qubits = 64 - range.leading_zeros() as usize;
        
        // Если диапазон требует больше кубитов, чем доступно, используем модульную арифметику
        if max_qubits > self.qubit_count {
            let rand_int = self.generate_int()?;
            return Ok(min + (rand_int % range) as i64);
        }
        
        // Иначе генерируем число точно в диапазоне
        loop {
            let rand_int = self.generate_int()?;
            if rand_int < range {
                return Ok(min + rand_int as i64);
            }
        }
    }
    
    /// Получает статистику
    pub fn get_stats(&self) -> &RNGStats {
        &self.stats
    }
    
    /// Сбрасывает статистику
    pub fn reset_stats(&mut self) {
        self.stats = RNGStats {
            generated_count: 0,
            avg_entropy: 0.0,
            avg_generation_time: 0.0,
            last_generation_time: 0,
            bit_distribution: [0, 0]
        };
    }
    
    /// Проверяет качество генератора
    pub fn test_quality(&mut self, samples: usize) -> Result<QualityTestResult, String> {
        if samples < 100 {
            return Err("Для теста необходимо минимум 100 образцов".to_string());
        }
        
        let mut values = Vec::with_capacity(samples);
        
        // Генерируем образцы
        for _ in 0..samples {
            values.push(self.generate_float()?);
        }
        
        // Вычисляем среднее
        let avg: f64 = values.iter().sum::<f64>() / samples as f64;
        
        // Вычисляем дисперсию
        let variance: f64 = values.iter()
            .map(|&v| (v - avg).powi(2))
            .sum::<f64>() / samples as f64;
        
        // Вычисляем автокорреляцию для лага 1
        let mut autocorr = 0.0;
        for i in 1..samples {
            autocorr += (values[i] - avg) * (values[i-1] - avg);
        }
        autocorr /= (samples - 1) as f64 * variance;
        
        // Проверка равномерности (тест хи-квадрат)
        let num_bins = 10;
        let mut bins = vec![0; num_bins];
        
        for &value in &values {
            let bin = (value * num_bins as f64).floor() as usize;
            let bin = bin.min(num_bins - 1); // На случай, если value = 1.0
            bins[bin] += 1;
        }
        
        let expected = samples as f64 / num_bins as f64;
        let chi_squared: f64 = bins.iter()
            .map(|&count| (count as f64 - expected).powi(2) / expected)
            .sum();
        
        Ok(QualityTestResult {
            samples,
            mean: avg,
            variance,
            autocorrelation: autocorr,
            chi_squared,
            passed: chi_squared < 16.919 && autocorr.abs() < 0.2 // 95% уровень для 9 степеней свободы
        })
    }
}

/// Результаты теста качества генератора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTestResult {
    /// Количество образцов
    pub samples: usize,
    /// Среднее значение
    pub mean: f64,
    /// Дисперсия
    pub variance: f64,
    /// Автокорреляция
    pub autocorrelation: f64,
    /// Результат теста хи-квадрат
    pub chi_squared: f64,
    /// Прохождение теста
    pub passed: bool
} 