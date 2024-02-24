CREATE TABLE listing (
        id INTEGER NOT NULL, 
        date DATE NOT NULL, 
        PRIMARY KEY (id)
);
CREATE TABLE song (
        id INTEGER NOT NULL, 
        title TEXT NOT NULL, 
        artist TEXT NOT NULL, 
        PRIMARY KEY (id)
);
CREATE TABLE position (
        id INTEGER NOT NULL, 
        song_id INTEGER NOT NULL, 
        listing_id INTEGER NOT NULL, 
        position INTEGER NOT NULL, 
        waiting_room BOOLEAN NOT NULL, 
        PRIMARY KEY (id), 
        FOREIGN KEY(song_id) REFERENCES song (id), 
        FOREIGN KEY(listing_id) REFERENCES listing (id)
);