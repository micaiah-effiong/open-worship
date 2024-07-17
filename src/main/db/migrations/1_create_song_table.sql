-- UP
CREATE TABLE IF NOT EXISTS "t_songs" (
  "id" integer not null primary key autoincrement,
  "title" text not null
);


CREATE TABLE IF NOT EXISTS "t_song_verses" (
  "id" integer not null primary key autoincrement,
  "verse" integer not null,
  "song_id" integer not null,
  "text" text not null,
  FOREIGN KEY(song_id) REFERENCES t_songs(id)
);


-- DOWN
DROP TABLE "t_songs";
DROP TABLE "t_song_verses";
