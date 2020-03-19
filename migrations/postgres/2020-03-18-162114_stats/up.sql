CREATE TABLE stats (
  name VARCHAR(128) PRIMARY KEY,
  created_on TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE visits (
  id SERIAL PRIMARY KEY,
  name VARCHAR(128) references stats(name),
  visit TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  ip_addr VARCHAR(128)
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

CREATE FUNCTION get_stat(VARCHAR(128)) RETURNS TABLE (
  visit_date TIMESTAMP WITH TIME ZONE,
  visit_count BIGINT) AS $$
BEGIN
  RETURN QUERY 
  SELECT date_trunc('day', visit) AS visit_date,
         COUNT(id) AS visit_count 
    FROM visits WHERE visits.name = $1
    GROUP BY visit_date
    ORDER BY visit_date;
END;
$$ LANGUAGE plpgsql;
