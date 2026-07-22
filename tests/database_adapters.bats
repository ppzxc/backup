#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "database_mysql_default_command returns mysql dump command" {
  run database_mysql_default_command
  [ "$status" -eq 0 ]
  [[ "$output" == *"mysqldump"* ]]
}

@test "database_mysql_validate_dump validates MySQL signature" {
  run database_mysql_validate_dump "blah blah MySQL dump blah"
  [ "$status" -eq 0 ]
  run database_mysql_validate_dump "some other content"
  [ "$status" -eq 1 ]
}

@test "database_mariadb_default_command returns mariadb dump command" {
  run database_mariadb_default_command
  [ "$status" -eq 0 ]
  [[ "$output" == *"mariadb-dump"* ]]
}

@test "database_mariadb_validate_dump validates MariaDB signature" {
  run database_mariadb_validate_dump "blah blah MariaDB dump blah"
  [ "$status" -eq 0 ]
  run database_mariadb_validate_dump "some other content"
  [ "$status" -eq 1 ]
}

@test "database_postgres_default_command returns pg dumpall command" {
  run database_postgres_default_command
  [ "$status" -eq 0 ]
  [[ "$output" == *"pg_dumpall"* ]]
}

@test "database_postgres_validate_dump validates PostgreSQL signature" {
  run database_postgres_validate_dump "PostgreSQL database dump blah"
  [ "$status" -eq 0 ]
  run database_postgres_validate_dump "some other content"
  [ "$status" -eq 1 ]
}

@test "database_custom_default_command returns empty string" {
  run database_custom_default_command
  [ "$status" -eq 0 ]
  [ "$output" = "" ]
}

@test "database_custom_validate_dump always returns 0" {
  run database_custom_validate_dump "any content"
  [ "$status" -eq 0 ]
}
