-- Add migration script here
CREATE TABLE messages
(
    id               SERIAL PRIMARY KEY NOT NULL,
    sender_id        INTEGER            NOT NULL,
    recipient_id     INTEGER            NOT NULL,
    message_contents TEXT               NOT NULL
)