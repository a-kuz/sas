# Тестовый MD3 рендерер на WGPU

Независимый тестовый рендерер для MD3 моделей на wgpu с поддержкой вращения.

## Запуск

```bash
cd test_md3_standalone
cargo run --release
```

## Управление

- **Стрелки влево/вправо** - вращение по оси Y (yaw)
- **Стрелки вверх/вниз** - вращение по оси X (pitch)
- **Q/E** - вращение по оси Z (roll)
- **Пробел** - переключение автоматического вращения
- **R** - сброс вращения

## Требования

- Файл модели: `q3-resources/models/weapons2/machinegun/machinegun.md3`
- Файл текстуры: `q3-resources/models/weapons2/machinegun/machinegun.jpg`

