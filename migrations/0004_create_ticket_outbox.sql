CREATE TABLE ticket_outbox (
    event_id UUID PRIMARY KEY,
    meta_id UUID NOT NULL UNIQUE REFERENCES ticket_meta (ticket_meta_id),
    payload JSONB NOT NULL,
    published_at TIMESTAMPTZ
);