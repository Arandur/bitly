CREATE TABLE shortlinks (
  shortlink VARCHAR(16) PRIMARY KEY,
  target VARCHAR(2048) not null
);

CREATE UNIQUE INDEX ON shortlinks (target);
