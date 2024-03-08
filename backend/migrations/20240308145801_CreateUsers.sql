-- Add migration script here

CREATE TABLE users
(
    uuid       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR NOT NULL,
    last_name  VARCHAR NOT NULL
)