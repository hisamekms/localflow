CREATE TABLE IF NOT EXISTS metadata_fields (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    field_type TEXT NOT NULL,
    required_on_complete BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    UNIQUE(project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_metadata_fields_project_id ON metadata_fields(project_id)
