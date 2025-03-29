//! Пример генерации квантовых случайных чисел.
//! 
//! Этот пример демонстрирует простой способ использования квантового симулятора
//! для генерации истинно случайных чисел, используя принципы квантовой механики.

use crate::core::quantum_simulator::QuantumSimulator;
use crate::quest::QuESTSimulator;
use std::time::{Duration, Instant};
use env_logger;

/// Генерирует случайный бит (0 или 1) с использованием квантового симулятора
pub fn generate_random_bit() -> bool {
    // Создаем симулятор с одним кубитом
    let mut simulator = QuESTSimulator::new(1);
    
    // Проверяем, что симулятор корректно инициализирован
    assert_eq!(simulator.num_qubits(), 1, "Симулятор должен иметь ровно 1 кубит");
    
    // Применяем гейт Адамара для создания суперпозиции
    simulator.hadamard(0);
    
    // Измеряем кубит, получая случайный результат
    simulator.measure(0)
}

/// Генерирует случайное число в диапазоне [0, max_value)
pub fn generate_random_number(max_value: usize) -> usize {
    // Определяем количество бит, необходимых для представления числа
    let num_bits = (max_value as f64).log2().ceil() as usize;
    
    // Генерируем случайные биты и составляем из них число
    let mut result = 0;
    for i in 0..num_bits {
        let bit = generate_random_bit();
        if bit {
            result |= 1 << i;
        }
    }
    
    // Ограничиваем результат диапазоном [0, max_value)
    result % max_value
}

/// Проверяет статистику распределения случайных чисел
pub fn random_number_statistics(max_value: usize, num_samples: usize) -> Vec<usize> {
    // Создаем гистограмму для подсчета частоты каждого числа
    let mut histogram = vec![0; max_value];
    
    // Генерируем случайные числа и обновляем гистограмму
    for _ in 0..num_samples {
        let num = generate_random_number(max_value);
        histogram[num] += 1;
    }
    
    histogram
}

/// Измеряет время, затрачиваемое на генерацию случайных битов
pub fn measure_random_bit_generation_time(num_bits: usize) -> (f64, usize) {
    let start = Instant::now();
    let mut count_ones = 0;
    
    // Генерируем указанное количество случайных битов
    for _ in 0..num_bits {
        if generate_random_bit() {
            count_ones += 1;
        }
    }
    
    let elapsed = start.elapsed();
    let time_per_bit = elapsed.as_secs_f64() / num_bits as f64;
    
    (time_per_bit, count_ones)
}

/// Демонстрирует работу квантового генератора случайных чисел
pub fn demonstrate_random_number_generation() {
    println!("Демонстрация квантового генератора случайных чисел:");
    
    // Генерируем и выводим несколько случайных битов
    println!("\nГенерация случайных битов:");
    for i in 1..=10 {
        let bit = generate_random_bit();
        println!("Случайный бит #{}: {}", i, bit);
    }
    
    // Генерируем и выводим несколько случайных чисел
    println!("\nГенерация случайных чисел от 0 до 99:");
    for i in 1..=10 {
        let num = generate_random_number(100);
        println!("Случайное число #{}: {}", i, num);
    }
    
    // Проверяем статистику распределения
    println!("\nСтатистика распределения случайных чисел от 0 до 9:");
    let num_samples = 1000;
    let histogram = random_number_statistics(10, num_samples);
    
    for (i, count) in histogram.iter().enumerate() {
        let percentage = (*count as f64 / num_samples as f64) * 100.0;
        println!("Число {}: {} раз ({}%)", i, count, percentage.round());
    }
    
    // Измеряем производительность
    println!("\nПроизводительность генерации случайных битов:");
    let (time_per_bit, count_ones) = measure_random_bit_generation_time(1000);
    println!("Среднее время на генерацию одного бита: {:.6} с", time_per_bit);
    println!("Статистика: {} единиц из 1000 бит ({}%)", 
              count_ones, (count_ones as f64 / 10.0).round());
}

fn main() {
    // Инициализируем логирование
    env_logger::init();
    
    // Запускаем демонстрацию
    demonstrate_random_number_generation();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_bit_generation() {
        // Проверяем, что функция не вызывает ошибок
        let _bit = generate_random_bit();
    }
    
    #[test]
    fn test_random_number_generation() {
        // Проверяем, что сгенерированное число находится в правильном диапазоне
        let max_value = 100;
        for _ in 0..10 {
            let num = generate_random_number(max_value);
            assert!(num < max_value, "Сгенерированное число должно быть меньше максимального значения");
        }
    }
    
    #[test]
    fn test_random_number_statistics() {
        // Проверяем, что сумма всех элементов гистограммы равна количеству образцов
        let max_value = 10;
        let num_samples = 1000;
        let histogram = random_number_statistics(max_value, num_samples);
        
        assert_eq!(histogram.iter().sum::<usize>(), num_samples,
                  "Сумма всех элементов гистограммы должна быть равна количеству образцов");
    }
} 