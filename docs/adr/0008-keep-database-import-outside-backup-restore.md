# 0008. Keep database import outside `backup restore`

## Status

Accepted

## Decision

`backup restore`는 Database Stream의 SQL 파일을 복원 대상에 되살리는 데서 끝난다. Container E2E Matrix는 runner가 그 SQL을 대상 DB에 import하고 행 데이터를 검증한다. 자동 DB import는 운영 데이터 변경 위험이 있는 별도 기능으로 이번 범위에 포함하지 않는다.
