# i18n Design Specification: Interactive Wizard & CLI Help Messages

## 1. Goal
Provide bilingual (English & Korean) support across the `backup` CLI:
- **Interactive Setup Wizard (`backup setup`)**: Language selection prompt at startup or `--lang` CLI flag (Option A).
- **CLI Help Messages (`backup --help`)**: Environment-based language detection (`LANG` / `LC_ALL`) falling back to English (Option B), or bilingual dual-text display for maximum accessibility.

## 2. Architecture & Components

```
┌───────────────────────────────────────────────────────────┐
│                    backup CLI (main.rs)                   │
└─────────────────────────────┬─────────────────────────────┘
                              │
               ┌──────────────┴──────────────┐
               ▼                             ▼
   ┌──────────────────────┐      ┌──────────────────────┐
   │    src/i18n.rs       │      │  setup.rs / Prompter │
   │ (Language & Dict)    │      │  (Interactive I18n)  │
   └──────────────────────┘      └──────────────────────┘
```

### 2.1 `src/i18n.rs`
- **`Language` Enum**: `Ko`, `En`.
  - `Language::detect()`: Reads `LANG` / `LC_ALL` environment variables. If contains `"ko"`, returns `Language::Ko`, else `Language::En`.
  - `Language::from_str(s)`: Parses `"ko"`, `"en"`.
- **`I18nMessages` Struct**: Contains localized strings for:
  - Setup prompts (Profile name, Backup type, DB type, Retention, Storage backend, Encryption password warning).
  - Help texts (Subcommand descriptions, options).

### 2.2 Interactive Wizard (`src/commands/setup.rs`)
- `InquirePrompter` receives `Option<Language>`.
- If `lang` is `None` (and `non_interactive` is false), prompt:
  ```text
  ? Select Language / 언어 선택:
  > [1] 한국어 (Korean)
    [2] English
  ```
- All subsequent `inquire` prompts use the resolved `Language` messages from `I18nMessages`.

### 2.3 CLI Help (`src/main.rs`)
- Subcommand help docstrings are updated to include clear bilingual explanations (English & Korean) or dynamically formatted help texts based on `Language::detect()`.

## 3. Testing Seams
- Unit tests for `Language::detect()`, `Language::from_str()`, and localized string lookup in `src/i18n.rs`.
- Integration tests in `tests/cmd_setup_test.rs` verifying prompt responses under `Language::Ko` and `Language::En`.
