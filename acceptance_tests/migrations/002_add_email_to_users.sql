-- rambler up
ALTER TABLE users ADD COLUMN email VARCHAR(255);

-- rambler up
CREATE UNIQUE INDEX idx_users_email ON users (email);

-- rambler down
ALTER TABLE users DROP COLUMN email;

-- rambler down
DROP INDEX idx_users_email;
