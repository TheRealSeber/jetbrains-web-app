-- Add migration script here
CREATE TABLE blog_posts (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    image_path TEXT,
    username VARCHAR(255) NOT NULL,
    user_avatar_path TEXT
);