# 0011. Fix the Database E2E support matrix to production versions

## Status

Accepted

## Decision

Container E2E Matrix는 MariaDB 12 LTS, MariaDB 5.5.56, PostgreSQL 16을 각각 하나의 실제 Database Stream·복원 시나리오로 실행한다. 이전 설계의 MariaDB 10.11/10.6 조합은 이 프로덕션 지원 매트릭스로 대체한다.
