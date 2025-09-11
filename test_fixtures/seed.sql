-- Test seed data for template creation
INSERT INTO tb_user (username, email) VALUES 
    ('admin', 'admin@example.com'),
    ('john_doe', 'john@example.com'),
    ('jane_smith', 'jane@example.com');

INSERT INTO tb_post (user_id, title, content) VALUES 
    (1, 'Welcome to our blog!', 'This is the first post on our blog.'),
    (2, 'My first post', 'Hello world, this is my first blog post!'),
    (3, 'Thoughts on databases', 'PostgreSQL is really powerful for web applications.');

INSERT INTO tb_comment (post_id, user_id, content) VALUES 
    (1, 2, 'Great to see this blog getting started!'),
    (1, 3, 'Looking forward to more posts.'),
    (2, 1, 'Welcome to the community!'),
    (3, 2, 'I agree, PostgreSQL is fantastic.');