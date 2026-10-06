CREATE TABLE wishlist_feedback (
    wishlist_item_id UUID NOT NULL REFERENCES wishlist_items(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (category IN ('song', 'medley', 'try', 'unplayable')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (wishlist_item_id, member_id)
);
