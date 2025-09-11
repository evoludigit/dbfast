-- Sample comments for backend testing
INSERT INTO blog.tb_comment (
    pk_comment,
    pk_post,
    pk_author_user,
    pk_parent_comment,
    content,
    is_approved,
    is_spam,
    created_by,
    updated_by
) VALUES
(
    '01031101-0000-0001-0001-000000000001', -- tb_comment(010311) + dir(01) + general(0000) + backend_comments(0001) + first_comment(0001-000000000001)
    '01021101-0000-0001-0001-000000000001', -- rust_post
    '01011101-0000-0002-0002-000000000003', -- jane_smith
    null, -- top-level comment
    'Great introduction to Rust! I''ve been wanting to learn systems programming and this looks like a perfect starting point.',
    true,
    false,
    '01011101-0000-0002-0002-000000000003', -- created by jane_smith
    '01011101-0000-0002-0002-000000000003'
),
(
    '01031101-0000-0001-0002-000000000002', -- tb_comment(010311) + dir(01) + general(0000) + backend_comments(0001) + reply_comment(0002-000000000002)
    '01021101-0000-0001-0001-000000000001', -- rust_post
    '01011101-0000-0002-0001-000000000002', -- john_doe (author replying)
    '01031101-0000-0001-0001-000000000001', -- reply to first comment
    'Thanks Jane! I''m planning to write more Rust tutorials. What topics would you like to see covered next?',
    true,
    false,
    '01011101-0000-0002-0001-000000000002', -- created by john_doe
    '01011101-0000-0002-0001-000000000002'
),
(
    '01031101-0000-0001-0003-000000000003', -- tb_comment(010311) + dir(01) + general(0000) + backend_comments(0001) + guest_comment(0003-000000000003)
    '01021101-0000-0001-0002-000000000002', -- ui_design_post
    null, -- guest comment
    null, -- top-level comment
    'Voice interfaces are definitely the future! I''ve been experimenting with them in my projects.',
    true,
    false,
    '01011101-0000-0001-0001-000000000001', -- approved by admin
    '01011101-0000-0001-0001-000000000001'
),
(
    '01031101-0000-0001-0004-000000000004', -- tb_comment(010311) + dir(01) + general(0000) + backend_comments(0001) + spam_comment(0004-000000000004)
    '01021101-0000-0001-0001-000000000001', -- rust_post
    null, -- guest spam
    null, -- top-level comment
    'Check out this amazing deal on programming courses! Click here now!!!',
    false, -- not approved
    true, -- marked as spam
    '01011101-0000-0001-0001-000000000001', -- reviewed by admin
    '01011101-0000-0001-0001-000000000001'
) ON CONFLICT (pk_comment) DO NOTHING;

-- Update guest comment with author info (since pk_author_user is null)
UPDATE blog.tb_comment
SET author_name = 'Alex Developer', author_email = 'alex@example.com', user_ip = '192.168.1.100'
WHERE pk_comment = '01031101-0000-0001-0003-000000000003';

-- Update spam comment with guest info
UPDATE blog.tb_comment
SET author_name = 'Spam Bot', author_email = 'spam@spammer.com', user_ip = '10.0.0.1'
WHERE pk_comment = '01031101-0000-0001-0004-000000000004';
