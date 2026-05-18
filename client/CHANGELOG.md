# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-05-18

### <!-- 0 -->🚀 Features

- [**breaking**] *(agent)* Add --json-table flag and remove --pretty

### <!-- 4 -->🔧 Refactor

- *(client)* Consolidate duplicate form encoding functions
- *(client)* Extract shared test scaffolding to test_support
- *(client)* Consolidate multipart form helpers into client.rs
- *(client)* Add post_datatables helper to deduplicate post+parse pattern
- Convert fetch_limit and fetch_all to Client methods

### <!-- 7 -->⚙️ Miscellaneous

- Enforce missing_docs lint in both crate roots


## [0.1.4] - 2026-05-18

### <!-- 1 -->🐛 Bug Fixes

- *(agent)* Omit PercentDailyVolume from trade-shaped output
- *(agent)* Trim sparse compact default fields


## [0.1.3] - 2026-05-18

### <!-- 7 -->⚙️ Miscellaneous

- Update Rust before release artifact builds


## [0.1.2] - 2026-05-18

### <!-- 0 -->🚀 Features

- *(client)* Use rust_decimal for currency fields


## [0.1.1] - 2026-05-17

### <!-- 7 -->⚙️ Miscellaneous

- Align release automation with crates publishing

