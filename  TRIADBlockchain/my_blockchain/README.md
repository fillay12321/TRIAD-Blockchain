<<<<<<< HEAD
# TRIAD-Blockchain
=======
TRIAD Blockchain

TRIAD Blockchain – это продвинутый блокчейн-проект, написанный на Rust. Проект разработан с упором на модульность, расширяемость и высокую производительность, обеспечивая:
- Многофункциональность консенсуса: реализация нескольких алгоритмов (PoW, PoS, DPoS, Tendermint, PoSpace) через плагин-систему.
- Кросс-чейн атомарные свопы: обеспечение безопасного обмена активами между сетями с использованием HTLC.
- P2P сеть и REST API: обмен данными между узлами и возможность интеграции с внешними сервисами.
- CI/CD: автоматическая сборка и тестирование с использованием GitHub Actions.

Особенности

- Модульная архитектура: 
  - Разделение функционала на отдельные модули: транзакции, блоки, кошельки, консенсус, atomic swap, внешние адаптеры, смарт‑контракты, mempool, P2P и REST API.
- Плагин-система для консенсуса:
  - Возможность комбинированной валидации блоков с использованием различных алгоритмов консенсуса.
- Кросс-чейн атомарные свопы:
  - Безопасный обмен активами между разными сетями.
- Интеграционные тесты и CI/CD:
  - Автоматическая сборка и тестирование через GitHub Actions.

 Структура проекта

TRIADBlockchain/ 
├── Cargo.toml 
├── README.md 
├── .github/ 
│ └── workflows/ 
│  └── ci.yml # CI/CD конфигурация GitHub Actions 
├── src/ 
│ ├── atomic_swap.rs # Реализация HTLC для атомарных свопов 
│ ├── block.rs # Определение блока, майнинг и валидация 
│ ├── consensus.rs # Алгоритмы консенсуса и интеграция плагинов 
│ ├── consensus_plugin.rs# Плагин-система для консенсусных алгоритмов 
│ ├── external_adapter.rs# Интерфейс для взаимодействия с внешними блокчейнами 
│ ├── lib.rs # Экспорт всех модулей проекта 
│ ├── main.rs # Основная логика приложения
│ ├── mempool.rs # Пул неподтверждённых транзакций 
│ ├── rest_api.rs # REST API сервер 
│ ├── smart_contract.rs # Интерфейс и менеджер смарт‑контрактов 
│ ├── token_economy.rs # Экономика токенов, инфляция и сжигание 
│ ├── transaction.rs # Определение транзакций и их методы 
| └── wallet.rs # Реализация кошельков и утилиты 
└── tests/ 
         └── blockchain_tests.rs # Интеграционные тесты для блокчейна



>>>>>>> 9baad7a (Create README.md)
