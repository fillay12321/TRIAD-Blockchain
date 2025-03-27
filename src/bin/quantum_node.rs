// Запуск распределенного квантового узла TRIAD
//
// Этот исполняемый файл запускает квантовый узел TRIAD с поддержкой
// квантовой запутанности и телепортации состояний между компьютерами.
// 
// Использование:
//   cargo run --bin quantum_node -- --listen=127.0.0.1:3030 --nodes=10
//   cargo run --bin quantum_node -- --listen=127.0.0.1:3031 --peers=127.0.0.1:3030 --nodes=10

use std::error::Error;
use std::time::Duration;
use std::net::SocketAddr;
use tokio::signal;
use tokio::time::sleep;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use warp::http::Response;
use serde_json::json;

use triad::distributed::quantum_peer::{QuantumPeer, TriadQuantumPeer};
use triad::distributed::metrics::DistributedMetrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Инициализация логирования
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Ошибка установки глобального логгера");
    
    info!("Запуск распределенного квантового узла TRIAD");
    
    // Парсинг аргументов командной строки
    let args: Vec<String> = std::env::args().collect();
    
    // Адрес для прослушивания квантового узла
    let listen_addr = args.iter()
        .find(|a| a.starts_with("--listen="))
        .map(|s| s.trim_start_matches("--listen="))
        .unwrap_or("127.0.0.1:3030");
    
    // Порт для HTTP метрик
    let metrics_port = args.iter()
        .find(|a| a.starts_with("--metrics-port="))
        .map(|s| s.trim_start_matches("--metrics-port=").parse::<u16>().unwrap_or(3080))
        .unwrap_or(3080);
    
    // Список начальных пиров для подключения
    let bootstrap_peers: Vec<String> = args.iter()
        .find(|a| a.starts_with("--peers="))
        .map(|s| s.trim_start_matches("--peers=").split(',').map(String::from).collect())
        .unwrap_or_default();
    
    // Количество локальных квантовых узлов
    let node_count = args.iter()
        .find(|a| a.starts_with("--nodes="))
        .map(|s| s.trim_start_matches("--nodes=").parse::<usize>().unwrap_or(5))
        .unwrap_or(5);
    
    // Режим запуска бенчмарка
    let run_benchmark = args.iter().any(|a| a == "--benchmark");
    
    info!("Конфигурация узла:");
    info!("  Адрес: {}", listen_addr);
    info!("  Метрики HTTP порт: {}", metrics_port);
    info!("  Квантовых узлов: {}", node_count);
    info!("  Начальные пиры: {:?}", bootstrap_peers);
    info!("  Бенчмарк: {}", run_benchmark);
    
    // Создание квантового узла
    let mut quantum_peer = TriadQuantumPeer::new(node_count)?;
    
    // Запуск прослушивания соединений
    quantum_peer.start(listen_addr).await?;
    info!("Узел запущен и прослушивает соединения");
    
    // Подключение к начальным пирам и установление запутанности
    for peer_addr in bootstrap_peers {
        match quantum_peer.connect_and_entangle(&peer_addr).await {
            Ok(_) => info!("Установлена квантовая запутанность с {}", peer_addr),
            Err(e) => error!("Ошибка подключения к {}: {}", peer_addr, e),
        }
    }
    
    // Создаём разделяемый объект метрик для HTTP сервера
    let metrics_shared = Arc::new(RwLock::new(quantum_peer.get_metrics()));
    
    // Запуск мониторинга и отображения метрик
    let metrics_quantum_peer = quantum_peer.clone();
    let metrics_update = metrics_shared.clone();
    
    tokio::spawn(async move {
        loop {
            // Периодический вывод метрик
            sleep(Duration::from_secs(10)).await;
            
            let metrics = metrics_quantum_peer.get_metrics();
            info!("Метрики узла: {}", metrics.summary());
            
            // Обновляем разделяемые метрики для HTTP сервера
            let mut metrics_lock = metrics_update.write().await;
            *metrics_lock = metrics;
        }
    });
    
    // Запуск HTTP сервера для предоставления метрик
    let metrics_data = metrics_shared.clone();
    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(warp::any().map(move || metrics_data.clone()))
        .and_then(|metrics: Arc<RwLock<DistributedMetrics>>| async move {
            let metrics_data = metrics.read().await.clone();
            
            match serde_json::to_string(&metrics_data) {
                Ok(json) => Ok::<_, warp::Rejection>(Response::builder()
                    .header("Content-Type", "application/json")
                    .body(json)),
                Err(_) => Ok(Response::builder()
                    .status(500)
                    .body("Error serializing metrics".into())),
            }
        });
    
    // Добавляем маршрут для мониторинга работоспособности
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| {
            let health = json!({
                "status": "UP",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            warp::reply::json(&health)
        });
    
    // Комбинируем маршруты
    let routes = metrics_route.or(health_route);
    
    // Запускаем HTTP сервер в отдельном потоке
    let metrics_host = format!("0.0.0.0:{}", metrics_port);
    tokio::spawn(async move {
        info!("Запуск HTTP сервера для метрик на {}", metrics_host);
        warp::serve(routes)
            .run(metrics_host.parse::<SocketAddr>().unwrap())
            .await;
    });
    
    // Если включен режим бенчмарка, запускаем тестовые транзакции
    if run_benchmark {
        let mut benchmark_quantum_peer = quantum_peer.clone();
        
        tokio::spawn(async move {
            // Даем время на установление всех соединений
            sleep(Duration::from_secs(5)).await;
            
            info!("Запуск распределенного бенчмарка...");
            
            // Запускаем N транзакций с интервалом
            let transaction_count = 100;
            for i in 0..transaction_count {
                let tx_data = format!("test_transaction_{}", i);
                
                match benchmark_quantum_peer.propose_transaction(&tx_data).await {
                    Ok(result) => {
                        info!("Транзакция {}/{} обработана: {} мс, успех: {}", 
                             i + 1, transaction_count, result.latency_ms, result.success);
                    },
                    Err(e) => {
                        error!("Ошибка при обработке транзакции {}: {}", i, e);
                    }
                }
                
                // Пауза между транзакциями
                sleep(Duration::from_millis(500)).await;
            }
            
            info!("Распределенный бенчмарк завершен");
        });
    }
    
    // Ожидание сигнала завершения (Ctrl+C)
    info!("Нажмите Ctrl+C для завершения работы узла");
    signal::ctrl_c().await?;
    
    info!("Завершение работы квантового узла...");
    
    Ok(())
} 