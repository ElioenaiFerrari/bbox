-- pub struct Vote {
--     pub id: String,
--     pub voter_id: String,
--     pub candidature_id: String,
--     pub voted_at: chrono::DateTime<chrono::Utc>,
-- }
CREATE TABLE votes (
  id UUID PRIMARY KEY,
  voter_id UUID NOT NULL,
  candidature_id UUID NOT NULL,
  candidature_position VARCHAR(255) NOT NULL,
  hash TEXT NOT NULL,
  previous_hash TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- INSERT GENESIS VOTE
INSERT INTO
  votes (
    id,
    voter_id,
    candidature_id,
    candidature_position,
    hash,
    previous_hash
  )
VALUES
  (
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000'
  );