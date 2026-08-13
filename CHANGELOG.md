# Changelog

Журнал содержит фактически сделанные изменения проекта. Пункты не считаются
завершёнными только потому, что код написан: для проверенных изменений указаны
соответствующие проверки.

## [Unreleased]

Пока нет незакоммиченных изменений, относящихся к проекту.

## 2026-08-14

### Добавлено

- Автоматический integration smoke-тест `client/run_integration.sh`. Он
  проверяет health API, если передан `ADMIN_HEALTH_URL`, и затем выполняет
  реальный TCP/mTLS сценарий EPP через `client/epp_smoke.py`: greeting, login,
  hello, logout. Проверено на VPS.
- Тесты EPP framing для EOF в заголовке и теле кадра.
- Ограниченный graceful shutdown EPP-сервера с настраиваемым периодом ожидания
  `EPP_SHUTDOWN_GRACE_PERIOD` (по умолчанию 10 секунд).

### Проверено

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` — 20 тестов успешно
- Реальный EPP smoke через VPS: greeting, login `1000`, hello, logout `1000`

## История до ведения changelog

Изменения ниже уже были реализованы до появления этого файла:

- `5979512` — сохранение ответа `hello` в журнале EPP-транзакций.
- `96b1cbc` — строгая проверка EPP XML namespace.
- `4c2abca` — проверка service negotiation при login.
- `9050650` — покрытие полного EPP smoke-сценария.
- `5d36f31` — dev helper для создания registrar.
- `70a74a5` — локальный Python EPP smoke-клиент.
- `2ae56ed` — bootstrap backend, PostgreSQL, TLS/mTLS, EPP framing,
  session lifecycle, Admin API и начальные миграции.

