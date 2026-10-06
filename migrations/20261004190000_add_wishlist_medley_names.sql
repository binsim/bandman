ALTER TABLE wishlist_medleys
    ADD COLUMN name TEXT NOT NULL DEFAULT 'Medley'
    CHECK (char_length(btrim(name)) BETWEEN 1 AND 120);
