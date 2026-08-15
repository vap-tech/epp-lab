CREATE TABLE domains (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    roid TEXT NOT NULL UNIQUE,
    zone_id UUID NOT NULL REFERENCES zones(id),
    sponsoring_registrar_id UUID NOT NULL REFERENCES registrars(id),
    auth_info_ciphertext TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES registrars(id),
    created_at TIMESTAMPTZ NOT NULL,
    updated_by UUID REFERENCES registrars(id),
    updated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    transferred_at TIMESTAMPTZ
);

CREATE TABLE domain_contacts (
    domain_id UUID NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('registrant', 'admin', 'tech', 'billing')),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    position SMALLINT NOT NULL DEFAULT 1 CHECK (position >= 1),
    PRIMARY KEY (domain_id, role, position)
);

CREATE UNIQUE INDEX domain_contacts_registrant_idx
    ON domain_contacts (domain_id)
    WHERE role = 'registrant';

CREATE INDEX domain_contacts_contact_idx ON domain_contacts (contact_id);
CREATE INDEX domain_contacts_domain_role_idx ON domain_contacts (domain_id, role);

CREATE TABLE domain_nameservers (
    domain_id UUID NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    position SMALLINT NOT NULL CHECK (position >= 1),
    hostname TEXT NOT NULL,
    PRIMARY KEY (domain_id, position),
    UNIQUE (domain_id, hostname)
);

CREATE INDEX domain_nameservers_domain_idx ON domain_nameservers (domain_id);

CREATE TABLE domain_statuses (
    domain_id UUID NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN (
        'clientDeleteProhibited', 'clientHold', 'clientRenewProhibited',
        'clientTransferProhibited', 'clientUpdateProhibited',
        'serverDeleteProhibited', 'serverHold', 'serverRenewProhibited',
        'serverTransferProhibited', 'serverUpdateProhibited'
    )),
    source TEXT NOT NULL CHECK (source IN ('client', 'server')),
    PRIMARY KEY (domain_id, status, source)
);

CREATE INDEX domains_zone_idx ON domains (zone_id);
CREATE INDEX domains_sponsoring_registrar_idx ON domains (sponsoring_registrar_id);
CREATE INDEX domains_created_at_idx ON domains (created_at);
CREATE INDEX domains_expires_at_idx ON domains (expires_at);
CREATE INDEX domain_statuses_domain_idx ON domain_statuses (domain_id);
