ALTER TABLE tasks ADD COLUMN task_number BIGINT;
UPDATE tasks SET task_number = id;
ALTER TABLE tasks ALTER COLUMN task_number SET NOT NULL;
CREATE UNIQUE INDEX idx_tasks_project_task_number ON tasks(project_id, task_number);
