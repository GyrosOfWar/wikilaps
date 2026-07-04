CREATE TABLE race_weekend (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    year INT NOT NULL,
    location VARCHAR NOT NULL,
    circuit_name VARCHAR NOT NULL,
    country_key VARCHAR NOT NULL,
    start_date DATE NOT NULL,
    round INT NOT NULL
);

CREATE TYPE session_type AS ENUM ('FreePractice', 'SprintQualification', 'SprintRace', 'Qualifying', 'Race');

CREATE TABLE session (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    weekend_id BIGINT REFERENCES race_weekend (id) NOT NULL,
    session_type session_type NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ
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
