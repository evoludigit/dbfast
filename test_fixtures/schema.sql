-- Test schema for template creation
CREATE TABLE IF NOT EXISTS tb_user (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tb_post (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES tb_user(id),
    title VARCHAR(200) NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tb_comment (
    id SERIAL PRIMARY KEY,
    post_id INTEGER REFERENCES tb_post(id),
    user_id INTEGER REFERENCES tb_user(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);