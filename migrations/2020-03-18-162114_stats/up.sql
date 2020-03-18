CREATE TABLE stats (
  name VARCHAR(128) PRIMARY KEY,
  created_on TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  visits INTEGER NOT NULL DEFAULT 0
);

CREATE FUNCTION create_stat() RETURNS trigger AS $create_stat$
BEGIN
  INSERT INTO stats (name) VALUES (NEW.name);
  RETURN NEW;
END;
$create_stat$ LANGUAGE plpgsql;

CREATE TRIGGER create_stat BEFORE INSERT ON canonical_shortlinks
  FOR EACH ROW EXECUTE PROCEDURE create_stat();

CREATE TRIGGER create_stat BEFORE INSERT ON custom_shortlinks
  FOR EACH ROW EXECUTE PROCEDURE create_stat();
