-- Add migration script here

CREATE TABLE users
(
    id         serial PRIMARY KEY NOT NULL,
    first_name VARCHAR            NOT NULL,
    last_name  VARCHAR            NOT NULL
)