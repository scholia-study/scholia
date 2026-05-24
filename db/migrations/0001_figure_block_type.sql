-- Add 'figure' to the block_type enum for diagram-like insertions
-- (e.g. Kant's table of judgments) rendered as verbatim <figure> HTML.

ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'figure';
