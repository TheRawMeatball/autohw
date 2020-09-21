-- Your SQL goes here
CREATE TABLE classes (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    pwhs VARCHAR(256) NOT NULL ,
    class_id INTEGER,
    FOREIGN KEY (class_id)
        REFERENCES classes (id)
	ON DELETE NO ACTION
	ON UPDATE NO ACTION

);

CREATE TABLE homework (
    id SERIAL PRIMARY KEY,
    due_date DATE,
    detail TEXT NOT NULL,
    amount SMALLINT NOT NULL,
    day_of_week INTEGER,
    class_id INTEGER,
    user_id INTEGER,
    FOREIGN KEY (class_id)
        REFERENCES classes (id)
	    ON DELETE NO ACTION
	    ON UPDATE NO ACTION,
    FOREIGN KEY (user_id)
        REFERENCES users (id) 
	    ON DELETE NO ACTION
	    ON UPDATE NO ACTION
);

CREATE TABLE hw_progress (
    homework_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    progress SMALLINT NOT NULL,
    PRIMARY KEY (homework_id, user_id),
    FOREIGN KEY (homework_id)
	REFERENCES homework (id)
	    ON DELETE CASCADE
	    ON UPDATE NO ACTION,
    FOREIGN KEY (user_id)
	REFERENCES users (id)
	    ON DELETE CASCADE
	    ON UPDATE NO ACTION    
);

CREATE TABLE cancels (
    cancel_id SERIAL PRIMARY KEY,
    homework_id INTEGER NOT NULL,
    on_date DATE NOT NULL,
    FOREIGN KEY (homework_id)
        REFERENCES homework (id)
	    ON DELETE CASCADE
	    ON UPDATE NO ACTION
);
