#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ko,
    En,
}

impl Language {
    /// POSIX 표준 우선순위(개리`LC_ALL > LANG`):
    /// LC_ALL이 설정되어 있으면 언제나 우선하고, 그 다음에 LANG을 확인합니다.
    pub fn detect() -> Self {
        let lc_all = std::env::var("LC_ALL").ok();
        let lang = std::env::var("LANG").ok();
        Self::detect_from(lang.as_deref(), lc_all.as_deref())
    }

    /// 환경변수 접근 없이 순수 인자로 언어를 감지합니다. 테스트 우호적 설계.
    /// POSIX 표준: LC_ALL이 설정되어 있으면 LANG보다 우선합니다.
    pub fn detect_from(lang: Option<&str>, lc_all: Option<&str>) -> Self {
        let effective = lc_all
            .filter(|s| !s.is_empty())
            .or_else(|| lang.filter(|s| !s.is_empty()))
            .unwrap_or("");
        if effective.to_lowercase().contains("ko") {
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

#[derive(Debug, Clone, Copy)]
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
    pub found_existing_keyfile: &'static str,
    pub auto_generate_password_prompt: &'static str,
    pub save_password_to_keyfile_prompt: &'static str,
    pub sftp_host: &'static str,
    pub sftp_port: &'static str,
    pub sftp_user: &'static str,
    pub sftp_path: &'static str,
    pub sftp_auto_gen_key: &'static str,
    pub sftp_key_file: &'static str,
    pub isms_sftp_key_error: &'static str,
    pub config_secondary_storage: &'static str,
    pub secondary_backend: &'static str,
    pub secondary_repo_uri: &'static str,
    pub secondary_password: &'static str,
    pub enable_isms_reports: &'static str,
    pub report_export_dir: &'static str,

    // S3 setup messages
    pub s3_mode_select: &'static str,
    pub s3_mode_detailed: &'static str,
    pub s3_mode_uri_only: &'static str,
    pub s3_endpoint: &'static str,
    pub s3_access_key_id: &'static str,
    pub s3_secret_access_key: &'static str,
    pub s3_region: &'static str,
    pub s3_bucket: &'static str,
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
                enter_encryption_password: "암호화 비밀번호 입력 (ISMS 준수를 위해 최소 12자 이상):",
                isms_password_error: "ISMS 컴플라이언스 오류: 비밀번호는 최소 12자 이상이어야 합니다.",
                found_existing_keyfile: "기존 암호화 키파일(/etc/backup/enc)을 발견했습니다. 비밀번호를 자동으로 로드합니다.",
                auto_generate_password_prompt: "안전한 32자 암호화 비밀번호를 자동 생성할까요? (/etc/backup/enc 보관)",
                save_password_to_keyfile_prompt: "입력한 비밀번호를 암호화 키파일(/etc/backup/enc)로 저장할까요?",
                sftp_host: "SFTP 호스트 주소 (IP 또는 도메인):",
                sftp_port: "SFTP 포트 번호:",
                sftp_user: "SFTP 계정 사용자명:",
                sftp_path: "SFTP 원격 백업 경로:",
                sftp_auto_gen_key: "SSH 키쌍(/etc/backup/id_ed25519)을 자동 생성하시겠습니까?",
                sftp_key_file: "SFTP SSH 개인키 파일 경로 (ISMS 필수):",
                isms_sftp_key_error: "ISMS 보안 통제 오류: SFTP 사용 시 비밀번호 없는 키 인증을 위한 key_file 경로가 필수입니다.",
                config_secondary_storage: "2차 (원격/소외) 저장소 설정 여부:",
                secondary_backend: "2차 저장소 백엔드 선택:",
                secondary_repo_uri: "2차 저장소 URI:",
                secondary_password: "2차 저장소 비밀번호:",
                enable_isms_reports: "ISMS 감사 및 일일/연간 보고서 자동 생성 활성화:",
                report_export_dir: "보고서 출력 디렉터리 경로:",

                s3_mode_select: "S3 설정 방식 선택:",
                s3_mode_detailed: "[1] S3 상세 정보 분해 입력 (Endpoint, Access Key, Secret Key 등) - 기본값",
                s3_mode_uri_only: "[2] S3 Repository URI만 직접 입력",
                s3_endpoint: "S3 엔드포인트 URL:",
                s3_access_key_id: "S3 Access Key ID:",
                s3_secret_access_key: "S3 Secret Access Key:",
                s3_region: "S3 리전 (선택사항):",
                s3_bucket: "S3 버킷 이름:",
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
                found_existing_keyfile: "Found existing encryption keyfile (/etc/backup/enc). Loaded automatically.",
                auto_generate_password_prompt: "Auto-generate secure 32-char encryption password? (Saved to /etc/backup/enc)",
                save_password_to_keyfile_prompt: "Save entered password to encryption keyfile (/etc/backup/enc)?",
                sftp_host: "SFTP Host Address (IP or Domain):",
                sftp_port: "SFTP Port:",
                sftp_user: "SFTP User:",
                sftp_path: "SFTP Remote Backup Path:",
                sftp_auto_gen_key: "Automatically generate SSH keypair (/etc/backup/id_ed25519)?",
                sftp_key_file: "SFTP SSH Key File Path (Required for ISMS):",
                isms_sftp_key_error: "ISMS Compliance Error: SFTP requires SSH key_file path for passwordless key-based authentication.",
                config_secondary_storage: "Configure Secondary (Offsite/Redundant) Storage?",
                secondary_backend: "Secondary Backend:",
                secondary_repo_uri: "Secondary Repository URI:",
                secondary_password: "Secondary Password:",
                enable_isms_reports: "Enable ISMS Audit & Daily/Annual Report Generation?",
                report_export_dir: "Report Export Directory:",

                s3_mode_select: "Select S3 Configuration Mode:",
                s3_mode_detailed: "[1] Detailed S3 Parameters (Endpoint, Access Key, Secret Key, etc.) - Default",
                s3_mode_uri_only: "[2] S3 Repository URI Only",
                s3_endpoint: "S3 Endpoint URL:",
                s3_access_key_id: "S3 Access Key ID:",
                s3_secret_access_key: "S3 Secret Access Key:",
                s3_region: "S3 Region (optional):",
                s3_bucket: "S3 Bucket Name:",
            },
        }
    }
}

/// CLI 도움말 텍스트 (서브커맨드/옵션별 언어별 설명)
#[derive(Debug, Clone, Copy)]
pub struct CliHelp {
    // top-level about
    pub about: &'static str,
    // subcommands
    pub cmd_setup: &'static str,
    pub cmd_config: &'static str,
    pub cmd_backend: &'static str,
    pub cmd_run: &'static str,
    pub cmd_doctor: &'static str,
    pub cmd_schedule: &'static str,
    pub cmd_restore: &'static str,
    pub cmd_snapshots: &'static str,
    pub cmd_status: &'static str,
    pub cmd_update: &'static str,
    pub cmd_version: &'static str,
    pub cmd_uninstall: &'static str,
    // setup sub-subcommands
    pub cmd_setup_dependencies: &'static str,
    pub cmd_setup_backend_init: &'static str,
    // setup options
    pub opt_setup_lang: &'static str,
    pub opt_setup_non_interactive: &'static str,
    // config sub-subcommands
    pub cmd_config_show: &'static str,
    pub cmd_config_edit: &'static str,
    pub cmd_config_import_legacy: &'static str,
    pub cmd_config_export: &'static str,
    pub opt_config_import_file: &'static str,
    pub opt_config_export_format: &'static str,
    // backend sub-subcommands
    pub cmd_backend_migrate: &'static str,
    // run options
    pub opt_run_skip_database: &'static str,
    pub opt_run_skip_secondary_sync: &'static str,
    pub opt_run_skip_retention: &'static str,
    pub opt_run_dry_run: &'static str,
    // doctor sub-subcommands
    pub cmd_doctor_environment: &'static str,
    pub cmd_doctor_time_sync: &'static str,
    pub cmd_doctor_restore_drill: &'static str,
    pub opt_doctor_file: &'static str,
    // schedule sub-subcommands
    pub cmd_schedule_enable: &'static str,
    pub cmd_schedule_disable: &'static str,
    pub cmd_schedule_status: &'static str,
    // restore options
    pub opt_restore_snapshot: &'static str,
    pub opt_restore_target: &'static str,
    // uninstall options
    pub opt_uninstall_yes: &'static str,
    pub opt_uninstall_purge: &'static str,
}

impl CliHelp {
    pub fn get(lang: Language) -> Self {
        match lang {
            Language::Ko => Self {
                about: "백업 CLI — restic/rclone 기반 백업 자동화 도구",
                cmd_setup: "백업 환경 및 백업 프로필 설정 마법사",
                cmd_config: "백업 설정 레지스트리 관리",
                cmd_backend: "저장소 백엔드 어댑터 마이그레이션",
                cmd_run: "백업 파이프라인 수동 실행",
                cmd_doctor: "시스템, 보안 및 ISMS-P 진단 보고서",
                cmd_schedule: "Systemd 타이머 / Cron 스케줄러 관리",
                cmd_restore: "스냅샷 기반 파일 및 DB 복구",
                cmd_snapshots: "1차·2차 저장소의 스냅샷 목록 조회",
                cmd_status: "운영 상태 및 스냅샷 주기 확인",
                cmd_update: "백업 바이너리 및 자산 자가 업데이트",
                cmd_version: "CLI 바이너리 버전 표시",
                cmd_uninstall: "백업 CLI 및 스케줄러 삭제",
                cmd_setup_dependencies: "필수 바이너리 의존성(restic, rclone, resticprofile) 확인 및 다운로드",
                cmd_setup_backend_init: "1차·2차 백엔드 어댑터 저장소 초기화",
                opt_setup_lang: "언어 선택 (ko/en)",
                opt_setup_non_interactive: "비대화형(자동화) 모드로 실행",
                cmd_config_show: "마스킹 처리된 비밀값으로 현재 설정 값 표시",
                cmd_config_edit: "권한 검증과 함께 설정 파일 편집",
                cmd_config_import_legacy: "기존 Bash 스타일 backup.env 설정 가져오기",
                cmd_config_export: "지정한 형식으로 현재 설정 내보내기",
                opt_config_import_file: "가져올 레거시 설정 파일 경로",
                opt_config_export_format: "내보내기 형식 (기본값: yaml)",
                cmd_backend_migrate: "1차 저장소에서 새 저장소 대상으로 스냅샷 마이그레이션",
                opt_run_skip_database: "데이터베이스 백업 단계 건너뜀",
                opt_run_skip_secondary_sync: "2차 저장소 동기화 단계 건너뜀",
                opt_run_skip_retention: "보존 정책 적용 단계 건너뜀",
                opt_run_dry_run: "실제 실행 없이 파이프라인 시뮬레이션",
                cmd_doctor_environment: "백업 환경 디렉터리/파일 권한 및 비밀값 마스킹 검사",
                cmd_doctor_time_sync: "NTP/Chrony 시간 동기화 상태 점검",
                cmd_doctor_restore_drill: "복구 드릴 실행, RTO 측정 및 DB 헤더 무결성 확인",
                opt_doctor_file: "진단 결과를 내보낼 파일 경로",
                cmd_schedule_enable: "Systemd 타이머 / Cron 폴백 활성화",
                cmd_schedule_disable: "예약 타이머 비활성화",
                cmd_schedule_status: "타이머/스케줄러 상태 표시",
                opt_restore_snapshot: "복구할 스냅샷 ID (기본값: latest)",
                opt_restore_target: "복구 대상 경로 (기본값: /tmp/restore)",
                opt_uninstall_yes: "삭제 확인 (프롬프트 생략)",
                opt_uninstall_purge: "설정 파일 및 데이터까지 완전 삭제",
            },
            Language::En => Self {
                about: "Backup CLI — restic/rclone based backup automation tool",
                cmd_setup: "Backup environment and profile setup wizard",
                cmd_config: "Configuration registry management",
                cmd_backend: "Storage backend adapter migration",
                cmd_run: "Execute backup pipeline manually",
                cmd_doctor: "System, security, and audit report diagnostics",
                cmd_schedule: "Systemd timer / Cron scheduler management",
                cmd_restore: "Restore files or database dumps from snapshot",
                cmd_snapshots: "List snapshots across primary and secondary storage targets",
                cmd_status: "Display operational status and snapshot recency",
                cmd_update: "Self-update backup binary and assets",
                cmd_version: "Display CLI binary version",
                cmd_uninstall: "Uninstall backup CLI and scheduled timers",
                cmd_setup_dependencies: "Verify and download required binary dependencies (restic, rclone, resticprofile)",
                cmd_setup_backend_init: "Initialize primary and secondary backend adapter repositories",
                opt_setup_lang: "Select language (ko/en)",
                opt_setup_non_interactive: "Run in non-interactive (automation) mode",
                cmd_config_show: "Show active configuration values with masked secrets",
                cmd_config_edit: "Edit configuration file with permission validation",
                cmd_config_import_legacy: "Import legacy Bash-style backup.env configuration",
                cmd_config_export: "Export active configuration in specified format",
                opt_config_import_file: "Path to the legacy config file to import",
                opt_config_export_format: "Export format (default: yaml)",
                cmd_backend_migrate: "Migrate snapshots from primary to new storage target",
                opt_run_skip_database: "Skip the database backup step",
                opt_run_skip_secondary_sync: "Skip the secondary storage sync step",
                opt_run_skip_retention: "Skip the retention policy enforcement step",
                opt_run_dry_run: "Simulate the pipeline without executing",
                cmd_doctor_environment: "Check backup environment directory/file permissions and secret masking",
                cmd_doctor_time_sync: "Inspect NTP/Chrony time synchronization status",
                cmd_doctor_restore_drill: "Execute restore drill, measure RTO, and check database header integrity",
                opt_doctor_file: "File path to export diagnostic results",
                cmd_schedule_enable: "Enable systemd timers / cron fallback",
                cmd_schedule_disable: "Disable scheduled timers",
                cmd_schedule_status: "Display timer/scheduler status",
                opt_restore_snapshot: "Snapshot ID to restore (default: latest)",
                opt_restore_target: "Restore destination path (default: /tmp/restore)",
                opt_uninstall_yes: "Confirm uninstall (skip prompt)",
                opt_uninstall_purge: "Also remove configuration files and data",
            },
        }
    }

    /// `clap::Command` 트리를 이 CliHelp의 도움말 텍스트로 수정(mutate)합니다.
    /// derive(Parser)로 생성된 command 구조를 유지하면서 about/help 문자열만 교체합니다.
    ///
    /// # 사용 예시
    /// ```ignore
    /// let lang = Language::detect();
    /// let cmd = CliHelp::get(lang).apply_to_command(Cli::command());
    /// ```
    pub fn apply_to_command(&self, mut cmd: clap::Command) -> clap::Command {
        cmd = cmd.about(self.about);

        cmd = cmd.mut_subcommand("setup", |c| {
            c.about(self.cmd_setup)
                .mut_arg("lang", |a| a.help(self.opt_setup_lang))
                .mut_arg("non_interactive", |a| a.help(self.opt_setup_non_interactive))
                .mut_subcommand("dependencies", |s| s.about(self.cmd_setup_dependencies))
                .mut_subcommand("backend-init", |s| s.about(self.cmd_setup_backend_init))
        });

        cmd = cmd.mut_subcommand("config", |c| {
            c.about(self.cmd_config)
                .mut_subcommand("show", |s| s.about(self.cmd_config_show))
                .mut_subcommand("edit", |s| s.about(self.cmd_config_edit))
                .mut_subcommand("import-legacy", |s| {
                    s.about(self.cmd_config_import_legacy)
                        .mut_arg("file", |a| a.help(self.opt_config_import_file))
                })
                .mut_subcommand("export", |s| {
                    s.about(self.cmd_config_export)
                        .mut_arg("format", |a| a.help(self.opt_config_export_format))
                })
        });

        cmd = cmd.mut_subcommand("backend", |c| {
            c.about(self.cmd_backend)
                .mut_subcommand("migrate", |s| s.about(self.cmd_backend_migrate))
        });

        cmd = cmd.mut_subcommand("run", |c| {
            c.about(self.cmd_run)
                .mut_arg("skip_database", |a| a.help(self.opt_run_skip_database))
                .mut_arg("skip_secondary_sync", |a| a.help(self.opt_run_skip_secondary_sync))
                .mut_arg("skip_retention", |a| a.help(self.opt_run_skip_retention))
                .mut_arg("dry_run", |a| a.help(self.opt_run_dry_run))
        });

        cmd = cmd.mut_subcommand("doctor", |c| {
            c.about(self.cmd_doctor)
                .mut_subcommand("environment", |s| {
                    s.about(self.cmd_doctor_environment)
                        .mut_arg("file", |a| a.help(self.opt_doctor_file))
                })
                .mut_subcommand("time-sync", |s| {
                    s.about(self.cmd_doctor_time_sync)
                        .mut_arg("file", |a| a.help(self.opt_doctor_file))
                })
                .mut_subcommand("restore-drill", |s| {
                    s.about(self.cmd_doctor_restore_drill)
                        .mut_arg("file", |a| a.help(self.opt_doctor_file))
                })
        });

        cmd = cmd.mut_subcommand("schedule", |c| {
            c.about(self.cmd_schedule)
                .mut_subcommand("enable", |s| s.about(self.cmd_schedule_enable))
                .mut_subcommand("disable", |s| s.about(self.cmd_schedule_disable))
                .mut_subcommand("status", |s| s.about(self.cmd_schedule_status))
        });

        cmd = cmd.mut_subcommand("restore", |c| {
            c.about(self.cmd_restore)
                .mut_arg("snapshot", |a| a.help(self.opt_restore_snapshot))
                .mut_arg("target", |a| a.help(self.opt_restore_target))
        });

        cmd = cmd.mut_subcommand("snapshots", |c| c.about(self.cmd_snapshots));
        cmd = cmd.mut_subcommand("status", |c| c.about(self.cmd_status));
        cmd = cmd.mut_subcommand("update", |c| c.about(self.cmd_update));
        cmd = cmd.mut_subcommand("version", |c| c.about(self.cmd_version));

        cmd = cmd.mut_subcommand("uninstall", |c| {
            c.about(self.cmd_uninstall)
                .mut_arg("yes", |a| a.help(self.opt_uninstall_yes))
                .mut_arg("purge", |a| a.help(self.opt_uninstall_purge))
        });

        cmd
    }
}

