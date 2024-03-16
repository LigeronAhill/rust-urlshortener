CREATE TABLE IF NOT EXISTS link_statistics (
    id SERIAL PRIMARY KEY,
    link_id TEXT NOT NULL,
    referer TEXT,
    user_agent TEXT,
    CONSTRAINT fk_link_id FOREIGN KEY (link_id) REFERENCES links(id)
);

CREATE INDEX isx_link_statistics_link_id ON link_statistics USING btree(link_id)