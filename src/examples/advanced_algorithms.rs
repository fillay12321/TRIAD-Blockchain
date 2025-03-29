//! Расширенные квантовые алгоритмы, демонстрирующие суперпозицию и запутанность.
//! 
//! Этот модуль содержит реализации различных квантовых алгоритмов, которые
//! наглядно демонстрируют принципы суперпозиции и квантовой запутанности.

use crate::api::QuantumEngine;
use crate::quest::QuESTSimulator;
use crate::core::quantum_simulator::{QuantumSimulator, AdvancedQuantumSimulator};
use crate::core::gates::*;
use crate::core::qubit::Qubit;
use std::f64::consts::PI;
use std::time::Instant;
use rand::Rng;
use std::fmt::Debug;

/// Реализует алгоритм квантовой телепортации произвольного состояния
/// Берёт квантовое состояние (theta, phi) и телепортирует его с первого кубита на третий
pub fn quantum_teleportation(state: (f64, f64)) -> bool {
    // Создаем квантовый движок с 3 кубитами
    let mut engine = QuantumEngine::new(3);
    
    // Подготавливаем состояние для телепортации
    let (theta, phi) = state;
    // Используем напрямую методы rx и ry симулятора
    {
        engine.simulator_mut().ry(0, 2.0 * theta);
        engine.simulator_mut().rz(0, phi);
    }
    
    // Создаем запутанную пару между кубитами 1 и 2
    engine.hadamard(1);
    engine.cnot(1, 2);
    
    // Запутываем кубит состояния с первым кубитом запутанной пары
    engine.cnot(0, 1);
    engine.hadamard(0);
    
    // Измеряем первые два кубита
    let m0 = engine.measure(0);
    let m1 = engine.measure(1);
    
    // Применяем нужные гейты в зависимости от результатов измерений
    if m1 {
        engine.x(2);
    }
    if m0 {
        engine.z(2);
    }
    
    // Проверяем успешность телепортации через статистическое измерение
    let mut success_count = 0;
    const TRIALS: usize = 100;
    
    for _ in 0..TRIALS {
        // Создаем новый симулятор для проверки
        let mut sim_copy = QuESTSimulator::new(1);
        
        // Подготавливаем исходное состояние на единственном кубите
        {
            sim_copy.ry(0, 2.0 * theta);
            sim_copy.rz(0, phi);
        }
        
        // Сравниваем с телепортированным состоянием
        let original_prob0 = sim_copy.probability_of_outcome(0, false);
        
        // Создаем новый движок для телепортации
        let mut teleport_engine = QuantumEngine::new(3);
        // Подготавливаем состояние для телепортации
        {
            teleport_engine.simulator_mut().ry(0, 2.0 * theta);
            teleport_engine.simulator_mut().rz(0, phi);
        }
        
        // Выполняем протокол телепортации
        teleport_engine.hadamard(1);
        teleport_engine.cnot(1, 2);
        teleport_engine.cnot(0, 1);
        teleport_engine.hadamard(0);
        
        let m0 = teleport_engine.measure(0);
        let m1 = teleport_engine.measure(1);
        
        if m1 {
            teleport_engine.x(2);
        }
        if m0 {
            teleport_engine.z(2);
        }
        
        // Измеряем вероятность в телепортированном состоянии
        let teleported_prob0 = teleport_engine.simulator().probability_of_outcome(2, false);
        
        // Сравниваем вероятности с небольшой погрешностью
        if (original_prob0 - teleported_prob0).abs() < 0.1 {
            success_count += 1;
        }
    }
    
    // Если успешность телепортации выше 90%, считаем протокол работающим
    let success_rate = success_count as f64 / TRIALS as f64;
    success_rate > 0.9
}

/// Реализует квантовое сверхплотное кодирование
/// (передача 2 классических битов через 1 квантовый бит)
pub fn superdense_coding(bits: (bool, bool)) -> (bool, bool) {
    // Создаем систему из 2 кубитов
    let mut engine = QuantumEngine::new(2);
    
    // Создаем состояние Белла между кубитами 0 и 1
    engine.create_bell_state(0, 1).unwrap();
    
    // Отправитель кодирует 2 бита, применяя одно из четырех преобразований к кубиту 0
    match bits {
        (false, false) => {}, // Ничего не делаем (I)
        (false, true) => { engine.x(0); }, // Применяем X
        (true, false) => { engine.z(0); }, // Применяем Z
        (true, true) => { engine.z(0); engine.x(0); }, // Применяем ZX
    }
    
    // Получатель декодирует сообщение
    // Сначала обратное преобразование Белла
    engine.cnot(0, 1);
    engine.hadamard(0);
    
    // Затем измерение обоих кубитов
    let bit1 = engine.measure(0);
    let bit2 = engine.measure(1);
    
    (bit1, bit2)
}

/// Демонстрация алгоритма Гровера для поиска в неструктурированных данных
pub fn grover_search(num_qubits: usize, target_state: u64) -> u64 {
    if num_qubits > 20 {
        panic!("Слишком много кубитов для симуляции");
    }
    
    // Создаём симулятор
    let mut engine = QuantumEngine::new(num_qubits);
    
    // Инициализируем равную суперпозицию всех состояний
    engine.create_uniform_superposition();
    
    // Определяем оптимальное число итераций
    let n = 1 << num_qubits;
    let iterations = (std::f64::consts::PI / 4.0 * (n as f64).sqrt()) as usize;
    
    // Итерации алгоритма Гровера
    for _ in 0..iterations {
        // Шаг 1: Фаза оракула (инвертируем целевое состояние)
        // Мы можем создать оракул, который помечает целевое состояние, инвертируя его фазу
        for i in 0..num_qubits {
            if (target_state >> i) & 1 == 0 {
                engine.x(i);
            }
        }
        
        // Применяем многокубитный Z-гейт (не напрямую доступен, поэтому используем обходной путь)
        // В реальной имплементации нужен контролируемый Z-гейт с N-1 контролями
        engine.z(num_qubits - 1);
        
        // Возвращаем состояние кубитов
        for i in 0..num_qubits {
            if (target_state >> i) & 1 == 0 {
                engine.x(i);
            }
        }
        
        // Шаг 2: Диффузия (инвертируем вокруг среднего)
        // Сначала Адамара на все кубиты
        for i in 0..num_qubits {
            engine.hadamard(i);
        }
        
        // Затем X на все кубиты
        for i in 0..num_qubits {
            engine.x(i);
        }
        
        // Применяем многокубитный Z-гейт (то же обходное решение)
        engine.z(num_qubits - 1);
        
        // Возвращаемся, применив X и Адамара на все кубиты
        for i in 0..num_qubits {
            engine.x(i);
        }
        
        for i in 0..num_qubits {
            engine.hadamard(i);
        }
    }
    
    // Измеряем все кубиты и конструируем результат
    let mut result = 0u64;
    for i in 0..num_qubits {
        if engine.measure(i) {
            result |= 1 << i;
        }
    }
    
    result
}

/// Демонстрирует различные продвинутые квантовые алгоритмы
pub fn demonstrate_advanced_algorithms() {
    println!("=== Демонстрация продвинутых квантовых алгоритмов ===\n");
    
    // Демонстрация квантовой телепортации
    println!("1. Квантовая телепортация:");
    let state_to_teleport = (0.8, 0.6); // Состояние для телепортации
    let success = quantum_teleportation(state_to_teleport);
    println!("   Состояние {:?} телепортировано успешно: {}\n", state_to_teleport, success);
    
    // Демонстрация сверхплотного кодирования
    println!("2. Сверхплотное кодирование:");
    for &bit1 in &[false, true] {
        for &bit2 in &[false, true] {
            let bits = (bit1, bit2);
            let result = superdense_coding(bits);
            println!("   Передано: {:?}, Получено: {:?}", bits, result);
        }
    }
    println!();
    
    // Демонстрация алгоритма Гровера
    println!("3. Алгоритм Гровера (поиск в неструктурированных данных):");
    let num_qubits = 3; // Используем 3 кубита для демонстрации
    let target = 5; // Ищем состояние |101⟩
    println!("   Поиск состояния |{:0width$b}⟩ в пространстве из {} состояний:", 
             target, 1 << num_qubits, width = num_qubits);
    
    let result = grover_search(num_qubits, target);
    println!("   Найдено состояние: |{:0width$b}⟩", result, width = num_qubits);
    println!("   Поиск успешен: {}\n", result == target);
    
    println!("=== Демонстрация завершена ===");
}

// ======== Алгоритм Шора ========

/// Функция для вычисления наибольшего общего делителя (GCD)
fn gcd_func(a: u64, b: u64) -> u64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Функция для вычисления модульного возведения в степень: a^b mod n
fn mod_pow(a: u64, b: u64, n: u64) -> u64 {
    let mut result = 1;
    let mut base = a % n;
    let mut exponent = b;
    
    while exponent > 0 {
        if exponent % 2 == 1 {
            result = (result * base) % n;
        }
        exponent >>= 1;
        base = (base * base) % n;
    }
    
    result
}

/// Функция для факторизации числа n с использованием алгоритма Шора
pub fn shor_factorize(n: u64) -> Option<(u64, u64)> {
    // Базовые проверки
    if n < 3 || n % 2 == 0 {
        return if n % 2 == 0 { Some((2, n / 2)) } else { None };
    }
    
    let mut rng = rand::thread_rng();
    
    // Даем шансов найти период
    for _ in 0..10 {
        // Выбираем случайное число a, такое что 1 < a < n
        let a = rng.gen_range(2..n);
        
        // Вычисляем НОД(a, n)
        let first_gcd = gcd_func(a, n);
        
        // Если НОД > 1, мы нашли нетривиальный делитель
        if first_gcd > 1 {
            return Some((first_gcd, n / first_gcd));
        }
        
        // Попытка найти период r
        // В реальном квантовом алгоритме Шора здесь был бы квантовый блок
        let mut found_r = None;
        for r in 2..1000 {
            if mod_pow(a, r, n) == 1 {
                found_r = Some(r);
                break;
            }
        }
        
        let r = match found_r {
            Some(r) => r,
            None => continue, // Не нашли период для этого a
        };
        
        // Если r нечетное, пробуем другое a
        if r % 2 != 0 {
            continue;
        }
        
        // Вычисляем потенциальные факторы
        let a_pow_r_div_2 = mod_pow(a, r / 2, n);
        let factor1 = gcd_func(a_pow_r_div_2 - 1, n);
        let factor2 = gcd_func(a_pow_r_div_2 + 1, n);
        
        if factor1 > 1 && factor1 < n {
            return Some((factor1, n / factor1));
        }
        
        if factor2 > 1 && factor2 < n {
            return Some((factor2, n / factor2));
        }
    }
    
    None
}

/// Демонстрация алгоритма Шора
pub fn demonstrate_shors_algorithm() {
    println!("Демонстрация алгоритма Шора:");
    
    let test_numbers = [15, 21, 33, 35, 39];
    
    for &n in &test_numbers {
        println!("\nПоиск множителей для числа: {}", n);
        
        let start = Instant::now();
        let result = shor_factorize(n);
        let elapsed = start.elapsed();
        
        if let Some((p, q)) = result {
            println!("Успех! {} = {} * {}", n, p, q);
            println!("Проверка: {} * {} = {}", p, q, p * q);
        } else {
            println!("Не удалось найти множители для {}", n);
        }
        
        println!("Время выполнения: {:?}", elapsed);
    }
}

// ======== Квантовое моделирование гамильтонианов ========

/// Структура для представления квантовой функции
pub struct PauliHamiltonian {
    terms: Vec<(f64, Vec<(usize, char)>)>,
}

impl PauliHamiltonian {
    /// Создает гамильтониан для молекулы H2 на заданном расстоянии
    pub fn h2_molecule(distance: f64) -> Self {
        // Упрощенная модель для H2
        // В реальности коэффициенты рассчитываются более сложно
        let g = 1.0 / distance;
        
        let mut terms = Vec::new();
        
        // Кинетическая энергия
        terms.push((0.5, vec![(0, 'X'), (1, 'X')]));
        terms.push((0.5, vec![(0, 'Y'), (1, 'Y')]));
        
        // Потенциальная энергия
        terms.push((g, vec![(0, 'Z')]));
        terms.push((g, vec![(1, 'Z')]));
        terms.push((-g, vec![(0, 'Z'), (1, 'Z')]));
        
        PauliHamiltonian { terms }
    }
    
    /// Вычисляет ожидаемое значение гамильтониана для данного квантового состояния
    pub fn expectation_value(&self, simulator: &QuESTSimulator) -> f64 {
        let mut energy = 0.0;
        
        for (coefficient, pauli_term) in &self.terms {
            let expectation = simulator.get_expectation_value(pauli_term);
            energy += coefficient * expectation;
        }
        
        energy
    }
}

/// Симулирует временную эволюцию системы с заданным гамильтонианом
pub fn simulate_time_evolution(hamiltonian: &PauliHamiltonian, time: f64, num_qubits: usize) -> QuESTSimulator {
    let mut simulator = QuESTSimulator::new(num_qubits);
    
    // Начинаем в суперпозиции
    simulator.reset();
    simulator.hadamard(0);
    simulator.hadamard(1);
    
    // Применяем операторы, соответствующие exp(-i*H*t)
    // В реальности используется алгоритм Троттера или другие методы
    // Здесь мы просто приближаем эволюцию для небольших времен
    
    for (coefficient, pauli_term) in &hamiltonian.terms {
        let angle = coefficient * time;
        
        for &(qubit, operator) in pauli_term {
            match operator {
                'X' => simulator.rx(qubit, angle),
                'Y' => simulator.ry(qubit, angle),
                'Z' => simulator.rz(qubit, angle),
                _ => panic!("Неизвестный оператор Паули: {}", operator),
            }
        }
    }
    
    simulator
}

/// Вычисляет энергию H2 молекулы как функцию межатомного расстояния
fn h2_energy_vs_distance(distances: &[f64]) -> Vec<f64> {
    let mut energies = Vec::with_capacity(distances.len());
    
    for &distance in distances {
        let hamiltonian = PauliHamiltonian::h2_molecule(distance);
        
        // Симулируем эволюцию небольшое время, чтобы приблизиться к основному состоянию
        let simulator = simulate_time_evolution(&hamiltonian, 1.0, 2);
        
        // Вычисляем энергию
        let energy = hamiltonian.expectation_value(&simulator);
        energies.push(energy);
    }
    
    energies
}

/// Демонстрация квантового моделирования молекулы H2
pub fn demonstrate_quantum_simulation() {
    println!("Демонстрация квантового моделирования молекулы H2:");
    
    // Расчет энергии как функции межатомного расстояния
    let distances: Vec<f64> = (5..=25).map(|i| i as f64 / 10.0).collect();
    let energies = h2_energy_vs_distance(&distances);
    
    println!("\nЗависимость энергии H2 от межатомного расстояния:");
    println!("-------------------------------------------------");
    println!("Расстояние (A) | Энергия (a.u.)");
    println!("-------------------------------------------------");
    
    let mut min_energy = f64::INFINITY;
    let mut optimal_distance = 0.0;
    
    for (i, &d) in distances.iter().enumerate() {
        println!("{:13.2} | {:14.6}", d, energies[i]);
        
        if energies[i] < min_energy {
            min_energy = energies[i];
            optimal_distance = d;
        }
    }
    
    println!("-------------------------------------------------");
    println!("Оптимальное межатомное расстояние: {:.2} A", optimal_distance);
    println!("Энергия в минимуме: {:.6} a.u.", min_energy);
}

// ======== Тесты ========

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_superdense_coding() {
        for &bit1 in &[false, true] {
            for &bit2 in &[false, true] {
                let bits = (bit1, bit2);
                let result = superdense_coding(bits);
                assert_eq!(bits, result, "Сверхплотное кодирование не работает для битов {:?}", bits);
            }
        }
    }
    
    #[test]
    fn test_grover_small_case() {
        // Тестируем на небольшом примере (2 кубита)
        let num_qubits = 2;
        let target = 3; // |11⟩
        let result = grover_search(num_qubits, target);
        assert_eq!(result, target, "Алгоритм Гровера не смог найти состояние |11⟩");
    }
    
    #[test]
    fn test_gcd() {
        assert_eq!(gcd_func(48, 18), 6);
        assert_eq!(gcd_func(17, 13), 1);
        assert_eq!(gcd_func(1, 1), 1);
        assert_eq!(gcd_func(0, 5), 5);
    }
    
    #[test]
    fn test_mod_exp() {
        assert_eq!(mod_pow(2, 10, 1000), 1024 % 1000);
        assert_eq!(mod_pow(7, 13, 15), 7);
        assert_eq!(mod_pow(2, 3, 5), 3);
    }
    
    #[test]
    fn test_h2_hamiltonian() {
        let hamiltonian = PauliHamiltonian::h2_molecule(1.0);
        assert!(!hamiltonian.terms.is_empty());
        
        // Создаем симулятор и вычисляем энергию
        let simulator = QuESTSimulator::new(2);
        let energy = hamiltonian.expectation_value(&simulator);
        
        // Проверяем, что энергия имеет разумное значение
        assert!(energy < 0.0);
    }
} 