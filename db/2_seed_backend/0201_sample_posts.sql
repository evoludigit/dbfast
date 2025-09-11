-- Sample blog posts for backend testing
INSERT INTO blog.tb_post (
    pk_post,
    pk_author_user,
    title,
    slug,
    content,
    excerpt,
    is_published,
    is_featured,
    published_at,
    view_count,
    created_by,
    updated_by
) VALUES
(
    '01021101-0000-0001-0001-000000000001', -- tb_post(010211) + dir(01) + general(0000) + backend_posts(0001) + rust_post(0001-000000000001)
    '01011101-0000-0002-0001-000000000002', -- john_doe
    'Getting Started with Rust Programming',
    'getting-started-rust-programming',
    '# Getting Started with Rust Programming

Rust is a systems programming language that focuses on safety, speed, and concurrency. In this post, we''ll explore the basics of Rust programming and why it''s becoming increasingly popular.

## Why Rust?

1. **Memory Safety**: Rust prevents common programming errors like null pointer dereferences
2. **Performance**: Zero-cost abstractions mean you don''t pay for what you don''t use
3. **Concurrency**: Built-in support for safe concurrent programming

## Hello, World!

Let''s start with a simple example:

```rust
fn main() {
    println!("Hello, world!");
}
```

This is the traditional first program in any language. In Rust, the `main` function is the entry point of your program.',
    'Learn the fundamentals of Rust programming language, from basic syntax to advanced concepts.',
    true,
    true,
    now() - interval '2 days',
    156,
    '01011101-0000-0002-0001-000000000002', -- created by john_doe
    '01011101-0000-0002-0001-000000000002'
),
(
    '01021101-0000-0001-0002-000000000002', -- tb_post(010211) + dir(01) + general(0000) + backend_posts(0001) + ui_design_post(0002-000000000002)
    '01011101-0000-0002-0002-000000000003', -- jane_smith
    'The Future of User Interface Design',
    'future-user-interface-design',
    '# The Future of User Interface Design

User interface design is constantly evolving. As we move forward, several trends are shaping how we interact with digital products.

## Key Trends

### 1. Voice Interfaces
Voice-controlled interfaces are becoming more sophisticated and natural.

### 2. Gesture Control
Touch-free interaction through gesture recognition.

### 3. Augmented Reality
Blending digital content with the physical world.

## Conclusion

The future of UI design is about making technology more human and intuitive.',
    'Exploring emerging trends in user interface design and their impact on user experience.',
    true,
    false,
    now() - interval '1 day',
    89,
    '01011101-0000-0002-0002-000000000003', -- created by jane_smith
    '01011101-0000-0002-0002-000000000003'
),
(
    '01021101-0000-0001-0003-000000000003', -- tb_post(010211) + dir(01) + general(0000) + backend_posts(0001) + tech_docs_post(0003-000000000003)
    '01011101-0000-0002-0003-000000000004', -- tech_writer
    'Writing Better Technical Documentation',
    'writing-better-technical-documentation',
    '# Writing Better Technical Documentation

Good documentation is crucial for any software project. Here''s how to write documentation that developers actually want to read.

## Principles of Good Documentation

1. **Start with the user''s perspective**
2. **Keep it simple and clear**
3. **Provide working examples**
4. **Keep it up to date**

## Structure Your Content

Use a consistent structure:
- Overview
- Quick start guide
- Detailed API reference
- Troubleshooting

Remember: documentation is a product too!',
    'Learn how to create technical documentation that developers love to read and use.',
    false, -- draft
    false,
    null,
    0,
    '01011101-0000-0002-0003-000000000004', -- created by tech_writer
    '01011101-0000-0002-0003-000000000004'
) ON CONFLICT (pk_post) DO NOTHING;
