CREATE TABLE IF NOT EXISTS proof_statuses ( 
    proof_status_id SMALLINT NOT NULL,
    request_id UUID NOT NULL,
    status SMALLINT DEFAULT 0, 
    proof JSON,
    PRIMARY KEY (proof_status_id, request_id)
);

CREATE OR REPLACE FUNCTION status_update_notify() RETURNS trigger AS $$
DECLARE
  notification_payload JSON;
BEGIN
  IF (TG_OP = 'UPDATE' AND NEW.status IS DISTINCT FROM OLD.status) OR TG_OP = 'INSERT' THEN
    notification_payload = json_build_object(
      'proof_status_id', NEW.proof_status_id,
      'request_id', NEW.request_id,
      'new_status', NEW.status,
      'proof', NEW.proof
    );

    PERFORM pg_notify('status_update', notification_payload::text);
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER status_update_notify
AFTER UPDATE ON proof_statuses
FOR EACH ROW
EXECUTE PROCEDURE status_update_notify();

CREATE TRIGGER status_insert_notify
AFTER INSERT ON proof_statuses
FOR EACH ROW
EXECUTE PROCEDURE status_update_notify();