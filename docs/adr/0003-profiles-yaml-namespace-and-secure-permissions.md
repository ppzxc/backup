# 0003. Profiles YAML Namespace Isolation and Secure Permission Enforcement

## Status

Accepted

## Context

코드베이스 하드코딩 전수조사 및 AGENTS.md 규정 검토 과정에서 다음 문제점들이 식별되었습니다:

1. **Resticprofile 설정 공유 파일(`profiles.yaml`)의 키 충돌 위험**: `backup` CLI 어플리케이션과 `resticprofile` 바이너리가 동일한 `/etc/backup/profiles.yaml` 설정 파일을 공유함에 따라, `reports`, `audit` 등 backup 전용 설정 항목이 resticprofile 고유 스키마(`version`, `global`, `groups`, `profiles`)와 충돌하거나 오파싱될 가능성이 존재함.
2. **Unix 권한 강제 미적용**: `/etc/backup` 디렉터리(`700`) 및 생성되는 설정 파일(`600`)에 대한 POSIX 권한이 `fs::write` / `fs::create_dir_all` 시점에 명시적으로 지정되지 않아 컴플라이언스 위험이 존재함.
3. **하드코딩된 평문 폴백 및 경로 문자열**: 평문 폴백 비밀번호(`default_secret_pass123`), 경로(`"/etc/backup"`, `"/data/backup/reports"`) 등이 소스 코드 곳곳에 무분별하게 하드코딩됨.

## Decision

1. **`profiles.yaml` 네임스페이스 격리**: `profiles.yaml` 내 `global` 또는 `backup_app` 네임스페이스 키를 할당하여 `resticprofile` 고유 키와 접점이 겹치지 않게 구조화하고, `BackupConfig`에 명시된 설정을 최우선 적용하되 미설정 시에만 정적 상수로 구성된 기본값을 디폴트로 폴백함.
2. **보안 권한 강제 도우미 함수 캡슐화**: 디렉터리 생성 및 파일 쓰기 시 Unix 권한 `0o700` (디렉터리) 및 `0o600` (파일)을 강제 적용하는 `create_secure_dir` 및 `save_secure_file` 공통 헬퍼를 캡슐화하여 사용함.
3. **하드코딩 완전 제거 및 강타입화**: 취약한 평문 폴백 비밀번호를 완전 제거하고 필수 입력 에러로 대체하며, DB 백업 타입(`DatabaseType`) 및 진단 상태(`DoctorStatus`, `DoctorCategory`)를 Enum으로 강타입화함.

## Consequences

- `resticprofile` 바이너리와 동일한 `profiles.yaml` 파일 공유 시 설정 키 충돌이나 파싱 에러가 철저히 방지됨.
- 보안 권한이 파일 생성 시점마다 100% 강제 적용되어 ISMS-P 감사 증적 요건을 충족함.
- 소스 코드 내 무분별한 하드코딩 리터럴이 사라지고 타입 안전성(Type Safety)이 극대화됨.
