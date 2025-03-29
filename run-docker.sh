#!/bin/bash

# Цвета для вывода
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Проверяем, установлен ли Docker
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}Docker не установлен. Пожалуйста, установите Docker перед запуском этого скрипта.${NC}"
    echo "Инструкции по установке: https://docs.docker.com/get-docker/"
    exit 1
fi

# Имя образа
IMAGE_NAME="triad-quest"

echo -e "${GREEN}Шаг 1/3: Сборка Docker-образа ${IMAGE_NAME}...${NC}"
docker build -t ${IMAGE_NAME} .

echo -e "${GREEN}Шаг 2/3: Очистка предыдущих контейнеров...${NC}"
docker rm -f triad-container 2>/dev/null || true

echo -e "${GREEN}Шаг 3/3: Запуск контейнера с монтированием текущей директории...${NC}"
docker run --name triad-container -it --rm \
    -v "$(pwd):/app" \
    ${IMAGE_NAME} \
    bash -c "cargo run -- quest"

echo -e "${GREEN}Готово!${NC}" 