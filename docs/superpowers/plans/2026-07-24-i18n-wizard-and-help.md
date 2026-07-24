# Interactive Setup Wizard & CLI Help i18n Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement bilingual (English & Korean) support for the interactive setup wizard and CLI help messages.

**Architecture:** Create a central `src/i18n.rs` module with `Language` enum and `I18nMessages` dictionary, update `SetupPrompter` in `src/commands/setup.rs` to support language selection and localized prompts, and update CLI descriptions in `src/main.rs`.

**Tech Stack:** Rust, Clap 4.5, Inquire 0.7, Anyhow.

## Global Constraints
- POSIX permissions (0700/0600) rules must remain strictly enforced.
- Existing tests must pass without regressions.
- No heavy external i18n crates; use lightweight native Rust dictionary.

---

### Task 1: Implement `src/i18n.rs` Module

**Files:**
- Create: `src/i18n.rs`
- Modify: `src/lib.rs`
- Test: `tests/i18n_test.rs`

- [ ] **Step 1: Write failing unit test for `Language` and `I18nMessages`**

Create `tests/i18n_test.rs`:
```rust
use backup::i18n::{Language, I18nMessages};

#[test]
fn test_language_parse_and_detection() {
    assert_eq!(Language::from_str("ko"), Language::Ko);
    assert_eq!(Language::from_str("en"), Language::En);
    assert_eq!(Language::from_str("invalid"), Language::En);
}

#[test]
fn test_i18n_messages_lookup() {
    let msg_ko = I18nMessages::get(Language::Ko);
    let msg_en = I18nMessages::get(Language::En);

    assert!(msg_ko.enter_profile_name.contains("프로필"));
    assert!(msg_en.enter_profile_name.contains("Profile"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test i18n_test`
Expected: FAIL with "unresolved import / module not found"

- [ ] **Step 3: Implement `src/i18n.rs` and update `src/lib.rs`**

Create `src/i18n.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ko,
    En,
}

impl Language {
    pub fn detect() -> Self {
        let lang = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")).unwrap_or_default();
        if lang.to_lowercase().contains("ko") {
            Language::Ko
        } else {
            Language::En
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ko" | "korean" => Language::Ko,
            _ => Language::En,
        }
    }
}

pub struct I18nMessages {
    pub select_language: &'static str,
    pub enter_profile_name: &'static str,
    pub select_backup_type: &'static str,
    pub dir_batch_backup: &'static str,
    pub db_stream_backup: &'static str,
    pub enter_target_dir: &'static str,
    pub select_db_type: &'static str,
    pub enter_conn_url: &'static str,
    pub enter_exclude_patterns: &'static str,
    pub retention_keep_daily: &'static str,
    pub retention_keep_weekly: &'static str,
    pub retention_keep_monthly: &'static str,
    pub primary_storage_backend: &'static str,
    pub primary_repo_uri: &'static str,
    pub enter_encryption_password: &'static str,
    pub isms_password_error: &'static str,
    pub sftp_host: &'static str,
    pub sftp_port: &'static str,
    pub sftp_user: &'static str,
    pub sftp_key_file: &'static str,
    pub isms_sftp_key_error: &'static str,
    pub config_secondary_storage: &'static str,
    pub secondary_backend: &'static str,
    pub secondary_repo_uri: &'static str,
    pub secondary_password: &'static str,
    pub enable_isms_reports: &'static str,
    pub report_export_dir: &'static str,
}

impl I18nMessages {
    pub fn get(lang: Language) -> Self {
        match lang {
            Language::Ko => Self {
                select_language: "언어 선택 / Select Language:",
                enter_profile_name: "백업 프로필 이름 입력:",
                select_backup_type: "백업 유형 선택:",
                dir_batch_backup: "[1] 디렉터리 일괄 백업 (Directory Batch)",
                db_stream_backup: "[2] DB 스트리밍 백업 (DB Stream)",
                enter_target_dir: "백업 대상 디렉터리 경로 (쉼표 구분):",
                select_db_type: "DB 유형 선택:",
                enter_conn_url: "DB 접속 URL (선택사항):",
                enter_exclude_patterns: "제외할 패턴 (선택사항, 쉼표 구분):",
                retention_keep_daily: "보존 정책: 일간 스냅샷 보존 개수:",
                retention_keep_weekly: "보존 정책: 주간 스냅샷 보존 개수:",
                retention_keep_monthly: "보존 정책: 월간 스냅샷 보존 개수:",
                primary_storage_backend: "1차 저장소 백엔드 선택:",
                primary_repo_uri: "1차 저장소 저장소(Repository) URI:",
                enter_encryption_password: "암호화 비밀번호 입력 (ISMS 규정: 최소 12자 이상):",
                isms_password_error: "ISMS 보안 통제 오류: 비밀번호는 최소 12자 이상이어야 합니다.",
                sftp_host: "SFTP 호스트 주소:",
                sftp_port: "SFTP 포트 번호:",
                sftp_user: "SFTP 계정 사용자명:",
                sftp_key_file: "SFTP SSH 개인키 파일 경로 (ISMS 필수):",
                isms_sftp_key_error: "ISMS 보안 통제 오류: SFTP 사용 시 비밀번호 없는 키 인증을 위한 key_file 경로가 필수입니다.",
                config_secondary_storage: "2차 (원격/소외) 저장소 설정 여부:",
                secondary_backend: "2차 저장소 백엔드 선택:",
                secondary_repo_uri: "2차 저장소 URI:",
                secondary_password: "2차 저장소 비밀번호:",
                enable_isms_reports: "ISMS 감사 및 일일/연간 보고서 자동 생성 활성화:",
                report_export_dir: "보고서 출력 디렉터리 경로:",
            },
            Language::En => Self {
                select_language: "Select Language / 언어 선택:",
                enter_profile_name: "Enter Profile Name:",
                select_backup_type: "Select Backup Type:",
                dir_batch_backup: "[1] Directory Batch Backup",
                db_stream_backup: "[2] DB Streaming Backup",
                enter_target_dir: "Enter Target Directory(ies), comma-separated:",
                select_db_type: "Select DB Type:",
                enter_conn_url: "Enter Connection URL (optional):",
                enter_exclude_patterns: "Enter Exclude Patterns, comma-separated (optional):",
                retention_keep_daily: "Retention: Keep Daily Snapshots:",
                retention_keep_weekly: "Retention: Keep Weekly Snapshots:",
                retention_keep_monthly: "Retention: Keep Monthly Snapshots:",
                primary_storage_backend: "Primary Storage Backend:",
                primary_repo_uri: "Primary Repository URI:",
                enter_encryption_password: "Enter Encryption Password (min 12 chars for ISMS):",
                isms_password_error: "ISMS Compliance Error: Password must be at least 12 characters long.",
                sftp_host: "SFTP Host:",
                sftp_port: "SFTP Port:",
                sftp_user: "SFTP User:",
                sftp_key_file: "SFTP SSH Key File Path (Required for ISMS):",
                isms_sftp_key_error: "ISMS Compliance Error: SFTP requires SSH key_file path for passwordless key-based authentication.",
                config_secondary_storage: "Configure Secondary (Offsite/Redundant) Storage?",
                secondary_backend: "Secondary Backend:",
                secondary_repo_uri: "Secondary Repository URI:",
                secondary_password: "Secondary Password:",
                enable_isms_reports: "Enable ISMS Audit & Daily/Annual Report Generation?",
                report_export_dir: "Report Export Directory:",
            },
        }
    }
}
```

Update `src/lib.rs`:
```rust
pub mod i18n;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test i18n_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/i18n.rs src/lib.rs tests/i18n_test.rs
git commit -m "feat(i18n): implement Language enum and I18nMessages dictionary"
```

---

### Task 2: Update Setup Wizard (`src/commands/setup.rs`) with Language Support

**Files:**
- Modify: `src/commands/setup.rs`
- Modify: `src/main.rs`
- Modify: `tests/cmd_setup_test.rs`

- [ ] **Step 1: Update `SetupPrompter` Trait and `InquirePrompter` implementation**

Update `src/commands/setup.rs`:
```rust
use crate::i18n::{Language, I18nMessages};

pub trait SetupPrompter {
    fn prompt_setup_params(&self, lang_opt: Option<Language>) -> Result<SetupParams>;
}

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(&self, lang_opt: Option<Language>) -> Result<SetupParams> {
        let lang = match lang_opt {
            Some(l) => l,
            None => {
                let choice = inquire::Select::new(
                    "Select Language / 언어 선택:",
                    vec!["[1] 한국어 (Korean)", "[2] English"],
                ).prompt()?;
                if choice.starts_with("[1]") {
                    Language::Ko
                } else {
                    Language::En
                }
            }
        };

        let msg = I18nMessages::get(lang);

        let profile = inquire::Text::new(msg.enter_profile_name)
            .with_default("default")
            .prompt()?;

        let backup_type_choice = inquire::Select::new(
            msg.select_backup_type,
            vec![msg.dir_batch_backup, msg.db_stream_backup],
        ).prompt()?;

        let (backup_type, targets) = if backup_type_choice.starts_with("[1]") {
            let t = inquire::Text::new(msg.enter_target_dir)
                .with_default("/data")
                .prompt()?;
            let target_list: Vec<String> = t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (BackupType::Directory, target_list)
        } else {
            let db_kind = inquire::Select::new(msg.select_db_type, vec!["mysql", "postgres"]).prompt()?;
            let conn = inquire::Text::new(msg.enter_conn_url).prompt_skippable()?;
            (
                BackupType::DbStream {
                    db_type: db_kind.to_string(),
                    connection_url: conn.filter(|s| !s.is_empty()),
                    dump_command: None,
                },
                vec![format!("db-stream:{}", db_kind)],
            )
        };

        let excludes_str = inquire::Text::new(msg.enter_exclude_patterns)
            .with_default("")
            .prompt()?;
        let excludes: Vec<String> = excludes_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        let (default_daily, default_weekly, default_monthly) = match backup_type {
            BackupType::Directory => (30, 4, 12),
            BackupType::DbStream { .. } => (180, 12, 24),
        };

        let keep_daily = inquire::CustomType::<u32>::new(msg.retention_keep_daily)
            .with_default(default_daily)
            .prompt()?;
        let keep_weekly = inquire::CustomType::<u32>::new(msg.retention_keep_weekly)
            .with_default(default_weekly)
            .prompt()?;
        let keep_monthly = inquire::CustomType::<u32>::new(msg.retention_keep_monthly)
            .with_default(default_monthly)
            .prompt()?;

        let backend = inquire::Select::new(msg.primary_storage_backend, vec!["sftp", "s3", "local"])
            .prompt()?;

        let repository = inquire::Text::new(msg.primary_repo_uri)
            .with_default("sftp:user@backup-server:/var/backups")
            .prompt()?;

        let password = inquire::Password::new(msg.enter_encryption_password)
            .without_confirmation()
            .prompt()?;
        
        if password.len() < 12 {
            anyhow::bail!(msg.isms_password_error);
        }

        let sftp_config = if backend == "sftp" {
            let host = inquire::Text::new(msg.sftp_host).with_default("backup-server").prompt()?;
            let port = inquire::CustomType::<u16>::new(msg.sftp_port).with_default(22).prompt()?;
            let user = inquire::Text::new(msg.sftp_user).with_default("backup").prompt()?;
            let key_file = inquire::Text::new(msg.sftp_key_file)
                .with_default("/etc/backup/id_rsa")
                .prompt()?;
            if key_file.trim().is_empty() {
                anyhow::bail!(msg.isms_sftp_key_error);
            }
            Some(SftpConfig {
                host,
                port,
                user,
                key_file: Some(key_file),
            })
        } else {
            None
        };

        let primary_storage = StorageTarget {
            backend: backend.to_string(),
            repository,
            password: SecretString::new(password),
            sftp: sftp_config,
            s3: None,
        };

        let enable_sec = inquire::Confirm::new(msg.config_secondary_storage)
            .with_default(false)
            .prompt()?;

        let secondary_storage = if enable_sec {
            let sec_backend = inquire::Select::new(msg.secondary_backend, vec!["sftp", "s3", "local"]).prompt()?;
            let sec_repo = inquire::Text::new(msg.secondary_repo_uri).prompt()?;
            let sec_pass = inquire::Password::new(msg.secondary_password).without_confirmation().prompt()?;
            Some(SecondaryStorageTarget {
                enabled: true,
                backend: sec_backend.to_string(),
                repository: sec_repo,
                password: SecretString::new(sec_pass),
            })
        } else {
            None
        };

        let enable_reports = inquire::Confirm::new(msg.enable_isms_reports)
            .with_default(true)
            .prompt()?;

        let reports = if enable_reports {
            let output_dir = inquire::Text::new(msg.report_export_dir)
                .with_default("/var/log/backup/reports")
                .prompt()?;
            ReportsConfig {
                output_dir,
                enable_daily_reports: true,
                enable_annual_dr_drill_report: true,
            }
        } else {
            ReportsConfig {
                output_dir: "/var/log/backup/reports".into(),
                enable_daily_reports: false,
                enable_annual_dr_drill_report: false,
            }
        };

        Ok(SetupParams {
            profile,
            backup_type,
            targets,
            excludes,
            retention: RetentionPolicy {
                keep_daily,
                keep_weekly,
                keep_monthly,
            },
            primary_storage,
            secondary_storage,
            reports,
        })
    }
}
```

Update `src/main.rs` CLI parsing for `setup`:
```rust
    Setup {
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        non_interactive: bool,
        #[command(subcommand)]
        action: Option<SetupAction>,
    },
```

- [ ] **Step 2: Run cargo test to verify**

Run: `cargo test`
Expected: PASS (fix mock prompters in `tests/cmd_setup_test.rs` if needed)

- [ ] **Step 3: Commit**

```bash
git add src/commands/setup.rs src/main.rs tests/cmd_setup_test.rs
git commit -m "feat(i18n): update interactive setup wizard with Korean and English prompter messages"
```

---

### Task 3: Bilingual CLI Help Descriptions in `src/main.rs`

**Files:**
- Modify: `src/main.rs`
- Test: `tests/cli_test.rs`

- [ ] **Step 1: Update `src/main.rs` docstrings with bilingual descriptions**

Update subcommand docstrings in `src/main.rs`:
```rust
#[derive(Subcommand)]
enum Commands {
    /// Backup Environment and Backup Profile setup / 백업 환경 및 프로필 설정 마법사
    Setup {
        /// Select language (ko/en) / 언어 선택 (ko/en)
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        non_interactive: bool,
        #[command(subcommand)]
        action: Option<SetupAction>,
    },
    /// Configuration Registry management / 백업 설정 레지스트리 관리
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Storage Backend Adapter migration / 저장소 백엔드 마이그레이션
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
    /// Execute backup pipeline / 백업 파이프라인 수동 실행
    Run {
        #[arg(long)]
        skip_database: bool,
        #[arg(long)]
        skip_secondary_sync: bool,
        #[arg(long)]
        skip_retention: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// System, security, and audit report diagnostics / 시스템, 보안 및 ISMS-P 진단 보고서
    Doctor {
        #[command(subcommand)]
        action: Option<DoctorAction>,
    },
    /// Systemd timer / Cron scheduler management / 스케줄러 타이머 관리
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Restore files or database dumps from snapshot / 스냅샷 기반 파일 및 DB 복구
    Restore {
        #[arg(long, default_value = "latest")]
        snapshot: String,
        #[arg(long, default_value = "/tmp/restore")]
        target: String,
    },
    /// List snapshots across primary and secondary storage targets / 스냅샷 목록 조회
    Snapshots,
    /// Display operational status and snapshot recency / 운영 상태 및 스냅샷 주기 확인
    Status,
    /// Self-update backup binary and assets / 바이너리 및 자산 자가 업데이트
    Update,
    /// Uninstall backup CLI and scheduled timers / 백업 CLI 및 스케줄러 삭제
    Uninstall {
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        purge: bool,
    },
}
```

- [ ] **Step 2: Run all tests to confirm zero regressions**

Run: `cargo test`
Expected: All unit and integration tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "docs(cli): add bilingual docstrings to CLI subcommands in main.rs"
```
