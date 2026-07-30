use crate::commands::report::{AuditDiagnosticResults, ReportType};

pub fn render_html(report_type: ReportType, results: &AuditDiagnosticResults) -> String {
    let title = match report_type {
        ReportType::All => "종합 백업 보안 설정 검토 보고서",
        ReportType::Environment => "일일 백업 결과 및 보안 설정 검토 보고서",
        ReportType::TimeSync => "ISMS-P 2.9.3 시각 동기화 점검 보고서",
        ReportType::RestoreDrill => "백업 데이터 복구 및 정합성 테스트 결과 보고서",
    };

    let status_badge_class = if results.overall_pass { "badge-success" } else { "badge-warning" };
    let status_badge_text = if results.overall_pass { "안전 / PASS" } else { "미흡 / FAIL" };

    let mut rows = String::new();
    for item in &results.items {
        let item_badge = if item.pass {
            r#"<span class="badge badge-success">적합 / PASS</span>"#
        } else {
            r#"<span class="badge badge-warning">미흡 / FAIL</span>"#
        };
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            item.name, item.criterion, item.result, item_badge
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>{}</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {{
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }}
    .report-card {{
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }}
    header {{
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }}
    h1 {{
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }}
    .meta-table {{
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }}
    .meta-table td {{
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }}
    .meta-table td.label {{
      background-color: #f1f5f9;
      font-weight: 600;
      width: 20%;
    }}
    h2 {{
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #0f172a;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }}
    .data-table {{
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }}
    .data-table th, .data-table td {{
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }}
    .data-table th {{
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }}
    .badge {{
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }}
    .badge-success {{
      background-color: #dcfce7;
      color: #15803d;
    }}
    .badge-warning {{
      background-color: #fee2e2;
      color: #b91c1c;
    }}
    .signature-area {{
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }}
    .signature-box {{
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }}
    .signature-box .title {{
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }}
    .signature-box .sign {{
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }}
    @media print {{
      @page {{
        size: A4;
        margin: 12mm 15mm 12mm 15mm;
      }}
      body {{
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8.5pt;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }}
      .report-card {{
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }}
      .data-table th, .data-table td {{
        padding: 5px 7px;
        font-size: 8pt;
      }}
      .meta-table td {{
        padding: 5px 8px;
        font-size: 8.5pt;
      }}
      h1 {{
        font-size: 14pt;
      }}
      h2 {{
        font-size: 10pt;
        margin: 14px 0 7px 0;
      }}
      .badge {{
        font-size: 7.5pt;
        padding: 1px 5px;
      }}
      .signature-area {{
        margin-top: 18px;
      }}
    }}
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>{}</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>{}</td>
      <td class="label">대상 서버 호스트</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">종합 보안 상태</td>
      <td colspan="3"><span class="badge {}">{}</span></td>
    </tr>
  </table>

  <h2>점검 항목 및 무결성 진단 내역</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ISMS 보안 감사 항목</th>
        <th>점검 기준</th>
        <th>실제 측정 결과</th>
        <th>보안 판정</th>
      </tr>
    </thead>
    <tbody>
{}
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">시스템 운영팀 (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">정보보안책임자 (서명생략)</div>
    </div>
  </div>
</div>

</body>
</html>"#,
        title, title, results.timestamp, results.host_name, status_badge_class, status_badge_text, rows.trim_end()
    )
}
