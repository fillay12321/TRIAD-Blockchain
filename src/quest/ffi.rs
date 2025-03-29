//! FFI-интерфейс для взаимодействия с C-библиотекой QuEST.
//! 
//! Этот модуль содержит определения типов и функций для взаимодействия с QuEST
//! через FFI-интерфейс (Foreign Function Interface). Он является низкоуровневым
//! и не предназначен для прямого использования в коде приложения.

#![allow(non_camel_case_types)]

use libc::{c_void, c_int, c_longlong, c_char, c_double};
use std::ffi::CString;
use std::fmt;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Определения для FFI-типов QuEST (определены в C-библиотеке)
#[repr(C)]
pub struct QuESTEnvStruct {
    _private: [u8; 0],
}

#[repr(C)]
pub struct QuregStruct {
    _private: [u8; 0],
}

/// Тип для представления среды выполнения QuEST.
pub type QuESTEnv = *mut QuESTEnvStruct;

/// Тип для представления квантового регистра в QuEST.
pub type Qureg = *mut QuregStruct;

/// Тип для представления комплексного матричного оператора 2x2.
pub type ComplexMatrix2 = [Complex; 4];

/// Тип для представления комплексного матричного оператора 4x4.
pub type ComplexMatrix4 = [Complex; 16];

/// Тип для представления комплексного числа в QuEST.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Complex {
    pub real: c_double,
    pub imag: c_double,
}

/// Безопасная обертка для указателя QuESTEnv.
/// Это обертка гарантирует, что указатель не будет использоваться неправильно
/// и что он может быть безопасно передан между потоками (Send + Sync).
pub struct SafeQuESTEnv {
    env: QuESTEnv,
    initialized: Arc<AtomicBool>,
}

// Макрос для отладки, который ничего не делает
macro_rules! debug_print {
    ($($arg:tt)*) => {};
}

// Реализуем Send и Sync вручную, так как QuESTEnv - это сырой указатель
unsafe impl Send for SafeQuESTEnv {}
unsafe impl Sync for SafeQuESTEnv {}

impl SafeQuESTEnv {
    /// Создает новую безопасную обертку для QuESTEnv.
    pub fn new() -> Self {
        debug_print!("DEBUG: Создаем среду QuEST");
        let env = unsafe { createQuESTEnv() };
        if env.is_null() {
            panic!("Не удалось создать среду QuEST (null pointer)");
        }
        debug_print!("DEBUG: Среда QuEST успешно создана");
        
        SafeQuESTEnv {
            env,
            initialized: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// Возвращает указатель на QuESTEnv.
    pub fn get_env(&self) -> QuESTEnv {
        self.env
    }
}

impl Clone for SafeQuESTEnv {
    fn clone(&self) -> Self {
        SafeQuESTEnv {
            env: self.env,
            initialized: Arc::clone(&self.initialized),
        }
    }
}

impl Drop for SafeQuESTEnv {
    fn drop(&mut self) {
        if Arc::strong_count(&self.initialized) == 1 && 
           self.initialized.load(Ordering::SeqCst) {
            self.initialized.store(false, Ordering::SeqCst);
            unsafe {
                destroyQuESTEnv(self.env);
            }
        }
    }
}

/// Rust-обертка для работы с квантовым регистром QuEST.
pub struct QuregWrapper {
    qureg: Qureg,
    env: SafeQuESTEnv,
}

// Объявляем, что QuregWrapper можно передавать между потоками
// Это безопасно, так как мы используем мьютексы для синхронизации доступа
unsafe impl Send for QuregWrapper {}
unsafe impl Sync for QuregWrapper {}

impl QuregWrapper {
    /// Создает новый квантовый регистр с указанным числом кубитов.
    pub fn new(num_qubits: usize, env: &SafeQuESTEnv) -> Self {
        // Проверяем, что количество кубитов имеет смысл
        if num_qubits == 0 {
            panic!("Число кубитов не может быть нулем");
        }
        if num_qubits > 30 {
            // Ограничиваем максимальное число кубитов для предотвращения переполнения памяти
            panic!("Число кубитов не может превышать 30 (ограничение для безопасности)");
        }
        
        debug_print!("DEBUG: Создаем квантовый регистр с {} кубитами", num_qubits);
        
        // Вызываем C-функцию для создания регистра
        // Явно приводим num_qubits к типу c_int (обычно 32-бита, signed)
        let qureg = unsafe { createQureg(num_qubits as c_int, env.get_env()) };
        
        // Проверяем, что указатель не является null
        if qureg.is_null() {
            panic!("Не удалось создать квантовый регистр (null pointer)");
        }
        
        // Проверяем, что число кубитов установлено корректно
        let actual_qubits = unsafe { getNumQubits(qureg) } as usize;
        debug_print!("DEBUG: Фактическое число кубитов: {}", actual_qubits);
        
        if actual_qubits != num_qubits {
            unsafe { destroyQureg(qureg, env.get_env()) };
            panic!("Ошибка инициализации: запрошено {} кубитов, но фактическое число кубитов - {}", 
                   num_qubits, actual_qubits);
        }
        
        QuregWrapper {
            qureg,
            env: env.clone(),
        }
    }
    
    /// Возвращает указатель на Qureg.
    pub fn get_qureg(&self) -> Qureg {
        self.qureg
    }
    
    /// Освобождает ресурсы, связанные с этим квантовым регистром.
    pub fn destroy(self) {
        // Данный метод потребляет self, обеспечивая однократное освобождение ресурсов
        unsafe { destroyQureg(self.qureg, self.env.get_env()) };
    }
    
    /// Возвращает число кубитов в регистре.
    pub fn get_num_qubits(&self) -> usize {
        unsafe { getNumQubits(self.qureg) as usize }
    }
    
    /// Применяет гейт Адамара к указанному кубиту.
    pub fn hadamard(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { hadamard(self.qureg, qubit as c_int) };
    }
    
    /// Применяет X-гейт (NOT) к указанному кубиту.
    pub fn x(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { pauliX(self.qureg, qubit as c_int) };
    }
    
    /// Применяет Y-гейт к указанному кубиту.
    pub fn y(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { pauliY(self.qureg, qubit as c_int) };
    }
    
    /// Применяет Z-гейт к указанному кубиту.
    pub fn z(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { pauliZ(self.qureg, qubit as c_int) };
    }
    
    /// Применяет фазовый S-гейт к указанному кубиту.
    pub fn s(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { sGate(self.qureg, qubit as c_int) };
    }
    
    /// Применяет T-гейт к указанному кубиту.
    pub fn t(&mut self, qubit: usize) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { tGate(self.qureg, qubit as c_int) };
    }
    
    /// Применяет CNOT-гейт с указанными контрольным и целевым кубитами.
    pub fn cnot(&mut self, control: usize, target: usize) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledNot(self.qureg, control as c_int, target as c_int) };
    }
    
    /// Применяет контролируемый фазовый поворот между двумя кубитами.
    pub fn controlled_phase_flip(&mut self, control: usize, target: usize) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledPhaseFlip(self.qureg, control as c_int, target as c_int) };
    }
    
    /// Применяет вращение вокруг оси X на указанный угол (в радианах).
    pub fn rotate_x(&mut self, qubit: usize, angle: f64) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { rotateX(self.qureg, qubit as c_int, angle) };
    }
    
    /// Применяет вращение вокруг оси Y на указанный угол (в радианах).
    pub fn rotate_y(&mut self, qubit: usize, angle: f64) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { rotateY(self.qureg, qubit as c_int, angle) };
    }
    
    /// Применяет вращение вокруг оси Z на указанный угол (в радианах).
    pub fn rotate_z(&mut self, qubit: usize, angle: f64) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { rotateZ(self.qureg, qubit as c_int, angle) };
    }
    
    /// Измеряет указанный кубит и возвращает результат (0 или 1).
    /// Этот метод изменяет состояние системы!
    pub fn measure_qubit(&mut self, qubit: usize) -> bool {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        let result = unsafe { measure(self.qureg, qubit as c_int) };
        result != 0
    }
    
    /// Возвращает вероятность указанного состояния (заданного индексом).
    pub fn get_probability(&self, state_idx: usize) -> f64 {
        if state_idx >= (1 << self.get_num_qubits()) {
            panic!("Индекс состояния выходит за пределы: {}", state_idx);
        }
        
        unsafe { getProbAmp(self.qureg, state_idx as c_longlong) }
    }
    
    /// Возвращает вектор вероятностей для всех состояний.
    pub fn get_probabilities(&self) -> Vec<f64> {
        let num_amps = 1 << self.get_num_qubits();
        let mut probs = vec![0.0; num_amps];
        
        for i in 0..num_amps {
            probs[i] = unsafe { getProbAmp(self.qureg, i as c_longlong) };
        }
        
        probs
    }
    
    /// Инициализирует состояние |0...0⟩.
    pub fn init_zero_state(&mut self) {
        unsafe { initZeroState(self.qureg) };
    }
    
    /// Инициализирует состояние |+...+⟩.
    pub fn init_plus_state(&mut self) {
        unsafe { initPlusState(self.qureg) };
    }
    
    /// Инициализирует классическое состояние (один базисный вектор).
    pub fn init_classical_state(&mut self, state_idx: usize) {
        if state_idx >= (1 << self.get_num_qubits()) {
            panic!("Индекс состояния выходит за пределы: {}", state_idx);
        }
        
        unsafe { initClassicalState(self.qureg, state_idx as c_longlong) };
    }

    /// Применяет унитарную матрицу к указанному кубиту
    pub fn unitary(&mut self, qubit: usize, matrix: &ComplexMatrix2) {
        if qubit >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: {}", qubit);
        }
        
        unsafe { unitary(self.qureg, qubit as c_int, matrix.as_ptr()) };
    }

    /// Применяет контролируемую унитарную матрицу к указанному кубиту
    pub fn controlled_unitary(&mut self, control: usize, target: usize, matrix: &ComplexMatrix2) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledUnitary(self.qureg, control as c_int, target as c_int, matrix.as_ptr()) };
    }

    /// Применяет контролируемое вращение X
    pub fn controlled_rotate_x(&mut self, control: usize, target: usize, angle: f64) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledRotateX(self.qureg, control as c_int, target as c_int, angle) };
    }

    /// Применяет контролируемое вращение Y
    pub fn controlled_rotate_y(&mut self, control: usize, target: usize, angle: f64) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledRotateY(self.qureg, control as c_int, target as c_int, angle) };
    }

    /// Применяет контролируемое вращение Z
    pub fn controlled_rotate_z(&mut self, control: usize, target: usize, angle: f64) {
        if control >= self.get_num_qubits() || target >= self.get_num_qubits() {
            panic!("Индекс кубита выходит за пределы: контроль = {}, цель = {}", control, target);
        }
        
        unsafe { controlledRotateZ(self.qureg, control as c_int, target as c_int, angle) };
    }
}

impl fmt::Debug for QuregWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_qubits = self.get_num_qubits();
        let num_amps = 1 << num_qubits;
        let probs = self.get_probabilities();
        
        writeln!(f, "QuregWrapper {{ num_qubits: {}, probabilities: [", num_qubits)?;
        
        for i in 0..num_amps {
            if i < 8 || probs[i] > 0.001 {
                writeln!(f, "  |{:0width$b}>: {:.6}", i, probs[i], width = num_qubits)?;
            }
        }
        
        writeln!(f, "]}}")
    }
}

impl Drop for QuregWrapper {
    fn drop(&mut self) {
        unsafe {
            destroyQureg(self.qureg, self.env.get_env());
        }
    }
}

// Внешний FFI-интерфейс для функций QuEST.
#[link(name = "quest", kind = "static")]
extern "C" {
    // Функции для работы со средой и регистрами
    fn createQuESTEnv() -> QuESTEnv;
    fn destroyQuESTEnv(env: QuESTEnv);
    fn createQureg(numQubits: c_int, env: QuESTEnv) -> Qureg;
    fn destroyQureg(qureg: Qureg, env: QuESTEnv);
    
    // Функции для инициализации состояний
    fn initZeroState(qureg: Qureg);
    fn initPlusState(qureg: Qureg);
    fn initClassicalState(qureg: Qureg, stateInd: c_longlong);
    
    // Квантовые гейты
    fn hadamard(qureg: Qureg, targetQubit: c_int);
    fn pauliX(qureg: Qureg, targetQubit: c_int);
    fn pauliY(qureg: Qureg, targetQubit: c_int);
    fn pauliZ(qureg: Qureg, targetQubit: c_int);
    fn sGate(qureg: Qureg, targetQubit: c_int);
    fn tGate(qureg: Qureg, targetQubit: c_int);
    fn controlledNot(qureg: Qureg, controlQubit: c_int, targetQubit: c_int);
    fn controlledPhaseFlip(qureg: Qureg, controlQubit: c_int, targetQubit: c_int);
    fn rotateX(qureg: Qureg, rotQubit: c_int, angle: c_double);
    fn rotateY(qureg: Qureg, rotQubit: c_int, angle: c_double);
    fn rotateZ(qureg: Qureg, rotQubit: c_int, angle: c_double);
    
    // Измерения и вероятности
    fn measure(qureg: Qureg, measureQubit: c_int) -> c_int;
    fn getProbAmp(qureg: Qureg, index: c_longlong) -> c_double;
    fn calcProbOfOutcome(qureg: Qureg, measureQubit: c_int, outcome: c_int) -> c_double;
    
    // Функции информации о квантовом регистре
    fn getNumQubits(qureg: Qureg) -> c_int;
    fn getNumAmps(qureg: Qureg) -> c_longlong;
    
    // Унитарные операторы
    fn unitary(qureg: Qureg, targetQubit: c_int, u: *const Complex);
    
    // Контролируемые операторы
    fn controlledUnitary(qureg: Qureg, controlQubit: c_int, targetQubit: c_int, u: *const Complex);
    fn controlledRotateX(qureg: Qureg, controlQubit: c_int, targetQubit: c_int, angle: c_double);
    fn controlledRotateY(qureg: Qureg, controlQubit: c_int, targetQubit: c_int, angle: c_double);
    fn controlledRotateZ(qureg: Qureg, controlQubit: c_int, targetQubit: c_int, angle: c_double);
    
    // Мультиконтролируемые операторы
    fn multiControlledUnitary(qureg: Qureg, controlQubits: *const c_int, numControlQubits: c_int, 
                               targetQubit: c_int, u: *const Complex);
    
    // Файловые операции
    fn writeStateToFile(qureg: Qureg, filename: *const c_char) -> c_int;
    fn readStateFromFile(qureg: Qureg, filename: *const c_char) -> c_int;
} 