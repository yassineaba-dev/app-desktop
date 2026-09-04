-- Add an optional "duplicate" (مكرر) flag for incoming records.
-- The sequential number (registration_number) is stored as a plain numeric value
-- so it can be sorted and searched numerically. The optional "مكرر" label is
-- modelled as a boolean status flag on the record instead of being part of the
-- number itself.
ALTER TABLE incoming ADD COLUMN is_duplicate INTEGER NOT NULL DEFAULT 0;

-- Backfill: existing records that stored the "مكرر" label as a suffix of their
-- sequential number are normalized - the label is moved into the flag and the
-- number is reduced to its pure numeric value. No record is lost; the duplicate
-- status is preserved in the new column.
UPDATE incoming
SET is_duplicate = 1,
    registration_number = TRIM(SUBSTR(registration_number, 1, LENGTH(registration_number) - LENGTH('مكرر')))
WHERE is_duplicate = 0
  AND registration_number LIKE '%مكرر'
  AND LENGTH(registration_number) > LENGTH('مكرر');

-- Guard against a value that is exactly "مكرر" (no number): fall back to a clean value.
UPDATE incoming
SET registration_number = '0'
WHERE registration_number = 'مكرر';
