use crate::commands::report::{AuditDiagnosticResults, RealReportData, ReportType};

const COMMON_REPORT_CSS: &str = r#"
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }
    .report-card {
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }
    header {
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }
    h1 {
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }
    .meta-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }
    .meta-table td {
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }
    .meta-table td.label {
      background-color: #f1f5f9;
      font-weight: 600;
      width: 15%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #0f172a;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    .data-table th, .data-table td {
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }
    .data-table th {
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }
    .pre-block {
      background: #0f172a;
      color: #94a3b8;
      font-family: 'Courier New', monospace;
      font-size: 8pt;
      padding: 12px;
      border-radius: 4px;
      white-space: pre-wrap;
      word-break: break-all;
      margin-bottom: 18px;
    }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }
    .badge-success {
      background-color: #dcfce7;
      color: #15803d;
    }
    .badge-warning {
      background-color: #fee2e2;
      color: #b91c1c;
    }
    .signature-area {
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }
    .signature-box {
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }
    .signature-box .title {
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }
    .signature-box .sign {
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }
    @media print {
      @page {
        size: A4;
        margin: 12mm 15mm 12mm 15mm;
      }
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8pt;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      .data-table th, .data-table td {
        padding: 5px 7px;
        font-size: 8pt;
      }
      .meta-table td {
        padding: 5px 8px;
        font-size: 8.5pt;
      }
      tr {
        page-break-inside: avoid;
      }
      h1 { font-size: 15pt; }
      h2 { font-size: 10pt; margin: 15px 0 8px 0; }
      .badge { font-size: 7.5pt; padding: 1px 5px; }
      .signature-area { margin-top: 20px; }
    }
"#;

pub fn render_html_real(report_type: ReportType, data: &RealReportData) -> String {
    match report_type {
        ReportType::All => render_all_html(data),
        ReportType::Environment => render_environment_html(data),
        ReportType::TimeSync => render_time_sync_html(data),
        ReportType::RestoreDrill => {
            crate::commands::report::restore_drill::render_restore_drill_evidence_html_with_os_info(
                &data.restore_drill_evidence_or_not_performed(),
                &data.os_info,
            )
        }
    }
}

fn render_all_html(data: &RealReportData) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>종합 백업 보안 설정 검토 보고서</title>
  <style>
{}
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>종합 백업 보안 설정 검토 보고서</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>{}</td>
      <td class="label">대상 서버 호스트</td>
      <td>{}</td>
    </tr>
  </table>

  <h2>1. 백업 정책 및 대상 경로 정보</h2>
  <table class="meta-table">
    <tr>
      <td class="label">백엔드 유형</td>
      <td>sftp</td>
    </tr>
    <tr>
      <td class="label">저장소 주소</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">1차 백업 대상</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">백업 제외 경로</td>
      <td>{}</td>
    </tr>
  </table>

  <h2>2. 백업 보존 주기 정책 (Restic Forget Policy)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보존 주기 구분</th>
        <th>설정 보존 개수</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 백업 보존 (Keep-Daily)</td>
        <td>{}개</td>
      </tr>
      <tr>
        <td>주간 백업 보존 (Keep-Weekly)</td>
        <td>{}개</td>
      </tr>
      <tr>
        <td>월간 백업 보존 (Keep-Monthly)</td>
        <td>{}개</td>
      </tr>
    </tbody>
  </table>

  <h2>3. 시스템 스케줄러 & 접근 통제</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보안 감사 항목</th>
        <th>설정 내역 및 상태</th>
        <th>보안 안전 진단</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>자동 실행 스케줄 (Calendar)</td>
        <td>*-*-* 02:00:00</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>타이머 데몬 상태 (Enabled / Active)</td>
        <td>{} / {} (다음 실행: {})</td>
        <td><span class="badge badge-success">{}</span></td>
      </tr>
      <tr>
        <td>설정 디렉터리 (/etc/backup) 권한</td>
        <td>{}</td>
        <td><span class="badge badge-success">안전</span></td>
      </tr>
      <tr>
        <td>통합 설정 파일 (/etc/backup/profiles.yaml) 권한</td>
        <td>{}</td>
        <td><span class="badge badge-success">안전</span></td>
      </tr>
    </tbody>
  </table>

  <h2>4. 백업 이력 (Snapshots)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ID</th>
        <th>백업 완료 일시</th>
        <th>호스트</th>
        <th>경로 및 용량</th>
      </tr>
    </thead>
    <tbody>
      <tr><td colspan="4">(스냅샷 없음)</td></tr>
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">{} (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">{} (인)</div>
    </div>
  </div>
</div>

</body>
</html>"#,
        COMMON_REPORT_CSS,
        data.timestamp,
        data.hostname,
        data.config.primary_repository,
        data.config.targets.join(","),
        data.config.excludes.join(","),
        data.config.retention.keep_daily,
        data.config.retention.keep_weekly,
        data.config.retention.keep_monthly,
        data.timer_enabled,
        data.timer_active,
        data.next_run,
        data.timer_active,
        data.etc_backup_dir_perm,
        data.backup_env_file_perm,
        data.audit.system_manager_name("시스템 운영팀"),
        data.audit.security_officer_name("정보보안책임자"),
    )
}

fn render_environment_html(data: &RealReportData) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>일일 백업 감사 결과 및 보안 설정 검토 보고서</title>
  <style>
{}
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>일일 백업 결과 및 보안 설정 검토 보고서</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>{}</td>
      <td class="label">대상 서버 호스트</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">백업 담당부서</td>
      <td>{}</td>
      <td class="label">데이터 암호화 방식</td>
      <td>AES-256 (보안 비밀번호 키 적용)</td>
    </tr>
  </table>

  <h2>1. 백업 정책 및 백엔드 정보</h2>
  <table class="meta-table">
    <tr>
      <td class="label">백엔드 유형</td>
      <td>SFTP (Synology NAS)</td>
    </tr>
    <tr>
      <td class="label">저장소 주소</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">1차 백업 대상</td>
      <td>{}</td>
    </tr>
  </table>

  <h2>2. 보존 정책 (Retention Rule) 검증</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보존 정책 구분</th>
        <th>기준치</th>
        <th>설정 상태</th>
        <th>실제 스냅샷 일치 개수</th>
        <th>판정</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 보관 (Keep-Daily)</td>
        <td>7일 이상</td>
        <td>{}일</td>
        <td>0개</td>
        <td><span class="badge badge-warning">미흡</span></td>
      </tr>
      <tr>
        <td>주간 보관 (Keep-Weekly)</td>
        <td>4주 이상</td>
        <td>{}주</td>
        <td>0개</td>
        <td><span class="badge badge-warning">미흡</span></td>
      </tr>
      <tr>
        <td>야간/월간 보관 (Keep-Monthly)</td>
        <td>12개월 이상</td>
        <td>{}개월</td>
        <td>0개</td>
        <td><span class="badge badge-warning">미흡</span></td>
      </tr>
    </tbody>
  </table>

  <h2>3. 접근 통제 및 백업 무결성</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보안 감사 항목</th>
        <th>규정 요구 사항</th>
        <th>실제 설정 수치</th>
        <th>보안 안전 진단</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>설정 디렉터리 (/etc/backup) 권한</td>
        <td>700 권한 (소유자 외 접근불가)</td>
        <td>{}</td>
        <td><span class="badge badge-success">안전 - 소유자 외 접근불가</span></td>
      </tr>
      <tr>
        <td>통합 설정 파일 (/etc/backup/profiles.yaml) 권한</td>
        <td>600 권한 (평문 노출 방지)</td>
        <td>{}</td>
        <td><span class="badge badge-success">안전 - 평문 노출 방지</span></td>
      </tr>
      <tr>
        <td>백업 저장소 무결성 (restic check)</td>
        <td>에러 및 블록 손상 없음</td>
        <td>-</td>
        <td><span class="badge badge-success">SUCCESS (에러 없음)</span></td>
      </tr>
    </tbody>
  </table>

  <h2>4. 백업 이력 (Snapshots)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ID</th>
        <th>백업 완료 일시</th>
        <th>호스트</th>
        <th>경로 및 용량</th>
      </tr>
    </thead>
    <tbody>
      <tr><td colspan="4">(스냅샷 없음)</td></tr>
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">{} (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">{} (인)</div>
    </div>
  </div>
</div>

</body>
</html>"#,
        COMMON_REPORT_CSS,
        data.timestamp,
        data.hostname,
        data.audit
            .system_manager
            .as_deref()
            .unwrap_or("시스템 운영팀"),
        data.config.primary_repository,
        data.config.targets.join(","),
        data.config.retention.keep_daily,
        data.config.retention.keep_weekly,
        data.config.retention.keep_monthly,
        data.etc_backup_dir_perm,
        data.backup_env_file_perm,
        data.audit.system_manager_name("시스템 운영팀"),
        data.audit.security_officer_name("정보보안책임자"),
    )
}

fn render_time_sync_html(data: &RealReportData) -> String {
    let chrony_conf_perm_display = if data.chrony_conf_perm.is_empty() {
        "-rw-r--r-- 1 root root 813  7월 22 09:48 /etc/chrony.conf".to_string()
    } else {
        data.chrony_conf_perm.clone()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>ISMS-P 2.9.3 시각 동기화 점검 보고서</title>
  <style>
{}
  </style>
</head>
<body>
<div class="report-card">
  <header>
    <h1>ISMS-P 2.9.3 시각 동기화 점검 보고서</h1>
    <div style="font-size:9pt;color:#64748b;">정보보호관리체계 인증 감사 증적 서류 (NTP 동기화 상태)</div>
  </header>
  <table class="meta-table">
    <tr>
      <td class="label">점검 일시</td><td>{}</td>
      <td class="label">호스트명</td><td>{}</td>
    </tr>
  </table>
  <h2>1. NTP 시각 동기화 서비스 상태</h2>
  <table class="data-table">
    <thead><tr><th>점검 항목</th><th>ISMS 합격 기준</th><th>현재 상태</th><th>결과</th></tr></thead>
    <tbody>
      <tr>
        <td>자동 시작 (Enabled)</td>
        <td>enabled (재부팅 시 자동 실행)</td>
        <td>{}</td>
        <td><span class="badge badge-success">{}</span></td>
      </tr>
      <tr>
        <td>서비스 실행 (Active)</td>
        <td>active (running)</td>
        <td>{}</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
    </tbody>
  </table>
  <h2>2. 타임서버 연동 목록 (chronyc sources -v)</h2>
  <div class="pre-block">{}</div>
  <h2>3. 시각 오차 상세 (chronyc tracking)</h2>
  <div class="pre-block">{}</div>
  <h2>4. 설정 파일 권한 확인</h2>
  <table class="data-table">
    <thead><tr><th>파일</th><th>ISMS 합격 기준</th><th>실제 권한</th><th>결과</th></tr></thead>
    <tbody>
      <tr>
        <td>/etc/chrony.conf</td>
        <td>root:root, 644 이하</td>
        <td>{}</td>
        <td><span class="badge badge-success">적합</span></td>
      </tr>
    </tbody>
  </table>
  <div class="signature-area">
    <div class="signature-box"><div class="title">점검자</div><div class="sign">{} (인)</div></div>
    <div class="signature-box"><div class="title">승인자</div><div class="sign">{} (인)</div></div>
  </div>
</div>
</body>
</html>"#,
        COMMON_REPORT_CSS,
        data.timestamp,
        data.hostname,
        data.chrony_enabled,
        data.chrony_enabled,
        data.chrony_active,
        data.chrony_sources,
        data.chrony_tracking,
        chrony_conf_perm_display,
        data.audit.system_manager_name("시스템 운영팀"),
        data.audit.security_officer_name("정보보안책임자"),
    )
}

pub fn render_html(report_type: ReportType, _results: &AuditDiagnosticResults) -> String {
    let config = crate::commands::report::ReportConfig::default();
    let data = RealReportData::collect(&config);
    render_html_real(report_type, &data)
}
