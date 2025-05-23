# Руководство по сопровождению

Данный документ содержит информацию по сопровождению, и будет полезен
в первую очередь сопровождающим (maintainers) репозитория.

## Обновление зависимостей и инструментов

В понедельник по крону создается PR с обновлением
[Cargo.lock](./Cargo.lock) файла, чтобы убедиться в том, что все зависимости
актуальны и не имеют известных уязвимостей. Также позже может прийти dependabot
со своими обновлениями.

### Обновление rust

Зависимости плагина требуют ночной версии Rust. При необходимости обновите поле
`channel` в файле [rust-toolchain](./rust-toolchain.toml) на более новую версию.

### Обновление swc_core

swc_core библиотека предназначена для взаимодействия swc с плагинами.
Обновление до новой мажорной версии происходит вручную, так как это может
потребовать изменений в коде. Рекомендуется отслеживать изменения в [swc-project/plugins](https://github.com/swc-project/plugins/commits/main/).
и делать аналогичные действия

Для определения какой вид версии (мажор, минор или патч) требуется выпустить,
рекомендуется смотреть на [swc-project/plugins](https://github.com/swc-project/plugins/commits/main/)

Примеры обновления swc_core:

- [#274](https://github.com/VKCOM/swc-plugin-css-modules/pull/274)
- [#272](https://github.com/VKCOM/swc-plugin-css-modules/pull/272)
- [#269](https://github.com/VKCOM/swc-plugin-css-modules/pull/269)

## Выпуск релизов

Запустите экшон Publish с необходимым типом версии по
[семантическому версионированию](https://semver.org/lang/ru/). После публикации
создайте релиз на GitHub с описанием изменений.

Если это обновление `swc_core` напишите на какую версию произошло обновление,
а также добавьте ссылку на https://plugins.swc.rs/versions/range.

Пример:

```md
## What's Changed

[swc_core=14](https://plugins.swc.rs/versions/range)

**Full Changelog**: https://github.com/VKCOM/swc-plugin-css-modules/compare/v2.2.0...v2.2.1
```
