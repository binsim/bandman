CREATE TABLE wishlist_medley_feedback (
    medley_id UUID NOT NULL REFERENCES wishlist_medleys(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (category IN (
        'song',
        'medley',
        'try',
        'unplayable',
        'other_ordering',
        'not_fit_medley'
    )),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (medley_id, member_id)
);
