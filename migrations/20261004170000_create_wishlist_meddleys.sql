CREATE TABLE wishlist_meddleys (
    id UUID PRIMARY KEY,
    created_by UUID REFERENCES members(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE wishlist_medley_items (
    medley_id UUID NOT NULL REFERENCES wishlist_meddleys(id) ON DELETE CASCADE,
    wishlist_item_id UUID NOT NULL UNIQUE REFERENCES wishlist_items(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (medley_id, wishlist_item_id),
    UNIQUE (medley_id, position)
);
