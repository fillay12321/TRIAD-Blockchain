# Используем Ubuntu 22.04 как базовый образ
FROM ubuntu:22.04

# Устанавливаем необходимые пакеты
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    curl \
    git \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Устанавливаем Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Создаем директорию для проекта
WORKDIR /app

# Копируем минимальный набор файлов для получения зависимостей
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Создаем пустую src для компиляции зависимостей
RUN mkdir -p src && \
    echo "fn main() {println!(\"placeholder\");}" > src/main.rs && \
    echo "pub mod core { pub mod quantum_simulator { pub trait QuantumSimulator {} } }" > src/lib.rs && \
    cargo build

# Удаляем временные файлы
RUN rm -rf src

# Копируем исходный код проекта
# Это будет перезаписано при запуске контейнера с монтированием,
# но нужно для построения образа
COPY . .

# Собираем проект
RUN cargo build

# Запускаем приложение по умолчанию
CMD ["cargo", "run", "--", "quest"] 