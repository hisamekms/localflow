-- Create a functional GIN index on the metadata column cast to JSONB.
-- The column stays TEXT; the index enables efficient @> containment queries
-- via the expression (metadata::jsonb).
CREATE INDEX idx_tasks_metadata_gin ON tasks USING GIN ((metadata::jsonb)) WHERE metadata IS NOT NULL;
