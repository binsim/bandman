CREATE TABLE wishlist_categories (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (char_length(btrim(name)) BETWEEN 1 AND 60),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX wishlist_categories_name_unique ON wishlist_categories (lower(name));

CREATE TABLE wishlist_labels (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (char_length(btrim(name)) BETWEEN 1 AND 40),
    color TEXT NOT NULL CHECK (color IN (
        '#2563EB', '#15803D', '#C2410C', '#BE123C', '#7E22CE', '#475569'
    )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX wishlist_labels_name_unique ON wishlist_labels (lower(name));

ALTER TABLE wishlist_items
    ADD COLUMN song_category_id UUID REFERENCES wishlist_categories(id) ON DELETE SET NULL;

ALTER TABLE wishlist_medleys
    ADD COLUMN song_category_id UUID REFERENCES wishlist_categories(id) ON DELETE SET NULL;

CREATE TABLE wishlist_item_labels (
    wishlist_item_id UUID NOT NULL REFERENCES wishlist_items(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES wishlist_labels(id) ON DELETE CASCADE,
    PRIMARY KEY (wishlist_item_id, label_id)
);

CREATE TABLE wishlist_medley_labels (
    medley_id UUID NOT NULL REFERENCES wishlist_medleys(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES wishlist_labels(id) ON DELETE CASCADE,
    PRIMARY KEY (medley_id, label_id)
);
