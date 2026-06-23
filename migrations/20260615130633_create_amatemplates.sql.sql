DROP TABLE IF EXISTS templates CASCADE;

CREATE TABLE IF NOT EXISTS amatemplates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    is_used BOOLEAN DEFAULT false,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_amatemplates_user_id ON amatemplates(user_id);
CREATE INDEX IF NOT EXISTS idx_amatemplates_is_used ON amatemplates(is_used);