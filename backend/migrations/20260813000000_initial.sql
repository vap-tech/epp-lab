CREATE TABLE registrars (
    id uuid PRIMARY KEY,
    handle text NOT NULL UNIQUE,
    name text NOT NULL,
    client_id text NOT NULL UNIQUE,
    password_hash text NOT NULL,
    status text NOT NULL CHECK (status IN ('active', 'disabled')),
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL
);

CREATE TABLE registrar_certificates (
    id uuid PRIMARY KEY,
    registrar_id uuid NOT NULL REFERENCES registrars(id),
    fingerprint_sha256 text NOT NULL UNIQUE,
    subject text NOT NULL,
    serial_number text,
    not_before timestamptz NOT NULL,
    not_after timestamptz NOT NULL,
    status text NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at timestamptz NOT NULL
);

CREATE TABLE epp_sessions (
    id uuid PRIMARY KEY,
    registrar_id uuid REFERENCES registrars(id),
    certificate_id uuid REFERENCES registrar_certificates(id),
    remote_addr text NOT NULL,
    connected_at timestamptz NOT NULL,
    authenticated_at timestamptz,
    disconnected_at timestamptz,
    disconnect_reason text
);

CREATE TABLE epp_transactions (
    id uuid PRIMARY KEY,
    session_id uuid NOT NULL REFERENCES epp_sessions(id),
    registrar_id uuid REFERENCES registrars(id),
    command text NOT NULL,
    cl_trid text,
    sv_trid text NOT NULL,
    request_xml text NOT NULL,
    response_xml text,
    response_code integer,
    started_at timestamptz NOT NULL,
    finished_at timestamptz,
    duration_ms bigint
);

CREATE INDEX epp_sessions_registrar_id_idx ON epp_sessions (registrar_id);
CREATE INDEX epp_transactions_session_id_idx ON epp_transactions (session_id);
CREATE INDEX epp_transactions_registrar_id_idx ON epp_transactions (registrar_id);
CREATE INDEX epp_transactions_started_at_idx ON epp_transactions (started_at);
CREATE INDEX epp_transactions_sv_trid_idx ON epp_transactions (sv_trid);
CREATE INDEX epp_transactions_cl_trid_idx ON epp_transactions (cl_trid);
