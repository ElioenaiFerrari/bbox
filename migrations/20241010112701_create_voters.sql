-- pub struct Voter {
--     pub id: String,
--     pub first_name: String,
--     pub last_name: String,
--     pub mother_name: String,
--     pub father_name: String,
--     pub birth_date: String,
-- }
CREATE TABLE voters (
  id UUID PRIMARY KEY,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  mother_name TEXT NOT NULL,
  father_name TEXT NOT NULL,
  birth_date DATE NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);