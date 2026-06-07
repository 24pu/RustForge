DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'categories' AND column_name = 'show_in_nav'
    ) THEN
        ALTER TABLE categories ADD COLUMN show_in_nav BOOLEAN DEFAULT true;
    END IF;
END $$;