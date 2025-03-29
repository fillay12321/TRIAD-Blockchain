// Вспомогательные функции для библиотеки QuEST
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

// Макрос для отладочного вывода, который ничего не делает
#define DEBUG_PRINT(...) ((void)0)

// Генератор случайных чисел для квантовых измерений
double genrand_real1() {
    // Простой генератор случайных чисел от 0 до 1 (не криптографический!)
    static int initialized = 0;
    if (!initialized) {
        srand(time(NULL));
        initialized = 1;
        DEBUG_PRINT("INFO: Инициализирован генератор случайных чисел\n");
    }
    return (double)rand() / (double)RAND_MAX;
}

// Функция для инициализации генератора случайных чисел
void init_by_array(unsigned long init_key[], int key_length) {
    // Используем первое значение для инициализации
    srand(init_key[0]);
    DEBUG_PRINT("INFO: Генератор случайных чисел инициализирован с seed=%lu\n", init_key[0]);
}

// Функции валидации для QuEST
void validateControlTarget(int controlQubit, int targetQubit, int numQubits) {
    if (controlQubit >= numQubits || targetQubit >= numQubits || controlQubit == targetQubit) {
        // Вместо вывода сообщения об ошибке, просто завершаем программу
        exit(1);
    }
}

void validateTarget(int targetQubit, int numQubits) {
    if (targetQubit >= numQubits) {
        // Вместо вывода сообщения об ошибке, просто завершаем программу
        exit(1);
    }
}

void validateNumQubitsInQureg(int numQubits) {
    DEBUG_PRINT("DEBUG: Проверка числа кубитов: %d\n", numQubits);
    if (numQubits <= 0 || numQubits > 50) {
        // Вместо вывода сообщения об ошибке, просто завершаем программу
        exit(1);
    }
}

void validateMemoryAllocationSize(long long int numValues) {
    if (numValues <= 0) {
        // Вместо вывода сообщения об ошибке, просто завершаем программу
        exit(1);
    }
}

void validateQuregAllocation(void* qureg, int numQubits) {
    if (qureg == NULL) {
        // Вместо вывода сообщения об ошибке, просто завершаем программу
        exit(1);
    }
}

// Функция для обработки переполнения буфера QASM
void raiseQASMBufferOverflow() {
    // Вместо вывода сообщения об ошибке, просто завершаем программу
    exit(1);
}

// Информационная функция TRIAD
void triad_print_info() {
    DEBUG_PRINT("TRIAD: Использование реальной библиотеки QuEST\n");
} 