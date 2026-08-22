-- Blog-style articles have no use for the quoting apparatus: the reader's
-- sentence-selection layer is noise there. Site admins flip this per
-- article; existing quotations are snapshots and survive the flip.
ALTER TABLE articles
    ADD COLUMN quoting_disabled BOOLEAN NOT NULL DEFAULT false;
