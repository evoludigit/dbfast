-- Insert admin user for the blog
INSERT INTO blog.tb_user (
    pk_user,
    username,
    email,
    password_hash,
    first_name,
    last_name,
    bio,
    is_active,
    is_verified,
    created_by,
    updated_by
) VALUES (
    '01011101-0000-0001-0001-000000000001', -- tb_user(010111) + dir(01) + general(0000) + admin_scenario(0001) + first_admin(0001-000000000001)
    'admin',
    'admin@blog.example',
    '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewfBQQk1mVMbY0VK', -- password: admin123
    'Blog',
    'Administrator',
    'System administrator for the blog platform.',
    true,
    true,
    '01011101-0000-0001-0001-000000000001', -- self-created
    '01011101-0000-0001-0001-000000000001'
) ON CONFLICT (pk_user) DO NOTHING;
