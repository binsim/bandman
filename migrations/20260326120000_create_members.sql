-- Members who log in by display name (no email / passwords for now).
CREATE TABLE IF NOT EXISTS members (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_members_active_name ON members (active, name);

-- Seed a default admin so the app is usable after first migrate.
INSERT INTO members (id, name, role, active)
VALUES ('00000000-0000-4000-8000-000000000001', 'Admin', 'admin', TRUE)
ON CONFLICT (name) DO NOTHING;
