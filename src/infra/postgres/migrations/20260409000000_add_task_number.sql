ALTER TABLE tasks ADD COLUMN IF NOT EXISTS task_number BIGINT;
UPDATE tasks SET task_number = id WHERE task_number IS NULL;
ALTER TABLE tasks ALTER COLUMN task_number SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_project_task_number ON tasks(project_id, task_number);
