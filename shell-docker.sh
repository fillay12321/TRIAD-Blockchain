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

echo -e "${GREEN}Проверка, существует ли образ ${IMAGE_NAME}...${NC}"
if ! docker image inspect ${IMAGE_NAME} >/dev/null 2>&1; then
    echo -e "${YELLOW}Образ ${IMAGE_NAME} не найден. Сначала запустите run-docker.sh${NC}"
    exit 1
fi

echo -e "${GREEN}Запуск интерактивной оболочки в контейнере...${NC}"
docker run --name triad-shell -it --rm \
    -v "$(pwd):/app" \
    ${IMAGE_NAME} \
    bash

echo -e "${GREEN}Сеанс завершен${NC}" 