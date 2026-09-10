-- Ancient works carry an estimated composition date, not an imprint year:
-- the Republic is conventionally dated c. 375 BCE, with a scholarly range
-- of roughly 380-370. `original_year` stays a sortable integer (negative
-- for BCE) so library ordering keeps working; this flag records that the
-- integer is an estimate, so citations can render "ca. 375 BCE".
ALTER TABLE sources
    ADD COLUMN original_year_circa BOOLEAN NOT NULL DEFAULT FALSE;
