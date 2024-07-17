import path from "path";
import sqlite3 from "sqlite3";
import { load_migrations, run_migration } from "./migrations";

const base_db_path = path.resolve(process.cwd(), "src/main/db");
const db_path = path.resolve(base_db_path, "store", "open_worship.sqlite");
const migration_path = path.resolve(base_db_path, "migrations");

export const db = new sqlite3.Database(db_path);

load_migrations(migration_path)
  .then((mig) => run_migration(db, mig))
  .catch(console.error)
  .finally(() => db.close());

// type i_bible_verse = {
//   id: number;
//   text: string;
//   chapter: number;
//   verse: number;
// };

// type i_song = {
//   id: number;
//   title: string;
// };
// type i_song_verse = {
//   id: number;
//   /** @description verse_no is verse number */
//   verse_no: number;
//   song_id: number;
//   text: string;
// };

// db.serialize(async () => {
//   const res = await callback_handler<i_bible_verse[]>((fn) => {
//     db.all<i_bible_verse>(
//       "SELECT * FROM bible_kjv WHERE chapter = 1 AND verse = 1 LIMIT 4",
//       fn,
//     );
//   });
//
//   console.log(res);
// });

// db.close();
// const get_all = new sqlite3.Statement().all<i_bible_verse>(
//   "SELECT * FROM bible_kjv WHERE chapter = 1 AND verse = 1 LIMIT 4",
//   (_err, row) => {
//     if (!_err) {
//       // console.log(row.id + ":\r\n" + row.text, "\r\n");
//       console.log(JSON.stringify(row, null, 2));
//     }
//   },
// );
