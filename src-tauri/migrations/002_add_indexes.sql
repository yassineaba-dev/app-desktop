CREATE INDEX IF NOT EXISTS idx_incoming_date ON incoming(date);
CREATE INDEX IF NOT EXISTS idx_incoming_registration ON incoming(registration_number);
CREATE INDEX IF NOT EXISTS idx_incoming_correspondence ON incoming(correspondence_number);
CREATE INDEX IF NOT EXISTS idx_incoming_subject ON incoming(subject);
CREATE INDEX IF NOT EXISTS idx_incoming_sender ON incoming(sender);
CREATE INDEX IF NOT EXISTS idx_incoming_destination ON incoming(destination_service);
CREATE INDEX IF NOT EXISTS idx_incoming_updated_at ON incoming(updated_at);
CREATE INDEX IF NOT EXISTS idx_incoming_deleted_at ON incoming(deleted_at);
CREATE INDEX IF NOT EXISTS idx_incoming_sync ON incoming(sync_version);

CREATE INDEX IF NOT EXISTS idx_outgoing_date ON outgoing(date);
CREATE INDEX IF NOT EXISTS idx_outgoing_registration ON outgoing(registration_number);
CREATE INDEX IF NOT EXISTS idx_outgoing_correspondence ON outgoing(correspondence_number);
CREATE INDEX IF NOT EXISTS idx_outgoing_subject ON outgoing(subject);
CREATE INDEX IF NOT EXISTS idx_outgoing_recipient ON outgoing(recipient);
CREATE INDEX IF NOT EXISTS idx_outgoing_destination ON outgoing(destination_service);
CREATE INDEX IF NOT EXISTS idx_outgoing_updated_at ON outgoing(updated_at);
CREATE INDEX IF NOT EXISTS idx_outgoing_deleted_at ON outgoing(deleted_at);
CREATE INDEX IF NOT EXISTS idx_outgoing_sync ON outgoing(sync_version);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);
CREATE INDEX IF NOT EXISTS idx_users_sync ON users(sync_version);

CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp);
