-- pub struct Party {
--     pub id: String,
--     pub name: String,
--     pub description: String,
--     pub acronym: String,
-- }
CREATE TABLE parties (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  acronym VARCHAR(10) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);