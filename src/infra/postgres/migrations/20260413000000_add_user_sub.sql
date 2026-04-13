ALTER TABLE users ADD COLUMN sub TEXT;
UPDATE users SET sub = username WHERE sub IS NULL;
CREATE UNIQUE INDEX idx_users_sub ON users(sub);
