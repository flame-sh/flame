CREATE TABLE IF NOT EXISTS applications (
    name            TEXT NOT NULL,
    image           TEXT,
    url             TEXT,
    command         TEXT,
    shim            INTEGER NOT NULL,

    creation_time   INTEGER NOT NULL,
    state           INTEGER NOT NULL,

    PRIMARY KEY (name)
);

INSERT OR IGNORE INTO applications (name, command, shim, creation_time, state)
    VALUES ('flmping', '/usr/local/flame/bin/flmping-app', 1, strftime ('%s', 'now'), 0);
INSERT OR IGNORE INTO applications (name, shim, creation_time, state)
    VALUES ('flmexec', 0, strftime ('%s', 'now'), 0);