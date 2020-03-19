CREATE TABLE canonical_shortlinks (
  name VARCHAR(10) PRIMARY KEY,
  target VARCHAR(2048) NOT NULL
);

CREATE UNIQUE INDEX ON canonical_shortlinks (target);
