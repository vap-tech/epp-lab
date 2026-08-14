CREATE TABLE zones (
    id UUID PRIMARY KEY,
    ascii_name TEXT NOT NULL UNIQUE,
    unicode_name TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT zones_status_check CHECK (status IN ('active', 'disabled'))
);

CREATE TABLE zone_contact_policies (
    zone_id UUID PRIMARY KEY REFERENCES zones(id) ON DELETE RESTRICT,
    registrant_requirement TEXT NOT NULL,
    admin_requirement TEXT NOT NULL,
    tech_requirement TEXT NOT NULL,
    billing_requirement TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT zone_contact_policies_registrant_check
        CHECK (registrant_requirement IN ('forbidden', 'optional', 'required')),
    CONSTRAINT zone_contact_policies_admin_check
        CHECK (admin_requirement IN ('forbidden', 'optional', 'required')),
    CONSTRAINT zone_contact_policies_tech_check
        CHECK (tech_requirement IN ('forbidden', 'optional', 'required')),
    CONSTRAINT zone_contact_policies_billing_check
        CHECK (billing_requirement IN ('forbidden', 'optional', 'required'))
);

CREATE TABLE zone_extensions (
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE RESTRICT,
    extension_key TEXT NOT NULL,
    enabled BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (zone_id, extension_key)
);
