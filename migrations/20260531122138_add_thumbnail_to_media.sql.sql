DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'media_files' AND column_name = 'thumbnail_path'
    ) THEN
        ALTER TABLE media_files ADD COLUMN thumbnail_path VARCHAR(500);
    END IF;
END $$;