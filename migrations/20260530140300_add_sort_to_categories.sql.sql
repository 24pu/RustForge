DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'categories' AND column_name = 'sort'
    ) THEN
        ALTER TABLE categories ADD COLUMN sort INTEGER DEFAULT 0;
        UPDATE categories SET sort = id;
    END IF;
END $$;