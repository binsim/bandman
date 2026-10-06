ALTER TABLE wishlist_items
    ADD COLUMN tempo INTEGER CHECK (tempo BETWEEN 20 AND 300),
    ADD COLUMN musical_key TEXT;
