CREATE TABLE contacts (
    id UUID PRIMARY KEY,
    roid TEXT NOT NULL UNIQUE,
    sponsoring_registrar_id UUID NOT NULL REFERENCES registrars(id),
    created_by UUID NOT NULL REFERENCES registrars(id),
    created_at TIMESTAMPTZ NOT NULL,
    updated_by UUID NOT NULL REFERENCES registrars(id),
    updated_at TIMESTAMPTZ NOT NULL,
    transferred_at TIMESTAMPTZ,
    auth_info_ciphertext TEXT NOT NULL,
    disclose_flag TEXT NOT NULL CHECK (disclose_flag IN ('public', 'private'))
);

CREATE TABLE contact_postal_info (
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    info_type TEXT NOT NULL CHECK (info_type IN ('international', 'localized')),
    name TEXT NOT NULL,
    organization TEXT,
    city TEXT NOT NULL,
    state_province TEXT,
    postal_code TEXT,
    country_code CHAR(2) NOT NULL CHECK (country_code ~ '^[A-Z]{2}$'),
    PRIMARY KEY (contact_id, info_type)
);

CREATE TABLE contact_postal_streets (
    contact_id UUID NOT NULL,
    info_type TEXT NOT NULL,
    position SMALLINT NOT NULL CHECK (position BETWEEN 1 AND 3),
    street TEXT NOT NULL,
    PRIMARY KEY (contact_id, info_type, position),
    FOREIGN KEY (contact_id, info_type)
        REFERENCES contact_postal_info(contact_id, info_type) ON DELETE CASCADE
);

CREATE TABLE contact_phones (
    contact_id UUID PRIMARY KEY REFERENCES contacts(id) ON DELETE CASCADE,
    voice TEXT NOT NULL,
    voice_extension TEXT,
    fax TEXT,
    fax_extension TEXT,
    email TEXT NOT NULL
);

CREATE TABLE contact_statuses (
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('client', 'server')),
    PRIMARY KEY (contact_id, status, source)
);

CREATE TABLE contact_disclosure_fields (
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    field TEXT NOT NULL CHECK (field IN ('name', 'organization', 'address', 'voice', 'fax', 'email')),
    PRIMARY KEY (contact_id, field)
);

CREATE INDEX contacts_sponsoring_registrar_idx ON contacts (sponsoring_registrar_id);
CREATE INDEX contact_statuses_contact_idx ON contact_statuses (contact_id);
