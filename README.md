# Система выдачи инструментов (Rust версия)

Легковесная версия системы учета выдачи инструментов на Rust + egui.

**Размер программы: ~6-10 МБ** (вместо 200 МБ Electron версии)

## Функционал

- **Авторизация**: admin/admin123 или storekeeper/store123
- **Панель администратора**: управление сотрудниками, инструментами, просмотр выдач
- **Терминал выдачи**: сканирование QR кодов, выдача/возврат инструментов
- **Статистика**: общее количество, выдано, доступно

## Сборка

### Linux

```bash
cargo build --release
```

Бинарник будет в `target/release/tool-issuance`

### Windows (требуется Windows)

```powershell
# Установите Rust: https://rustup.rs/
cargo build --release
```

Бинарник будет в `target/release/tool-issuance.exe`

### Кросс-компиляция из Linux в Windows

Требуется Docker:

```bash
cargo install cross
cross build --target x86_64-pc-windows-gnu --release
```

Или с mingw-w64:

```bash
# Ubuntu/Debian
sudo apt-get install mingw-w64

# Добавить в ~/.cargo/config.toml:
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"

cargo build --target x86_64-pc-windows-gnu --release
```

## Структура данных

Данные хранятся в файле `data/data.json` рядом с исполняемым файлом.

## QR коды

QR коды генерируются автоматически:
- Сотрудники: `EMP_xxxxxxxx`
- Инструменты: `TOOL_xxxxxxxx`

Для работы с QR кодами можно использовать любой сканер или создать QR коды на сайте https://www.qr-code-generator.com/

## Тестовые аккаунты

| Логин | Пароль | Роль |
|-------|--------|------|
| admin | admin123 | Администратор |
| storekeeper | store123 | Кладовщик |

## Технологии

- **Rust** - язык программирования
- **egui/eframe** - GUI фреймворк
- **serde/serde_json** - сериализация данных
- **chrono** - работа с датой/временем
- **uuid** - генерация идентификаторов
