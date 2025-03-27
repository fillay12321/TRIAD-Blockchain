use num_complex::Complex64;
use serde::{Serialize, Deserialize};
use std::ops::{Index, IndexMut};
use std::mem;
use std::fmt;
use rand;
use std::collections::HashMap;

/// Состояние отдельного кубита в виде суперпозиции |ψ⟩ = α|0⟩ + β|1⟩
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QubitState {
    /// Амплитуда состояния |0⟩
    pub alpha: Complex64,
    /// Амплитуда состояния |1⟩
    pub beta: Complex64,
}

impl QubitState {
    /// Создает новое состояние кубита |0⟩
    pub fn new() -> Self {
        Self {
            alpha: Complex64::new(1.0, 0.0),
            beta: Complex64::new(0.0, 0.0),
        }
    }
    
    /// Создает состояние кубита с заданными амплитудами
    pub fn with_amplitudes(alpha: Complex64, beta: Complex64) -> Self {
        // Нормализуем амплитуды
        let norm = (alpha.norm_sqr() + beta.norm_sqr()).sqrt();
        Self {
            alpha: alpha / norm,
            beta: beta / norm,
        }
    }
    
    /// Вероятность измерения состояния |0⟩
    pub fn prob_zero(&self) -> f64 {
        self.alpha.norm_sqr()
    }
    
    /// Вероятность измерения состояния |1⟩
    pub fn prob_one(&self) -> f64 {
        self.beta.norm_sqr()
    }
    
    /// Коллапсирует состояние в |0⟩
    pub fn collapse_to_zero(&mut self) {
        self.alpha = Complex64::new(1.0, 0.0);
        self.beta = Complex64::new(0.0, 0.0);
    }
    
    /// Коллапсирует состояние в |1⟩
    pub fn collapse_to_one(&mut self) {
        self.alpha = Complex64::new(0.0, 0.0);
        self.beta = Complex64::new(1.0, 0.0);
    }
    
    /// Сериализует состояние в компактное представление (16 байт)
    pub fn to_compact_bytes(&self) -> [u8; 16] {
        let mut result = [0u8; 16];
        
        // Копируем значения alpha и beta
        let alpha_re_bytes = self.alpha.re.to_le_bytes();
        let alpha_im_bytes = self.alpha.im.to_le_bytes();
        let beta_re_bytes = self.beta.re.to_le_bytes();
        let beta_im_bytes = self.beta.im.to_le_bytes();
        
        // Заполняем массив байт
        result[0..4].copy_from_slice(&alpha_re_bytes[0..4]);
        result[4..8].copy_from_slice(&alpha_im_bytes[0..4]);
        result[8..12].copy_from_slice(&beta_re_bytes[0..4]);
        result[12..16].copy_from_slice(&beta_im_bytes[0..4]);
        
        result
    }
    
    /// Восстанавливает состояние из компактного представления
    pub fn from_compact_bytes(bytes: &[u8; 16]) -> Self {
        let mut alpha_re_bytes = [0u8; 4];
        let mut alpha_im_bytes = [0u8; 4];
        let mut beta_re_bytes = [0u8; 4];
        let mut beta_im_bytes = [0u8; 4];
        
        // Копируем данные из компактного представления
        alpha_re_bytes.copy_from_slice(&bytes[0..4]);
        alpha_im_bytes.copy_from_slice(&bytes[4..8]);
        beta_re_bytes.copy_from_slice(&bytes[8..12]);
        beta_im_bytes.copy_from_slice(&bytes[12..16]);
        
        // Восстанавливаем числа с плавающей точкой
        let alpha_re = f32::from_le_bytes(alpha_re_bytes) as f64;
        let alpha_im = f32::from_le_bytes(alpha_im_bytes) as f64;
        let beta_re = f32::from_le_bytes(beta_re_bytes) as f64;
        let beta_im = f32::from_le_bytes(beta_im_bytes) as f64;
        
        // Создаем состояние
        Self {
            alpha: Complex64::new(alpha_re, alpha_im),
            beta: Complex64::new(beta_re, beta_im),
        }
    }
}

impl Default for QubitState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for QubitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4} + {:.4}i)|0⟩ + ({:.4} + {:.4}i)|1⟩",
               self.alpha.re, self.alpha.im, self.beta.re, self.beta.im)
    }
}

/// Представление квантового регистра для нескольких кубитов
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantumState {
    /// Количество кубитов в регистре
    pub qubit_count: usize,
    /// Вектор состояния (амплитуды для всех возможных базисных состояний)
    amplitudes: Vec<Complex64>,
    /// Флаг, указывающий на возможную запутанность между кубитами
    is_entangled: bool,
}

impl QuantumState {
    /// Создает новое квантовое состояние с указанным числом кубитов в состоянии |0...0⟩
    pub fn new(qubit_count: usize) -> Self {
        // Проверяем, что количество кубитов не превышает разумный предел
        // 30 кубитов дадут вектор размером 2^30 = 1 млрд элементов, что уже много
        if qubit_count > 30 {
            panic!("Слишком большое количество кубитов: {}. Максимально допустимое значение: 30", qubit_count);
        }
        
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); 1 << qubit_count];
        amplitudes[0] = Complex64::new(1.0, 0.0); // Состояние |0...0⟩
        
        Self {
            qubit_count,
            amplitudes,
            is_entangled: false,
        }
    }
    
    /// Создает состояние из набора индивидуальных состояний кубитов (без запутанности)
    pub fn from_qubit_states(states: &[QubitState]) -> Self {
        let qubit_count = states.len();
        let mut result = Self::new(qubit_count);
        
        // Инициализируем состояние как тензорное произведение отдельных кубитов
        for i in 0..(1 << qubit_count) {
            let mut amplitude = Complex64::new(1.0, 0.0);
            
            for q in 0..qubit_count {
                let bit = (i >> q) & 1;
                if bit == 0 {
                    amplitude *= states[q].alpha;
                } else {
                    amplitude *= states[q].beta;
                }
            }
            
            result.amplitudes[i] = amplitude;
        }
        
        result
    }
    
    /// Получает состояние отдельного кубита (аппроксимация, если есть запутанность)
    pub fn get_qubit_state(&self, qubit_idx: usize) -> QubitState {
        if qubit_idx >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        // Вычисляем маргинальные вероятности
        let mut prob_zero = 0.0;
        let mut prob_one = 0.0;
        
        for i in 0..self.amplitudes.len() {
            let amplitude = self.amplitudes[i].norm_sqr();
            if (i >> qubit_idx) & 1 == 0 {
                prob_zero += amplitude;
            } else {
                prob_one += amplitude;
            }
        }
        
        // Создаем аппроксимацию состояния кубита
        // Это не точное представление для запутанных состояний, но достаточно для многих задач
        let alpha = Complex64::new(prob_zero.sqrt(), 0.0);
        let beta = Complex64::new(prob_one.sqrt(), 0.0);
        
        QubitState { alpha, beta }
    }
    
    /// Устанавливает состояние отдельного кубита (снимает запутанность)
    pub fn set_qubit_state(&mut self, qubit_idx: usize, state: QubitState) {
        if qubit_idx >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        // Если регистр в запутанном состоянии, мы теряем запутанность
        if self.is_entangled {
            // Сохраняем текущее распределение вероятностей
            let old_probs: Vec<f64> = self.amplitudes.iter().map(|a| a.norm_sqr()).collect();
            
            // Создаем новое состояние
            let mut new_amplitudes = vec![Complex64::new(0.0, 0.0); self.amplitudes.len()];
            
            for i in 0..self.amplitudes.len() {
                let bit = (i >> qubit_idx) & 1;
                let amp = if bit == 0 { state.alpha } else { state.beta };
                
                // Распределяем амплитуду пропорционально старым вероятностям
                let base_idx = i & !(1 << qubit_idx);
                let alt_idx = i | (1 << qubit_idx);
                let base_prob = old_probs[base_idx];
                let alt_prob = old_probs[alt_idx];
                let total_prob = base_prob + alt_prob;
                
                if total_prob > 0.0 {
                    let scale = (old_probs[i] / total_prob).sqrt();
                    new_amplitudes[i] = amp * scale;
                }
            }
            
            // Нормализуем новое состояние
            let norm: f64 = new_amplitudes.iter().map(|a| a.norm_sqr()).sum::<f64>().sqrt();
            for a in &mut new_amplitudes {
                *a /= norm;
            }
            
            self.amplitudes = new_amplitudes;
        } else {
            // Для незапутанного состояния просто заменяем амплитуды
            for i in 0..self.amplitudes.len() {
                let bit = (i >> qubit_idx) & 1;
                let qubit_amp = if bit == 0 { state.alpha } else { state.beta };
                
                // Старая амплитуда кубита
                let old_amp = if bit == 0 { 
                    self.get_qubit_state(qubit_idx).alpha 
                } else { 
                    self.get_qubit_state(qubit_idx).beta 
                };
                
                // Заменяем старую амплитуду на новую
                if old_amp.norm() > 0.0 {
                    self.amplitudes[i] = self.amplitudes[i] * (qubit_amp / old_amp);
                } else if qubit_amp.norm() > 0.0 {
                    // Если старая амплитуда была нулевой, а новая нет
                    let mut new_idx = i;
                    if bit == 0 {
                        new_idx &= !(1 << qubit_idx);
                    } else {
                        new_idx |= 1 << qubit_idx;
                    }
                    self.amplitudes[i] = self.amplitudes[new_idx] * qubit_amp;
                }
            }
        }
    }
    
    /// Измеряет указанный кубит и возвращает результат (0 или 1)
    pub fn measure_qubit(&mut self, qubit_idx: usize) -> usize {
        if qubit_idx >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        // Вычисляем вероятность измерения состояния |1⟩
        let mut prob_one = 0.0;
        for i in 0..self.amplitudes.len() {
            if (i >> qubit_idx) & 1 == 1 {
                prob_one += self.amplitudes[i].norm_sqr();
            }
        }
        
        // Генерируем случайное число для вероятностного результата измерения
        let random_value: f64 = rand::random();
        let result = if random_value < prob_one { 1 } else { 0 };
        
        // Коллапсируем состояние согласно измерению
        let mut new_amplitudes = vec![Complex64::new(0.0, 0.0); self.amplitudes.len()];
        let mut norm_factor = 0.0;
        
        for i in 0..self.amplitudes.len() {
            let bit = (i >> qubit_idx) & 1;
            if bit == result {
                new_amplitudes[i] = self.amplitudes[i];
                norm_factor += self.amplitudes[i].norm_sqr();
            }
        }
        
        // Нормализуем новое состояние
        let norm = norm_factor.sqrt();
        for a in &mut new_amplitudes {
            *a /= norm;
        }
        
        self.amplitudes = new_amplitudes;
        self.is_entangled = true; // Измерение обычно создает запутанность
        
        result
    }
    
    /// Вычисляет полную запутанность системы
    pub fn calculate_entanglement(&self) -> f64 {
        if self.qubit_count <= 1 {
            return 0.0; // Один кубит не может быть запутан
        }
        
        let mut total_entanglement = 0.0;
        
        // Проверяем все пары кубитов
        for i in 0..self.qubit_count {
            for j in (i+1)..self.qubit_count {
                total_entanglement += self.calculate_pair_entanglement(i, j);
            }
        }
        
        // Нормализуем результат
        let pair_count = (self.qubit_count * (self.qubit_count - 1)) / 2;
        total_entanglement / pair_count as f64
    }
    
    /// Вычисляет запутанность между парой кубитов
    pub fn calculate_pair_entanglement(&self, qubit1: usize, qubit2: usize) -> f64 {
        if qubit1 >= self.qubit_count || qubit2 >= self.qubit_count {
            panic!("Индексы кубитов вне диапазона");
        }
        
        if qubit1 == qubit2 {
            return 0.0; // Кубит не может быть запутан сам с собой
        }
        
        // Создаем матрицу плотности для подсистемы из двух кубитов
        let mut rho = vec![vec![Complex64::new(0.0, 0.0); 4]; 4];
        
        // Заполняем матрицу плотности
        for i in 0..self.amplitudes.len() {
            for j in 0..self.amplitudes.len() {
                let amp_i = self.amplitudes[i];
                let amp_j = self.amplitudes[j].conj();
                let product = amp_i * amp_j;
                
                // Извлекаем биты для кубитов qubit1 и qubit2
                let i1 = (i >> qubit1) & 1;
                let i2 = (i >> qubit2) & 1;
                let j1 = (j >> qubit1) & 1;
                let j2 = (j >> qubit2) & 1;
                
                // Индексы для матрицы 4x4
                let row = (i1 << 1) | i2;
                let col = (j1 << 1) | j2;
                
                // Проверяем, совпадают ли остальные биты
                let mut match_other_bits = true;
                for k in 0..self.qubit_count {
                    if k != qubit1 && k != qubit2 {
                        if ((i >> k) & 1) != ((j >> k) & 1) {
                            match_other_bits = false;
                            break;
                        }
                    }
                }
                
                if match_other_bits {
                    rho[row][col] += product;
                }
            }
        }
        
        // Вычисляем частичный след для первого кубита
        let mut rho_b = vec![vec![Complex64::new(0.0, 0.0); 2]; 2];
        rho_b[0][0] = rho[0][0] + rho[1][1];
        rho_b[0][1] = rho[0][2] + rho[1][3];
        rho_b[1][0] = rho[2][0] + rho[3][1];
        rho_b[1][1] = rho[2][2] + rho[3][3];
        
        // Вычисляем энтропию фон Неймана для rho_b
        let p0 = rho_b[0][0].re;
        let p1 = rho_b[1][1].re;
        
        // Нормализуем вероятности
        let sum = p0 + p1;
        let p0 = if sum > 0.0 { p0 / sum } else { 0.5 };
        let p1 = if sum > 0.0 { p1 / sum } else { 0.5 };
        
        // Вычисляем энтропию
        let mut entropy = 0.0;
        if p0 > 0.0 {
            entropy -= p0 * p0.log2();
        }
        if p1 > 0.0 {
            entropy -= p1 * p1.log2();
        }
        
        // Энтропия 0 означает отсутствие запутанности, 1 - максимальная запутанность
        entropy
    }
    
    /// Сериализует состояние в компактное представление для передачи по сети
    pub fn to_delta_bytes(&self) -> Vec<u8> {
        // Для простой версии сериализуем только наиболее значимые амплитуды
        // В реальной реализации нужна более сложная схема сжатия
        
        let mut result = Vec::with_capacity(8 + 16 * self.qubit_count);
        
        // Добавляем число кубитов и флаг запутанности
        result.extend_from_slice(&(self.qubit_count as u32).to_le_bytes());
        result.push(if self.is_entangled { 1 } else { 0 });
        result.push(0); // Зарезервировано для будущего использования
        result.push(0); // Зарезервировано для будущего использования
        result.push(0); // Зарезервировано для будущего использования
        
        // Если система не запутана, можно передавать только состояния отдельных кубитов
        if !self.is_entangled {
            for i in 0..self.qubit_count {
                let state = self.get_qubit_state(i);
                result.extend_from_slice(&state.to_compact_bytes());
            }
        } else {
            // Для запутанных состояний передаем наиболее значимые амплитуды
            // В реальной реализации здесь нужно использовать продвинутые алгоритмы сжатия
            
            // Находим наиболее значимые амплитуды
            let mut indices_with_probs: Vec<(usize, f64)> = self.amplitudes
                .iter()
                .enumerate()
                .map(|(idx, amp)| (idx, amp.norm_sqr()))
                .collect();
            
            // Сортируем по убыванию вероятности
            indices_with_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            // Берем только top-K амплитуд
            let k = (self.qubit_count * 2).min(indices_with_probs.len());
            for i in 0..k {
                let (idx, _) = indices_with_probs[i];
                
                // Записываем индекс и амплитуду
                result.extend_from_slice(&(idx as u32).to_le_bytes());
                let amp = self.amplitudes[idx];
                
                // Записываем действительную и мнимую части с округлением
                let real = (amp.re * 1000.0).round() as i16;
                let imag = (amp.im * 1000.0).round() as i16;
                result.extend_from_slice(&real.to_le_bytes());
                result.extend_from_slice(&imag.to_le_bytes());
            }
        }
        
        result
    }
    
    /// Восстанавливает состояние из компактного представления
    pub fn from_delta_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 8 {
            panic!("Неверный формат данных дельты");
        }
        
        // Извлекаем заголовок
        let mut qubit_count_bytes = [0u8; 4];
        qubit_count_bytes.copy_from_slice(&bytes[0..4]);
        let qubit_count = u32::from_le_bytes(qubit_count_bytes) as usize;
        
        let is_entangled = bytes[4] != 0;
        
        // Создаем новое состояние
        let mut state = Self::new(qubit_count);
        state.is_entangled = is_entangled;
        
        if !is_entangled {
            // Восстанавливаем отдельные состояния кубитов
            let mut offset = 8;
            for i in 0..qubit_count {
                if offset + 16 <= bytes.len() {
                    let mut qubit_bytes = [0u8; 16];
                    qubit_bytes.copy_from_slice(&bytes[offset..offset+16]);
                    let qubit_state = QubitState::from_compact_bytes(&qubit_bytes);
                    state.set_qubit_state(i, qubit_state);
                    offset += 16;
                }
            }
        } else {
            // Восстанавливаем амплитуды для запутанных состояний
            // В реальной реализации здесь нужна более сложная схема декомпрессии
            
            // Сначала обнуляем все амплитуды
            state.amplitudes = vec![Complex64::new(0.0, 0.0); 1 << qubit_count];
            
            // Восстанавливаем переданные амплитуды
            let mut offset = 8;
            while offset + 8 <= bytes.len() {
                let mut idx_bytes = [0u8; 4];
                idx_bytes.copy_from_slice(&bytes[offset..offset+4]);
                let idx = u32::from_le_bytes(idx_bytes) as usize;
                offset += 4;
                
                if idx < state.amplitudes.len() && offset + 4 <= bytes.len() {
                    let mut real_bytes = [0u8; 2];
                    let mut imag_bytes = [0u8; 2];
                    real_bytes.copy_from_slice(&bytes[offset..offset+2]);
                    imag_bytes.copy_from_slice(&bytes[offset+2..offset+4]);
                    
                    let real = i16::from_le_bytes(real_bytes) as f64 / 1000.0;
                    let imag = i16::from_le_bytes(imag_bytes) as f64 / 1000.0;
                    
                    state.amplitudes[idx] = Complex64::new(real, imag);
                    offset += 4;
                }
            }
            
            // Нормализуем состояние
            let norm = state.amplitudes.iter().map(|a| a.norm_sqr()).sum::<f64>().sqrt();
            if norm > 1e-10 {  // Используем маленькое число вместо 0 для избежания деления на очень малое число
                for a in &mut state.amplitudes {
                    *a /= norm;
                }
            } else {
                // Если не удалось восстановить амплитуды, инициализируем |0...0⟩
                state.amplitudes[0] = Complex64::new(1.0, 0.0);
            }
        }
        
        state
    }

    /// Получение амплитуды для указанного базисного состояния
    pub fn get_amplitude(&self, index: usize) -> Complex64 {
        if index >= self.amplitudes.len() {
            panic!("Индекс базисного состояния вне диапазона");
        }
        self.amplitudes[index]
    }
    
    /// Получение всех амплитуд
    pub fn get_amplitudes(&self) -> &Vec<Complex64> {
        &self.amplitudes
    }
    
    /// Получение изменяемой ссылки на все амплитуды
    /// Используйте этот метод с осторожностью, только если нужна прямая модификация состояния
    pub fn get_amplitudes_mut(&mut self) -> &mut Vec<Complex64> {
        &mut self.amplitudes
    }
    
    /// Устанавливает новые амплитуды для квантового состояния
    pub fn set_amplitudes(&mut self, new_amplitudes: Vec<Complex64>) {
        assert_eq!(new_amplitudes.len(), 1 << self.qubit_count, 
                  "Размер вектора амплитуд должен быть 2^qubit_count");
        self.amplitudes = new_amplitudes;
    }
    
    /// Устанавливает амплитуды из HashMap
    pub fn set_amplitudes_from_hashmap(&mut self, amplitudes_map: HashMap<usize, Complex64>) {
        // Сохраняем исходный размер вектора
        let dim = 1 << self.qubit_count;
        
        // Заполняем вектор амплитуд нулями
        self.amplitudes = vec![Complex64::new(0.0, 0.0); dim];
        
        // Устанавливаем амплитуды из карты
        for (&idx, &amp) in &amplitudes_map {
            if idx < dim {
                self.amplitudes[idx] = amp;
            }
        }
        
        // Нормализуем состояние
        self.normalize();
    }
    
    /// Получение количества элементов в векторе амплитуд
    pub fn amplitudes_len(&self) -> usize {
        self.amplitudes.len()
    }
    
    /// Клонирование состояния
    pub fn clone(&self) -> Self {
        Self {
            qubit_count: self.qubit_count,
            amplitudes: self.amplitudes.clone(),
            is_entangled: self.is_entangled,
        }
    }

    /// Применяет однокубитовый гейт к указанному кубиту
    pub fn apply_single_qubit_gate(&mut self, target: usize, matrix: Vec<Complex64>) {
        if target >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        if matrix.len() != 4 {
            panic!("Неверный размер матрицы для однокубитового гейта");
        }
        
        // Разбиваем матрицу на элементы
        let m00 = matrix[0];
        let m01 = matrix[1];
        let m10 = matrix[2];
        let m11 = matrix[3];
        
        // Создаем копию амплитуд для обновления
        let old_amplitudes = self.amplitudes.clone();
        
        // Обновляем амплитуды
        for i in 0..old_amplitudes.len() {
            let bit_value = (i >> target) & 1;
            
            if bit_value == 0 {
                // Если target бит = 0
                let i1 = i | (1 << target); // Индекс с битом target = 1
                self.amplitudes[i] = old_amplitudes[i] * m00 + old_amplitudes[i1] * m01;
            } else {
                // Если target бит = 1
                let i0 = i & !(1 << target); // Индекс с битом target = 0
                self.amplitudes[i] = old_amplitudes[i0] * m10 + old_amplitudes[i] * m11;
            }
        }
        
        // Устанавливаем флаг запутанности, если гейт может создать запутанность
        if (m01 != Complex64::new(0.0, 0.0) || m10 != Complex64::new(0.0, 0.0)) && self.qubit_count > 1 {
            self.is_entangled = true;
        }
    }
    
    /// Применяет двухкубитовый гейт к паре кубитов
    pub fn apply_two_qubit_gate(&mut self, control: usize, target: usize, matrix: Vec<Complex64>) {
        if control >= self.qubit_count || target >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        if control == target {
            panic!("Управляющий и целевой кубиты должны быть разными");
        }
        
        if matrix.len() != 16 {
            panic!("Неверный размер матрицы для двухкубитового гейта");
        }
        
        // Преобразуем матрицу в двумерный массив для удобства
        let mut m = [[Complex64::new(0.0, 0.0); 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] = matrix[i * 4 + j];
            }
        }
        
        // Создаем копию амплитуд для обновления
        let old_amplitudes = self.amplitudes.clone();
        
        // Обновляем амплитуды
        for i in 0..old_amplitudes.len() {
            let control_bit = (i >> control) & 1;
            let target_bit = (i >> target) & 1;
            
            // Определяем базисное состояние (от 0 до 3) для пары кубитов
            let basis_state = (control_bit << 1) | target_bit;
            
            // Индексы для всех 4 возможных состояний кубитов (00, 01, 10, 11)
            let idx00 = i & !(1 << control) & !(1 << target);
            let idx01 = idx00 | (1 << target);
            let idx10 = idx00 | (1 << control);
            let idx11 = idx00 | (1 << control) | (1 << target);
            
            let indices = [idx00, idx01, idx10, idx11];
            
            // Рассчитываем новую амплитуду как сумму произведений
            let mut new_amp = Complex64::new(0.0, 0.0);
            for j in 0..4 {
                new_amp += old_amplitudes[indices[j]] * m[basis_state][j];
            }
            
            self.amplitudes[i] = new_amp;
        }
        
        // Двухкубитовые гейты обычно создают запутанность
        self.is_entangled = true;
    }
    
    /// Измеряет все кубиты и возвращает результат в виде целого числа
    pub fn measure_all(&mut self) -> usize {
        // Вычисляем вероятности всех базисных состояний
        let probs: Vec<f64> = self.amplitudes.iter().map(|a| a.norm_sqr()).collect();
        
        // Генерируем случайное число
        let mut rand_val: f64 = rand::random();
        
        // Выбираем состояние в соответствии с распределением вероятностей
        let mut result = 0;
        let mut cumulative_prob = 0.0;
        
        for (i, prob) in probs.iter().enumerate() {
            cumulative_prob += prob;
            if rand_val < cumulative_prob {
                result = i;
                break;
            }
        }
        
        // Коллапсируем состояние к выбранному базисному состоянию
        for i in 0..self.amplitudes.len() {
            self.amplitudes[i] = if i == result {
                Complex64::new(1.0, 0.0)
            } else {
                Complex64::new(0.0, 0.0)
            };
        }
        
        result
    }
    
    /// Получает вероятность измерения указанного базисного состояния
    pub fn get_probability(&self, basis_state: usize) -> f64 {
        if basis_state >= (1 << self.qubit_count) {
            return 0.0;
        }
        
        self.amplitudes[basis_state].norm_sqr()
    }

    /// Нормализует амплитуды квантового состояния
    pub fn normalize(&mut self) {
        let norm: f64 = self.amplitudes.iter().map(|a| a.norm_sqr()).sum::<f64>().sqrt();
        if norm > 0.0 {
            for a in &mut self.amplitudes {
                *a /= norm;
            }
        }
    }

    /// Вычисляет и возвращает состояние в виде строки
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        // Ограничиваем вывод для больших состояний
        let max_states = 16;
        let mut shown_states = 0;
        
        for i in 0..self.amplitudes.len() {
            let amp = self.amplitudes[i];
            
            // Пропускаем состояния с очень малыми амплитудами
            if amp.norm_sqr() > 1e-10 {
                if shown_states > 0 {
                    result.push_str(" + ");
                }
                
                result.push_str(&format!("({:.4} + {:.4}i)|{:b}>", 
                                        amp.re, amp.im, i));
                
                shown_states += 1;
                if shown_states >= max_states && i < self.amplitudes.len() - 1 {
                    result.push_str(" + ...");
                    break;
                }
            }
        }
        
        if result.is_empty() {
            result = "0".to_string();
        }
        
        result
    }

    /// Устанавливает значение амплитуды для указанного базисного состояния
    pub fn set_amplitude(&mut self, index: usize, value: Complex64) {
        if index >= self.amplitudes.len() {
            panic!("Индекс базисного состояния вне диапазона");
        }
        self.amplitudes[index] = value;
    }
    
    /// Получает вероятность измерения кубита в состоянии |1⟩
    pub fn get_probability_one(&self, qubit_idx: usize) -> f64 {
        if qubit_idx >= self.qubit_count {
            panic!("Индекс кубита вне диапазона");
        }
        
        let mut probability = 0.0;
        for i in 0..self.amplitudes.len() {
            if (i >> qubit_idx) & 1 == 1 {
                probability += self.amplitudes[i].norm_sqr();
            }
        }
        
        probability
    }
    
    /// Получает вероятность измерения указанного кубита в указанном состоянии (0 или 1)
    pub fn get_measurement_probability(&self, qubit_idx: usize, one_state: bool) -> f64 {
        if qubit_idx >= self.qubit_count {
            return 0.0;
        }
        
        let mut probability = 0.0;
        let n = 1 << self.qubit_count;
        let mask = 1 << qubit_idx;
        
        for i in 0..n {
            let bit_set = (i & mask) != 0;
            if bit_set == one_state {
                probability += self.amplitudes[i].norm_sqr();
            }
        }
        
        probability
    }

    /// Получает количество кубитов в состоянии
    pub fn qubit_count(&self) -> usize {
        self.qubit_count
    }
}

impl Index<usize> for QuantumState {
    type Output = Complex64;
    
    fn index(&self, idx: usize) -> &Self::Output {
        &self.amplitudes[idx]
    }
}

impl IndexMut<usize> for QuantumState {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.amplitudes[idx]
    }
}

/// Представление кубита как индивидуальной единицы в TRIAD
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Qubit {
    /// Текущее состояние кубита
    pub state: QubitState,
    /// Уникальный идентификатор кубита в сети TRIAD
    pub id: u64,
    /// Информация о запутанности с другими кубитами
    pub entanglement_info: Vec<u64>,
    /// Метаданные кубита
    pub metadata: String,
}

impl Qubit {
    /// Создает новый кубит с заданным идентификатором
    pub fn new(id: u64) -> Self {
        Self {
            state: QubitState::new(),
            id,
            entanglement_info: Vec::new(),
            metadata: String::new(),
        }
    }
    
    /// Применяет квантовый гейт к кубиту (для незапутанных состояний)
    pub fn apply_gate(&mut self, gate: &str) {
        match gate {
            "H" => { // Адамара
                let new_alpha = (self.state.alpha + self.state.beta) / Complex64::new(2.0_f64.sqrt(), 0.0);
                let new_beta = (self.state.alpha - self.state.beta) / Complex64::new(2.0_f64.sqrt(), 0.0);
                self.state.alpha = new_alpha;
                self.state.beta = new_beta;
            },
            "X" => { // Паули-X (NOT)
                std::mem::swap(&mut self.state.alpha, &mut self.state.beta);
            },
            "Z" => { // Паули-Z (фазовый сдвиг)
                self.state.beta = -self.state.beta;
            },
            "Y" => { // Паули-Y
                let temp = self.state.alpha;
                self.state.alpha = -Complex64::i() * self.state.beta;
                self.state.beta = Complex64::i() * temp;
            },
            _ => panic!("Неизвестный гейт: {}", gate),
        }
    }
    
    /// Вероятность измерения состояния |1⟩
    pub fn probability_one(&self) -> f64 {
        self.state.prob_one()
    }
    
    /// Вероятность измерения состояния |0⟩
    pub fn probability_zero(&self) -> f64 {
        self.state.prob_zero()
    }
    
    /// Измеряет кубит и возвращает результат (0 или 1)
    pub fn measure(&mut self) -> usize {
        let p1 = self.probability_one();
        let random_value: f64 = rand::random();
        
        if random_value < p1 {
            self.state.collapse_to_one();
            1
        } else {
            self.state.collapse_to_zero();
            0
        }
    }
    
    /// Добавляет запутанность с другим кубитом
    pub fn entangle_with(&mut self, other_id: u64) {
        if !self.entanglement_info.contains(&other_id) {
            self.entanglement_info.push(other_id);
        }
    }
    
    /// Снимает запутанность с другим кубитом
    pub fn disentangle_from(&mut self, other_id: u64) {
        self.entanglement_info.retain(|&id| id != other_id);
    }
}

impl fmt::Display for Qubit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Qubit[{}]: {}", self.id, self.state)
    }
} 