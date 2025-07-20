#!/bin/bash

# Variables — customize as needed
DB_NAME="test_db"
DB_USER="test"
DB_PASS="123456"

# Create DB and user
sudo -u postgres psql <<EOF
CREATE DATABASE $DB_NAME;
CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASS';
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;

\c $DB_NAME

CREATE TABLE users (
  uuid UUID PRIMARY KEY,                             -- unique ID
  email TEXT NOT NULL UNIQUE,                        -- must be unique
  email_verified BOOLEAN NOT NULL DEFAULT false,     -- true if verified
  name TEXT NOT NULL,                                -- optional
  password_hash TEXT,                                -- optional (for Google users)
  google_sub TEXT UNIQUE,                            -- optional, but must be unique if present
  phone_num TEXT NOT NULL UNIQUE,                    -- optional, but must be unique if present
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),     -- time of account creation
  last_seen_at TIMESTAMPTZ,                          -- optional last-seen timestamp
  picture TEXT                                       -- optional avatar URL
);
ALTER TABLE users OWNER TO $DB_USER;

EOF
