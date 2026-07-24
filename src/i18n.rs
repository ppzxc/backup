#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ko,
    En,
}

impl Language {
    pub fn detect() -> Self {
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_default();
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
            },
        }
    }
}
