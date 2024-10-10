-- pub enum Position {
--     President,
--     VicePresident,
--     Governor,
--     ViceGovernor,
--     Senator,
--     FederalDeputy, // Representa deputados federais
--     StateDeputy,   // Representa deputados estaduais
--     Mayor,
--     ViceMayor,
--     Councilor,     // Substitui Councilman e Councilwoman, Alderman e Alderwoman
--     Minister,      // Representa ministros
--     Secretary,     // Representa secretários (por exemplo, de estado)
--     Other(String), // Para incluir cargos não listados
-- }
-- pub struct Candidature {
--     pub id: String,
--     pub party_id: String,
--     pub candidate_id: String,
--     pub code: String,
--     pub position: Position,
-- }
CREATE TABLE candidatures (
  id UUID PRIMARY KEY,
  party_id UUID REFERENCES parties(id),
  candidate_id UUID REFERENCES candidates(id),
  code VARCHAR(10) NOT NULL,
  position VARCHAR(20) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_candidatures_code ON candidatures(code);