CREATE TABLE
    ticket_meta (
        ticket_meta_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        order_id TEXT NOT NULL UNIQUE,
        event_id TEXT NOT NULL,
        event_title TEXT NOT NULL,
        session_id TEXT NOT NULL,
        session_starts_at TIMESTAMPTZ NOT NULL,
        venue_name TEXT NOT NULL,
        venue_city TEXT NOT NULL,
        customer_id TEXT NOT NULL,
        customer_email TEXT NOT NULL,
        customer_name TEXT 
    );

CREATE TABLE
    ticket (
        ticket_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        meta_id UUID NOT NULL REFERENCES ticket_meta (ticket_meta_id),
        order_line_id TEXT NOT NULL,
        sector_id TEXT NOT NULL,
        sector_name TEXT NOT NULL,
        seat_id TEXT NOT NULL,
        seat_label TEXT NOT NULL,
        seat_row TEXT, -- Teniamo text per lasciare libertà (row potrebbe essere una lettera o un numero)
        seat_number INTEGER,
        issued_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        status TEXT NOT NULL DEFAULT 'UNSCANNED' CHECK (status IN ('UNSCANNED', 'SCANNED', 'REVOKED'))
    )