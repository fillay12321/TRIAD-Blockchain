extern crate cc;

use std::env;
use std::path::Path;
use std::process::Command;
use std::fs;
use std::io::Write;

fn main() {
    // Проверяем, должны ли мы использовать эмуляцию или настоящую библиотеку QuEST
    let use_emulation = env::var("TRIAD_USE_EMULATION").unwrap_or_else(|_| "true".to_string()) == "true";
    
    println!("cargo:rerun-if-env-changed=TRIAD_USE_EMULATION");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/QuEST_full");
    println!("cargo:rerun-if-changed=src/utils.c");
    
    // Определяем признак для кода Rust, чтобы он знал, какой режим используется
    if use_emulation {
        println!("cargo:rustc-cfg=feature=\"emulation\"");
        println!("Compiling with emulation mode (no QuEST library)");
        
        // Собираем только заглушки
        cc::Build::new()
            .file("src/utils.c")
            .define("TRIAD_USE_EMULATION", "1")
            .compile("utils");
    } else {
        println!("cargo:rustc-cfg=feature=\"quest\"");
        println!("Compiling with real QuEST library");
        
        // Проверяем, существует ли директория QuEST_full
        let quest_full_path = Path::new("vendor/QuEST_full");
        if !quest_full_path.exists() {
            panic!("Директория QuEST_full не найдена. Убедитесь, что она находится в vendor/QuEST_full.");
        }

        // Путь к QuEST внутри QuEST_full
        let quest_path = quest_full_path.join("QuEST");
        if !quest_path.exists() {
            panic!("Директория QuEST не найдена внутри QuEST_full. Проверьте структуру проекта.");
        }

        // Пути к исходным файлам QuEST
        let quest_src_path = quest_path.join("src");
        if !quest_src_path.exists() {
            panic!("Директория с исходным кодом QuEST не найдена по пути {:?}", quest_src_path);
        }

        // Путь к файлам CPU
        let quest_cpu_path = quest_src_path.join("CPU");
        if !quest_cpu_path.exists() {
            panic!("Директория CPU не найдена по пути {:?}", quest_cpu_path);
        }

        // Собираем список исходных файлов QuEST
        let quest_src_files = vec![
            quest_src_path.join("QuEST.c"),
            quest_src_path.join("QuEST_common.c"),
            quest_src_path.join("QuEST_qasm.c"),
        ];

        // Собираем список файлов CPU (исключая distributed файл, который требует MPI)
        let mut quest_cpu_files = Vec::new();
        for entry in fs::read_dir(&quest_cpu_path).expect("Не удалось прочитать директорию CPU") {
            let entry = entry.expect("Ошибка при чтении элемента директории");
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "c") {
                // Пропускаем файл QuEST_cpu_distributed.c, который требует MPI
                let file_name = path.file_name().unwrap().to_string_lossy();
                if !file_name.contains("distributed") {
                    quest_cpu_files.push(path);
                }
            }
        }

        // Указываем библиотеке cc собрать QuEST
        let mut build = cc::Build::new();
        
        // Добавляем пути к заголовочным файлам
        build.include(quest_path.join("include"));
        build.include(quest_src_path);
        
        // Указываем файлы для сборки
        for file in quest_src_files {
            build.file(file);
        }
        
        for file in quest_cpu_files {
            build.file(file);
        }
        
        // Добавляем нашу реализацию заглушек утилит (не заменяющих функции QuEST)
        build.file("src/utils.c")
            .define("TRIAD_USE_QUEST", "1");
        
        // Устанавливаем флаги компиляции
        build.define("QuEST_PREC", "2"); // Определяем двойную точность
        build.define("MULTITHREADED", "1"); // Включаем многопоточность
        build.define("QuEST_DISTRIBUTED", "0"); // Отключаем распределенные вычисления
        
        // Собираем библиотеку
        build.compile("quest");
        
        // Указываем cargo пересобирать при изменении исходников QuEST
        println!("cargo:rerun-if-changed=vendor/QuEST_full/QuEST");
    }
    
    // Путь к динамической библиотеке
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    
    if !use_emulation {
        println!("cargo:rustc-link-lib=static=quest");
    } else {
        println!("cargo:rustc-link-lib=static=utils");
    }
}

// Функция для создания файлов-заглушек, если не удалось скачать QuEST
fn create_stub_files(quest_path: &Path) {
    let include_path = quest_path.join("include");
    let src_path = quest_path.join("src");
    
    // Создаем директории
    fs::create_dir_all(&include_path).expect("Не удалось создать директорию include");
    fs::create_dir_all(&src_path).expect("Не удалось создать директорию src");
    
    // Создаем заглушки для основных заголовочных файлов
    let quest_h = include_path.join("QuEST.h");
    fs::write(quest_h, "// Заглушка для QuEST.h\n#include <stdlib.h>\n#include <math.h>\n\ntypedef struct QuESTEnv {\n    int rank;\n    int numRanks;\n} QuESTEnv;\n\ntypedef struct Qureg {\n    int numQubits;\n    int dummy;\n} Qureg;\n\ntypedef struct Complex {\n    double real;\n    double imag;\n} Complex;\n\nQuESTEnv createQuESTEnv();\nvoid destroyQuESTEnv(QuESTEnv env);\nQureg createQureg(int numQubits, QuESTEnv env);\nvoid destroyQureg(Qureg qureg, QuESTEnv env);\nvoid initZeroState(Qureg qureg);\nvoid initPlusState(Qureg qureg);\nvoid initClassicalState(Qureg qureg, long long int stateInd);\nint getNumQubits(Qureg qureg);\nvoid hadamard(Qureg qureg, const int targetQubit);\nvoid pauliX(Qureg qureg, const int targetQubit);\nvoid pauliY(Qureg qureg, const int targetQubit);\nvoid pauliZ(Qureg qureg, const int targetQubit);\nvoid sGate(Qureg qureg, const int targetQubit);\nvoid tGate(Qureg qureg, const int targetQubit);\nvoid controlledNot(Qureg qureg, const int controlQubit, const int targetQubit);\nvoid controlledPhaseFlip(Qureg qureg, const int idQubit1, const int idQubit2);\nint measure(Qureg qureg, int measureQubit);\nvoid rotateX(Qureg qureg, int rotQubit, double angle);\nvoid rotateY(Qureg qureg, int rotQubit, double angle);\nvoid rotateZ(Qureg qureg, int rotQubit, double angle);\ndouble getProbAmp(Qureg qureg, long long int index);\n").expect("Не удалось создать заглушку для QuEST.h");
    
    // Создаем заглушки для исходных файлов
    let quest_c = src_path.join("QuEST_stub.c");
    fs::write(quest_c, "// Заглушка для QuEST.c\n#include \"QuEST.h\"\n\nQuESTEnv createQuESTEnv() {\n    QuESTEnv env;\n    env.rank = 0;\n    env.numRanks = 1;\n    return env;\n}\n\nvoid destroyQuESTEnv(QuESTEnv env) {}\n\nQureg createQureg(int numQubits, QuESTEnv env) {\n    Qureg qureg;\n    qureg.numQubits = numQubits;\n    qureg.dummy = 0;\n    return qureg;\n}\n\nvoid destroyQureg(Qureg qureg, QuESTEnv env) {}\n\nvoid initZeroState(Qureg qureg) {}\n\nvoid initPlusState(Qureg qureg) {}\n\nvoid initClassicalState(Qureg qureg, long long int stateInd) {}\n\nint getNumQubits(Qureg qureg) {\n    return qureg.numQubits;\n}\n\nvoid hadamard(Qureg qureg, const int targetQubit) {}\n\nvoid pauliX(Qureg qureg, const int targetQubit) {}\n\nvoid pauliY(Qureg qureg, const int targetQubit) {}\n\nvoid pauliZ(Qureg qureg, const int targetQubit) {}\n\nvoid sGate(Qureg qureg, const int targetQubit) {}\n\nvoid tGate(Qureg qureg, const int targetQubit) {}\n\nvoid controlledNot(Qureg qureg, const int controlQubit, const int targetQubit) {}\n\nvoid controlledPhaseFlip(Qureg qureg, const int idQubit1, const int idQubit2) {}\n\nint measure(Qureg qureg, int measureQubit) {\n    return 0;\n}\n\nvoid rotateX(Qureg qureg, int rotQubit, double angle) {}\n\nvoid rotateY(Qureg qureg, int rotQubit, double angle) {}\n\nvoid rotateZ(Qureg qureg, int rotQubit, double angle) {}\n\ndouble getProbAmp(Qureg qureg, long long int index) {\n    return 0.0;\n}\n").expect("Не удалось создать заглушку для QuEST_stub.c");
} 