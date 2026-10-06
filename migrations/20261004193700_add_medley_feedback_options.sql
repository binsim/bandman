ALTER TABLE wishlist_feedback
    DROP CONSTRAINT wishlist_feedback_category_check,
    ADD CONSTRAINT wishlist_feedback_category_check
        CHECK (category IN (
            'song',
            'medley',
            'try',
            'unplayable',
            'other_ordering',
            'not_fit_medley'
        ));
