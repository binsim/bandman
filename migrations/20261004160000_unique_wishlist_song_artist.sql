CREATE TEMP TABLE wishlist_item_dedup ON COMMIT DROP AS
SELECT
    id AS item_id,
    first_value(id) OVER (
        PARTITION BY lower(btrim(title)), lower(btrim(coalesce(artist, '')))
        ORDER BY created_at, id
    ) AS canonical_id
FROM wishlist_items;

WITH ranked_feedback AS (
    SELECT
        dedup.canonical_id,
        feedback.member_id,
        feedback.category,
        feedback.created_at,
        feedback.updated_at,
        row_number() OVER (
            PARTITION BY dedup.canonical_id, feedback.member_id
            ORDER BY feedback.updated_at DESC, feedback.created_at DESC
        ) AS feedback_rank
    FROM wishlist_feedback AS feedback
    JOIN wishlist_item_dedup AS dedup ON dedup.item_id = feedback.wishlist_item_id
)
INSERT INTO wishlist_feedback
    (wishlist_item_id, member_id, category, created_at, updated_at)
SELECT canonical_id, member_id, category, created_at, updated_at
FROM ranked_feedback
WHERE feedback_rank = 1
ON CONFLICT (wishlist_item_id, member_id)
DO UPDATE SET
    category = EXCLUDED.category,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

DELETE FROM wishlist_items AS item
USING wishlist_item_dedup AS dedup
WHERE item.id = dedup.item_id
  AND dedup.item_id <> dedup.canonical_id;

CREATE UNIQUE INDEX idx_wishlist_items_title_artist_unique
ON wishlist_items (lower(btrim(title)), lower(btrim(coalesce(artist, ''))));
