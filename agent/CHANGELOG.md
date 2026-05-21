# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-05-21

### <!-- 1 -->🐛 Bug Fixes

- *(trade)* Align daily cluster requests
- *(trade)* Match browser cluster defaults
- *(agent)* Match browser trade request defaults

### <!-- 4 -->🔧 Refactor

- [**breaking**] *(agent)* Remove json-table output

### <!-- 7 -->⚙️ Miscellaneous

- *(ci)* Add Codecov config and local patch coverage target


## [0.3.0] - 2026-05-18

### <!-- 0 -->🚀 Features

- [**breaking**] *(agent)* Add --json-table flag and remove --pretty

### <!-- 6 -->🧪 Testing

- Raise test coverage above 90% threshold

### <!-- 7 -->⚙️ Miscellaneous

- Enforce missing_docs lint in both crate roots


## [0.2.2] - 2026-05-18

### <!-- 1 -->🐛 Bug Fixes

- *(agent)* Omit PercentDailyVolume from trade-shaped output
- *(agent)* Trim sparse compact default fields
- *(agent)* Restore daily RSI and strip question-mark placeholders

### <!-- 4 -->🔧 Refactor

- *(agent)* Deduplicate TRADE_HEADERS and DATE_FMT constants
- *(agent)* Shorten verbose time field names in output

### <!-- 5 -->🎨 Styling

- Remove extra blank line from conflict resolution


## [0.2.1] - 2026-05-18

### <!-- 7 -->⚙️ Miscellaneous

- Update Rust before release artifact builds


## [0.2.0] - 2026-05-18

### <!-- 4 -->🔧 Refactor

- Remove CSV/TSV output, keep JSON only
- *(agent)* Centralize order direction mapping
- *(agent)* Extract shared client command scaffolding
- *(agent)* Decompose trade command modules


## [0.1.2] - 2026-05-18

### <!-- 0 -->🚀 Features

- *(agent)* Add dashboard transforms for token-efficient output

### <!-- 4 -->🔧 Refactor

- *(agent)* Share trade output transforms

### <!-- 6 -->🧪 Testing

- *(agent)* Cover cluster output transforms


## [0.1.1] - 2026-05-17

### <!-- 7 -->⚙️ Miscellaneous

- Align release automation with crates publishing

