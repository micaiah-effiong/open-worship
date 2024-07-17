-- UP
CREATE TABLE IF NOT EXISTS "t_bible_kjv" (
  "id" integer not null primary key autoincrement,
  "book" integer not null,
  "chapter" integer not null,
  "verse" integer not null,
  "text" text not null
);

CREATE INDEX "ixb" on "t_bible_kjv" ("book");
CREATE INDEX "ixc" on "t_bible_kjv" ("chapter");
CREATE INDEX "ixv" on "t_bible_kjv" ("verse");
CREATE INDEX "ixbcv" on "t_bible_kjv" ("book", "chapter", "verse");

-- DOWN
DROP INDEX "ixb";
DROP INDEX "ixc";
DROP INDEX "ixv";
DROP INDEX "ixbcv";
DROP TABLE "t_bible_kjv";
