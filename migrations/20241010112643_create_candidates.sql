-- pub struct Candidate {
--     pub id: String,
--     pub first_name: String,
--     pub last_name: String,
-- }
CREATE TABLE candidates (
  id UUID PRIMARY KEY,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);