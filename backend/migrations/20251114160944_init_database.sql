CREATE TABLE race_weekend (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    year INT NOT NULL,
    location VARCHAR NOT NULL,
    country_key VARCHAR NOT NULL,
    start_date DATE NOT NULL,
    round INT NOT NULL,
    official_name VARCHAR NOT NULL,
    circuit_full_name VARCHAR NOT NULL,
    circuit_id VARCHAR NOT NULL,
    grand_prix_id VARCHAR NOT NULL,
    UNIQUE (year, round)
);

CREATE TYPE session_type AS ENUM ('sprint_qualifying', 'sprint_race', 'qualifying', 'race');

CREATE TABLE session (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    weekend_id BIGINT REFERENCES race_weekend (id) NOT NULL,
    session_type session_type NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    UNIQUE (weekend_id, session_type, start_time)
);

CREATE TYPE vote_type AS ENUM ('FullRace', 'RaceIn30', 'Highlights');

CREATE TABLE votes (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    vote_type vote_type NOT NULL,
    user_identifier VARCHAR NOT NULL,
    session_id BIGINT REFERENCES session (id) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_identifier, session_id)
);
