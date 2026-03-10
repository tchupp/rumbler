-- rambler up
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    body TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- rambler down
DROP TABLE posts;
