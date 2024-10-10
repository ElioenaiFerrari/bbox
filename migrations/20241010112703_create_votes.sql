-- pub struct Vote {
--     pub id: String,
--     pub voter_id: String,
--     pub candidature_id: String,
--     pub voted_at: chrono::DateTime<chrono::Utc>,
-- }
CREATE TABLE votes (
  id UUID PRIMARY KEY,
  voter_id UUID NULL REFERENCES voters(id),
  candidature_id UUID NULL REFERENCES candidatures(id),
  candidature_position VARCHAR(255) NOT NULL,
  year INTEGER NOT NULL,
  hash TEXT NOT NULL,
  previous_hash TEXT REFERENCES votes(hash),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX votes_hash ON votes (hash);

-- INSERT GENESIS VOTE
INSERT INTO
  votes (
    id,
    candidature_position,
    hash,
    previous_hash,
    year
  )
VALUES
  (
    '00000000-0000-0000-0000-000000000000',
    'GENESIS',
    'GENESIS',
    'GENESIS',
    2024
  );