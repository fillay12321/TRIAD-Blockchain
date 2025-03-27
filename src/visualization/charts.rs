use plotters::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::path::Path;

const OUTPUT_DIR: &str = "visualizations";

/// Структура для десериализации данных сравнения консенсусов
#[derive(Debug, Deserialize)]
pub struct ConsensusComparison {
    pub node_counts: Vec<usize>,
    pub quantum_times: Vec<f64>,
    pub pow_times: Vec<f64>,
    pub pos_times: Vec<f64>,
    pub pow_energy_consumption: Vec<f64>,
    pub pow_block_propagation_time: Vec<f64>,
    pub pow_real_life_estimation: Vec<f64>,
}

/// Структура для результатов бенчмарка
#[derive(Debug, Deserialize)]
pub struct BenchmarkResult {
    // Конфигурация теста
    pub node_count: usize,
    pub qubits_per_node: usize,
    pub transaction_count: usize,
    pub topology: String,
    
    // Результаты производительности
    pub total_time_ms: f64,
    pub transactions_per_second: f64,
    pub avg_latency_ms: f64,
    pub max_latency_ms: f64,
    
    // Квантовые метрики
    pub avg_entanglement: f64,
    pub avg_interference: f64,
    pub consensus_rate: f64,
    
    // Ресурсные метрики
    pub memory_usage_mb: f64,
    pub relative_energy_consumption: f64,
    pub estimated_large_scale_tps: f64,
}

/// Загружает данные сравнения консенсусов из JSON файла
pub fn load_consensus_data(file_path: &str) -> Result<ConsensusComparison, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let data: ConsensusComparison = serde_json::from_str(&contents)?;
    Ok(data)
}

/// Создает директорию для вывода графиков, если она не существует
fn ensure_output_dir() -> Result<(), Box<dyn Error>> {
    let path = Path::new(OUTPUT_DIR);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Визуализирует сравнение времени обработки для разных механизмов консенсуса
pub fn visualize_consensus_time_comparison(data: &ConsensusComparison) -> Result<String, Box<dyn Error>> {
    ensure_output_dir()?;
    
    let output_path = format!("{}/consensus_time_comparison.png", OUTPUT_DIR);
    let path_clone = output_path.clone();
    let root = BitMapBackend::new(&path_clone, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let max_time = data.pow_times.iter()
        .chain(data.pos_times.iter())
        .chain(data.quantum_times.iter())
        .fold(0.0, |max, &val| if val > max { val } else { max });
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Сравнение скорости консенсуса", ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0..data.node_counts.len(),
            0.0..max_time * 1.1,
        )?;
    
    chart
        .configure_mesh()
        .x_labels(data.node_counts.len())
        .x_label_formatter(&|idx| {
            if *idx < data.node_counts.len() {
                format!("{}", data.node_counts[*idx])
            } else {
                "".to_string()
            }
        })
        .y_desc("Время (мс)")
        .x_desc("Количество узлов")
        .draw()?;
    
    // Линия для Quantum консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, data.quantum_times[i])),
        &BLUE,
    ))?
    .label("TRIAD Quantum")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Линия для PoW консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, data.pow_times[i])),
        &RED,
    ))?
    .label("Proof of Work")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    // Линия для PoS консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, data.pos_times[i])),
        &GREEN,
    ))?
    .label("Proof of Stake")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    println!("Сгенерирован график сравнения времени консенсуса: {}", output_path);
    Ok(output_path)
}

/// Визуализирует сравнение энергопотребления для разных механизмов консенсуса
pub fn visualize_energy_comparison(data: &ConsensusComparison) -> Result<String, Box<dyn Error>> {
    ensure_output_dir()?;
    
    let output_path = format!("{}/energy_comparison.png", OUTPUT_DIR);
    let path_clone = output_path.clone();
    let root = BitMapBackend::new(&path_clone, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Создаем оценочное энергопотребление для квантового консенсуса
    // (значительно меньше, чем у PoW, но больше чем у PoS)
    let quantum_energy: Vec<f64> = data.pow_energy_consumption.iter()
        .map(|&pow| pow * 0.01) // 1% от PoW
        .collect();
    
    // Создаем оценочное энергопотребление для PoS
    // (исторически в ~1000 раз эффективнее PoW)
    let pos_energy: Vec<f64> = data.pow_energy_consumption.iter()
        .map(|&pow| pow * 0.001) // 0.1% от PoW
        .collect();
    
    let max_energy = data.pow_energy_consumption.iter()
        .fold(0.0, |max, &val| if val > max { val } else { max });
    
    let min_energy = pos_energy.iter()
        .fold(max_energy, |min, &val| if val < min { val } else { min }) / 2.0;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Сравнение энергопотребления механизмов консенсуса", ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0..data.node_counts.len(),
            min_energy..max_energy * 1.1,
        )?;
    
    chart
        .configure_mesh()
        .x_labels(data.node_counts.len())
        .x_label_formatter(&|idx| {
            if *idx < data.node_counts.len() {
                format!("{}", data.node_counts[*idx])
            } else {
                "".to_string()
            }
        })
        .y_desc("Относительное энергопотребление (лог. шкала)")
        .y_label_formatter(&|v| format!("{:.1e}", v))
        .x_desc("Количество узлов")
        .draw()?;
    
    // Линия для Quantum консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, quantum_energy[i])),
        &BLUE,
    ))?
    .label("TRIAD Quantum")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Линия для PoW консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, data.pow_energy_consumption[i])),
        &RED,
    ))?
    .label("Proof of Work")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    // Линия для PoS консенсуса
    chart.draw_series(LineSeries::new(
        (0..data.node_counts.len()).map(|i| (i, pos_energy[i])),
        &GREEN,
    ))?
    .label("Proof of Stake")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    println!("Сгенерирован график сравнения энергопотребления: {}", output_path);
    Ok(output_path)
}

/// Визуализирует результаты бенчмарка производительности
pub fn visualize_performance_benchmark(results: &[BenchmarkResult]) -> Result<String, Box<dyn Error>> {
    ensure_output_dir()?;
    
    let output_path = format!("{}/performance_tps.png", OUTPUT_DIR);
    let path_clone = output_path.clone();
    let root = BitMapBackend::new(&path_clone, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let node_counts: Vec<usize> = results.iter().map(|r| r.node_count).collect();
    let tps_values: Vec<f64> = results.iter().map(|r| r.transactions_per_second).collect();
    
    let max_tps = tps_values.iter().fold(0.0, |max, &val| if val > max { val } else { max });
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Производительность TRIAD Network (TPS)", ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            *node_counts.iter().min().unwrap_or(&5) as f64..*node_counts.iter().max().unwrap_or(&100) as f64 * 1.1,
            0.0..max_tps * 1.1,
        )?;
    
    chart
        .configure_mesh()
        .y_desc("Транзакций в секунду (TPS)")
        .x_desc("Количество узлов")
        .draw()?;
    
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(tps_values.iter()).map(|(&n, &t)| (n as f64, t)),
        &BLUE,
    ))?;
    
    chart.draw_series(PointSeries::of_element(
        node_counts.iter().zip(tps_values.iter()).map(|(&n, &t)| (n as f64, t)),
        5,
        &BLUE,
        &|c, s, st| {
            EmptyElement::at(c)
                + Circle::new((0, 0), s, st.filled())
        },
    ))?;
    
    println!("Сгенерирован график производительности: {}", output_path);
    Ok(output_path)
}

/// Визуализирует сравнение латентности
pub fn visualize_latency_comparison(results: &[BenchmarkResult]) -> Result<String, Box<dyn Error>> {
    ensure_output_dir()?;
    
    let output_path = format!("{}/latency_comparison.png", OUTPUT_DIR);
    let path_clone = output_path.clone();
    let root = BitMapBackend::new(&path_clone, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let node_counts: Vec<usize> = results.iter().map(|r| r.node_count).collect();
    let avg_latency: Vec<f64> = results.iter().map(|r| r.avg_latency_ms).collect();
    let max_latency: Vec<f64> = results.iter().map(|r| r.max_latency_ms).collect();
    
    let max_value = max_latency.iter()
        .fold(0.0, |max, &val| if val > max { val } else { max });
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Латентность транзакций в TRIAD Network", ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            *node_counts.iter().min().unwrap_or(&5) as f64..*node_counts.iter().max().unwrap_or(&100) as f64 * 1.1,
            0.0..max_value * 1.1,
        )?;
    
    chart
        .configure_mesh()
        .y_desc("Латентность (мс)")
        .x_desc("Количество узлов")
        .draw()?;
    
    // Средняя латентность
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(avg_latency.iter()).map(|(&n, &l)| (n as f64, l)),
        &BLUE,
    ))?
    .label("Средняя латентность")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Максимальная латентность
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(max_latency.iter()).map(|(&n, &l)| (n as f64, l)),
        &RED,
    ))?
    .label("Максимальная латентность")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    println!("Сгенерирован график латентности: {}", output_path);
    Ok(output_path)
}

/// Визуализирует квантовые метрики
pub fn visualize_quantum_metrics(results: &[BenchmarkResult]) -> Result<String, Box<dyn Error>> {
    ensure_output_dir()?;
    
    let output_path = format!("{}/quantum_metrics.png", OUTPUT_DIR);
    let path_clone = output_path.clone();
    let root = BitMapBackend::new(&path_clone, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let node_counts: Vec<usize> = results.iter().map(|r| r.node_count).collect();
    let entanglement: Vec<f64> = results.iter().map(|r| r.avg_entanglement).collect();
    let interference: Vec<f64> = results.iter().map(|r| r.avg_interference).collect();
    let consensus_rate: Vec<f64> = results.iter().map(|r| r.consensus_rate).collect();
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Квантовые метрики TRIAD Network", ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            *node_counts.iter().min().unwrap_or(&5) as f64..*node_counts.iter().max().unwrap_or(&100) as f64 * 1.1,
            0.0..1.1,
        )?;
    
    chart
        .configure_mesh()
        .y_desc("Значение (0-1)")
        .x_desc("Количество узлов")
        .draw()?;
    
    // Запутанность
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(entanglement.iter()).map(|(&n, &e)| (n as f64, e)),
        &BLUE,
    ))?
    .label("Запутанность")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Интерференция
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(interference.iter()).map(|(&n, &i)| (n as f64, i)),
        &GREEN,
    ))?
    .label("Интерференция")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
    
    // Скорость консенсуса
    chart.draw_series(LineSeries::new(
        node_counts.iter().zip(consensus_rate.iter()).map(|(&n, &r)| (n as f64, r)),
        &RED,
    ))?
    .label("Скорость консенсуса")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    println!("Сгенерирован график квантовых метрик: {}", output_path);
    Ok(output_path)
} 