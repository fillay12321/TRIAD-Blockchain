// TRIAD Network - Визуализатор результатов бенчмаркинга
//
// Этот исполняемый файл служит для:
// 1. Создания визуальных графиков на основе данных бенчмаркинга
// 2. Генерации HTML-панели с интерактивными отчетами
// 3. Мониторинга сети в реальном времени с визуальным отображением

use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

use triad::visualization::dashboard::generate_dashboard;
use triad::visualization::charts::{
    load_consensus_data,
    visualize_consensus_time_comparison,
    visualize_energy_comparison,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("TRIAD Network Визуализатор");
    println!("=========================");
    
    // Получаем аргументы командной строки
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { &args[1] } else { "all" };
    
    match command {
        "dashboard" => {
            // Генерируем полную панель управления
            let dashboard_path = generate_dashboard()?;
            open_dashboard(&dashboard_path)?;
        }
        "consensus" => {
            // Визуализируем только сравнение консенсуса
            if Path::new("consensus_comparison.json").exists() {
                let data = load_consensus_data("consensus_comparison.json")?;
                visualize_consensus_time_comparison(&data)?;
                visualize_energy_comparison(&data)?;
                println!("Графики сравнения консенсуса созданы в директории 'visualizations'");
            } else {
                println!("Ошибка: Файл consensus_comparison.json не найден");
                println!("Запустите 'cargo run --bin benchmark -- consensus' для генерации данных");
            }
        }
        "all" | _ => {
            // Генерируем все и открываем панель управления
            let dashboard_path = generate_dashboard()?;
            open_dashboard(&dashboard_path)?;
        }
    }
    
    Ok(())
}

/// Открывает панель управления в браузере по умолчанию
fn open_dashboard(path: &str) -> Result<(), Box<dyn Error>> {
    println!("Открытие панели управления в браузере: {}", path);
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", path])
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    
    Ok(())
} 