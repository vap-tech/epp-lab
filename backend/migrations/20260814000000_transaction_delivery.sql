ALTER TABLE epp_transactions
    ADD COLUMN delivery_status text NOT NULL DEFAULT 'unknown',
    ADD COLUMN delivery_error text;

ALTER TABLE epp_transactions
    ADD CONSTRAINT epp_transactions_delivery_status_check
    CHECK (delivery_status IN ('delivered', 'failed', 'unknown'));

CREATE INDEX epp_transactions_delivery_status_idx
    ON epp_transactions (delivery_status);
