CREATE INDEX idx_tasks_metadata_gin ON tasks USING GIN ((metadata::jsonb)) WHERE metadata IS NOT NULL;
