-- Natural keys needed for idempotent upserts done by the f1db seeder.
ALTER TABLE race_weekend ADD CONSTRAINT race_weekend_year_round_unique UNIQUE (year, round);
ALTER TABLE session ADD CONSTRAINT session_weekend_type_start_unique UNIQUE (weekend_id, session_type, start_time);
