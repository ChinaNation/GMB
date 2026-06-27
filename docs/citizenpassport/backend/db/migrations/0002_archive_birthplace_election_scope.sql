-- CPMS archives: immutable birthplace and encrypted election-region scope.

ALTER TABLE archives
  ADD COLUMN IF NOT EXISTS birth_province_code TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS birth_city_code TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS birth_town_code TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS election_scope_level TEXT NOT NULL DEFAULT 'PROVINCE';

ALTER TABLE archives
  DROP CONSTRAINT IF EXISTS archives_election_scope_level_check;

ALTER TABLE archives
  ADD CONSTRAINT archives_election_scope_level_check
  CHECK (election_scope_level IN ('PROVINCE', 'CITY', 'TOWN'));
