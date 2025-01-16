-- schema.sql

-- NOTE: This is the source-of-truth file for the data schema.
-- When an update is applied, do the following:
--   1. Update this schema.sql file with the new changes
--   2. Create a new migration file in /versions with format:
--      v{NUMBER}__{DESCRIPTION}.sql (e.g., v4__add_email_column.sql)
--   3. In the migration file:
--      - Wrap changes in a transaction
--      - Include both UP migration and DOWN migrations (commented-out)
--          - The DOWN migration is there in-case a rollback is needed
--      - Add version tracking insert
--      - Test locally before committing
--   4. Update version number in schema_migrations table
--   5. Commit both schema.sql and new migration file together
--
-- Example migration file structure:
--   BEGIN;
--   -- UP: Add email column to users
--   ALTER TABLE users ADD COLUMN email TEXT;
--   
--   -- DOWN: Remove email column from users
--   -- ALTER TABLE users DROP COLUMN email;
--   
--   INSERT INTO schema_migrations (version) VALUES (4);
--   COMMIT;

-- Updated whenever migrations are added
-- ============ THIS is the source of truth (!!) ============

CREATE TABLE agent (
    id SERIAL PRIMARY KEY,
    name text NOT NULL,
    email text UNIQUE
);

CREATE TABLE 
