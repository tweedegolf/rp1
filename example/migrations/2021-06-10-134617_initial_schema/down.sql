DROP TRIGGER update_comments_updated_at ON comments;
DROP TABLE comments;

DROP TRIGGER update_posts_updated_at ON posts;
DROP TABLE posts;

DROP TRIGGER update_users_updated_at ON users;
DROP TABLE users;

-- Drop trigger function
DROP FUNCTION update_updated_at_column;
