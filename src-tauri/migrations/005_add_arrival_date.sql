ALTER TABLE incoming ADD COLUMN arrival_date TEXT;
UPDATE incoming SET arrival_date = created_at WHERE arrival_date IS NULL;
