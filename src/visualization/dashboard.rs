use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;

use super::charts::{
    ConsensusComparison, 
    BenchmarkResult,
    load_consensus_data,
    visualize_consensus_time_comparison,
    visualize_energy_comparison,
    visualize_performance_benchmark,
    visualize_latency_comparison,
    visualize_quantum_metrics,
};

const DASHBOARD_DIR: &str = "dashboard";
const BENCHMARK_RESULTS_FILE: &str = "benchmark_results.json";
const CONSENSUS_COMPARISON_FILE: &str = "consensus_comparison.json";
const NETWORK_MONITORING_FILE: &str = "network_monitoring.json";

/// Запускает создание всех графиков и собирает их в единую панель HTML
pub fn generate_dashboard() -> Result<String, Box<dyn Error>> {
    println!("\n🔄 Генерация интерактивной панели визуализации...");
    
    // Создаем директорию для панели, если она не существует
    let dashboard_path = Path::new(DASHBOARD_DIR);
    if !dashboard_path.exists() {
        fs::create_dir_all(dashboard_path)?;
    }
    
    // Получаем данные результатов бенчмаркинга
    let benchmark_results = load_benchmark_results()?;
    
    // Получаем данные сравнения консенсусов
    let consensus_data = if Path::new(CONSENSUS_COMPARISON_FILE).exists() {
        Some(load_consensus_data(CONSENSUS_COMPARISON_FILE)?)
    } else {
        println!("Файл сравнения консенсусов не найден, этот раздел будет пропущен");
        None
    };
    
    // Создаем графики
    let mut chart_paths = Vec::new();
    
    if !benchmark_results.is_empty() {
        chart_paths.push(visualize_performance_benchmark(&benchmark_results)?);
        chart_paths.push(visualize_latency_comparison(&benchmark_results)?);
        chart_paths.push(visualize_quantum_metrics(&benchmark_results)?);
    }
    
    if let Some(consensus_data) = consensus_data {
        chart_paths.push(visualize_consensus_time_comparison(&consensus_data)?);
        chart_paths.push(visualize_energy_comparison(&consensus_data)?);
    }
    
    // Создаем HTML страницу с рендером всех графиков
    let html_path = format!("{}/index.html", DASHBOARD_DIR);
    let html_content = generate_html_dashboard(&chart_paths, &benchmark_results);
    
    fs::write(&html_path, html_content)?;
    
    println!("✅ Панель визуализации создана: {}", html_path);
    Ok(html_path)
}

/// Загружает результаты бенчмаркинга из JSON файла
fn load_benchmark_results() -> Result<Vec<BenchmarkResult>, Box<dyn Error>> {
    if !Path::new(BENCHMARK_RESULTS_FILE).exists() {
        println!("Файл результатов бенчмаркинга не найден, будет использован тестовый набор данных");
        return Ok(generate_sample_benchmark_results());
    }
    
    let mut file = File::open(BENCHMARK_RESULTS_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let data: Vec<BenchmarkResult> = serde_json::from_str(&contents)?;
    Ok(data)
}

/// Генерирует тестовый набор данных бенчмаркинга для демонстрации
fn generate_sample_benchmark_results() -> Vec<BenchmarkResult> {
    let node_counts = vec![5, 10, 20, 50, 100];
    let mut results = Vec::new();
    
    for &node_count in &node_counts {
        let scalability_factor = 0.95f64.powf((node_count as f64).log10());
        let base_tps = 1000.0;
        
        results.push(BenchmarkResult {
            node_count,
            qubits_per_node: 3,
            transaction_count: 1000,
            topology: "FullMesh".to_string(),
            
            total_time_ms: 1000.0 * node_count as f64 / base_tps * scalability_factor,
            transactions_per_second: base_tps * scalability_factor,
            avg_latency_ms: 5.0 + node_count as f64 * 0.2,
            max_latency_ms: 10.0 + node_count as f64 * 0.5,
            
            avg_entanglement: 0.9 - 0.005 * node_count as f64,
            avg_interference: 0.85 - 0.003 * node_count as f64,
            consensus_rate: 0.98 - 0.0005 * node_count as f64,
            
            memory_usage_mb: node_count as f64 * 5.0,
            relative_energy_consumption: 0.001 + node_count as f64 * 0.0001,
            estimated_large_scale_tps: base_tps * 0.8 * (100_000.0 / node_count as f64).powf(0.9),
        });
    }
    
    results
}

/// Генерирует HTML страницу с панелью управления
fn generate_html_dashboard(chart_paths: &[String], results: &[BenchmarkResult]) -> String {
    let mut charts_html = String::new();
    
    for path in chart_paths {
        let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
        charts_html.push_str(&format!(
            r#"<div class="chart-container">
                <img src="../{}" alt="{}" class="chart-image">
            </div>
            "#,
            path, filename
        ));
    }
    
    let mut metrics_html = String::new();
    if !results.is_empty() {
        // Берем результаты для самого большого количества узлов
        let max_node_result = results.iter()
            .max_by_key(|r| r.node_count)
            .unwrap();
        
        metrics_html = format!(
            r#"<div class="metrics-table">
                <h3>Ключевые метрики</h3>
                <table>
                    <tr>
                        <th>Метрика</th>
                        <th>Значение</th>
                    </tr>
                    <tr>
                        <td>Количество узлов</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>Транзакций в секунду (TPS)</td>
                        <td>{:.2}</td>
                    </tr>
                    <tr>
                        <td>Средняя латентность (мс)</td>
                        <td>{:.2}</td>
                    </tr>
                    <tr>
                        <td>Запутанность</td>
                        <td>{:.2}</td>
                    </tr>
                    <tr>
                        <td>Консенсус</td>
                        <td>{:.1}%</td>
                    </tr>
                    <tr>
                        <td>Оценка TPS для 100K узлов</td>
                        <td>{:.2}</td>
                    </tr>
                </table>
            </div>"#,
            max_node_result.node_count,
            max_node_result.transactions_per_second,
            max_node_result.avg_latency_ms,
            max_node_result.avg_entanglement,
            max_node_result.consensus_rate * 100.0,
            max_node_result.estimated_large_scale_tps,
        );
    }
    
    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TRIAD Network - Аналитическая панель</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background-color: #f5f7fa;
            color: #333;
        }}
        header {{
            background: linear-gradient(135deg, #3a0ca3, #4361ee);
            color: white;
            padding: 20px;
            text-align: center;
        }}
        h1 {{
            margin: 0;
            font-size: 2.2em;
        }}
        .subtitle {{
            font-weight: 300;
            margin-top: 10px;
        }}
        .dashboard {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        .dashboard-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-top: 20px;
        }}
        @media (max-width: 768px) {{
            .dashboard-grid {{
                grid-template-columns: 1fr;
            }}
        }}
        .chart-container {{
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            padding: 15px;
            margin-bottom: 20px;
        }}
        .chart-image {{
            width: 100%;
            height: auto;
            border-radius: 4px;
        }}
        .metrics-container {{
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            padding: 20px;
            margin-bottom: 20px;
        }}
        .metrics-table {{
            width: 100%;
        }}
        .metrics-table table {{
            width: 100%;
            border-collapse: collapse;
        }}
        .metrics-table th, .metrics-table td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #eee;
        }}
        .metrics-table th {{
            background-color: #f9f9f9;
        }}
        footer {{
            text-align: center;
            padding: 20px;
            margin-top: 20px;
            color: #666;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <header>
        <h1>TRIAD Network</h1>
        <div class="subtitle">Аналитическая панель квантово-вдохновленной сети</div>
    </header>
    
    <div class="dashboard">
        <div class="metrics-container">
            {metrics_html}
        </div>
        
        <h2>Результаты бенчмаркинга</h2>
        <div class="dashboard-grid">
            {charts_html}
        </div>
    </div>
    
    <footer>
        <p>TRIAD Network © 2023 - Квантово-вдохновленная распределенная сеть</p>
    </footer>
</body>
</html>"#
    )
}

/// Создает бинарный исполняемый файл для запуска визуализации
pub fn create_visualizer_binary() -> Result<(), Box<dyn Error>> {
    // Имплементация создания исполняемого файла визуализатора
    // Этот функционал будет реализован позже
    Ok(())
} 