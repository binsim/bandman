CREATE TABLE wishlist_items (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT,
    link TEXT,
    targets TEXT[] NOT NULL DEFAULT '{}',
    category TEXT NOT NULL CHECK (category IN ('song', 'medley', 'try', 'unplayable')),
    proposed_by UUID REFERENCES members(id) ON DELETE SET NULL,
    proposed_by_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wishlist_items_created_at ON wishlist_items (created_at DESC);
