#!/bin/bash

# Skip these default databases and roles
DEFAULT_DBS=("postgres" "template0" "template1")
DEFAULT_USERS=("postgres")

# Convert to grep pattern
DEFAULT_DB_PATTERN=$(IFS="|"; echo "${DEFAULT_DBS[*]}")
DEFAULT_USER_PATTERN=$(IFS="|"; echo "${DEFAULT_USERS[*]}")

# Drop all non-default databases
echo "Dropping all user-created databases..."
sudo -u postgres psql -Atc "SELECT datname FROM pg_database WHERE datistemplate = false;" | \
  grep -Ev "^($DEFAULT_DB_PATTERN)$" | \
  while read db; do
    echo "Dropping database: $db"
    sudo -u postgres psql -c "DROP DATABASE IF EXISTS \"$db\";"
  done

# Drop all non-default users
echo "Dropping all user-created roles..."
sudo -u postgres psql -Atc "SELECT rolname FROM pg_roles;" | \
  grep -Ev "^($DEFAULT_USER_PATTERN)$" | \
  while read user; do
    echo "Dropping user: $user"
    sudo -u postgres psql -c "DROP ROLE IF EXISTS \"$user\";"
  done

