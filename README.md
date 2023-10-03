# test-api

https://test-api.medigor.ru

Http api для тестирования. Доступные эндпойнты см. в коде или примеры в [test.http](test.http)

Сборка:
```bash
podman build --tag test-api-rs .
```
Запуск:
```bash
podman run -d -p 8080:8080 --name test-api-rs localhost/test-api-rs:latest
```
В [Dockerfile](Dockerfile) указаны полные имена образов, чтобы сборка работала с **podman**, но **docker** тоже должен работать.
