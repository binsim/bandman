ALTER TABLE members
    DROP CONSTRAINT IF EXISTS members_role_check;

ALTER TABLE members
    ADD CONSTRAINT members_role_check
    CHECK (role IN ('admin', 'member', 'participant'));
