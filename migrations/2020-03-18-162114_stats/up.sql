CREATE TABLE stats (
  name VARCHAR(128) PRIMARY KEY,
  created_on TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  visits INTEGER NOT NULL DEFAULT 0
);
