-- Sample users for testing
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
) VALUES
(
    '01011101-0000-0002-0001-000000000002', -- tb_user(010111) + dir(01) + general(0000) + sample_users(0002) + john_doe(0001-000000000002)
    'john_doe',
    'john@example.com',
    '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewfBQQk1mVMbY0VK', -- password: test123
    'John',
    'Doe',
    'Software developer and tech blogger.',
    true,
    true,
    '01011101-0000-0001-0001-000000000001', -- created by admin
    '01011101-0000-0001-0001-000000000001'
),
(
    '01011101-0000-0002-0002-000000000003', -- tb_user(010111) + dir(01) + general(0000) + sample_users(0002) + jane_smith(0002-000000000003)
    'jane_smith',
    'jane@example.com',
    '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewfBQQk1mVMbY0VK', -- password: test123
    'Jane',
    'Smith',
    'UX designer with a passion for user-centered design.',
    true,
    true,
    '01011101-0000-0001-0001-000000000001', -- created by admin
    '01011101-0000-0001-0001-000000000001'
),
(
    '01011101-0000-0002-0003-000000000004', -- tb_user(010111) + dir(01) + general(0000) + sample_users(0002) + tech_writer(0003-000000000004)
    'tech_writer',
    'writer@example.com',
    '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewfBQQk1mVMbY0VK', -- password: test123
    'Tech',
    'Writer',
    'Technical writer specializing in developer documentation.',
    true,
    true,
    '01011101-0000-0001-0001-000000000001', -- created by admin
    '01011101-0000-0001-0001-000000000001'
) ON CONFLICT (pk_user) DO NOTHING;
