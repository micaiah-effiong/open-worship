import { async_handler, Result, unwrap } from "../utils";
import { readdir, readFile } from "fs/promises";
import path from "path";
import assert from "assert";
import { i_database } from "./db";

const FILE_NAME_REGEX = /^(\d+)_(\S+)(\.sql)$/;

type i_migrations = {
  id: number;
  name: string;
  up: string;
  down: string;
};
type i_migration_data = {
  id: number;
  name: string;
  filename: string;
  up: string;
  down: string;
};

export async function run_migration(
  database: i_database,
  migrations: i_migration_data[],
) {
  const migration_table = "t_migrations";

  await database.run(
    `CREATE TABLE IF NOT EXISTS "${migration_table}" (
        id   INTEGER PRIMARY KEY,
        name TEXT    NOT NULL,
        up   TEXT    NOT NULL,
        down TEXT    NOT NULL
      );
      `,
  );

  const [s, err, db_migrations] = await database.all<i_migrations>(
    `SELECT id, name, up, down FROM "${migration_table}" ORDER BY id ASC`,
  );

  if (s === "err") {
    throw err;
  }

  const migration_map = new Map<number, boolean>();
  migrations.forEach((m) => migration_map.set(m.id, true));

  const db_only_migrations = db_migrations.filter((m) => {
    return !migration_map.has(m.id);
  });

  const rev_migration = db_only_migrations.sort((a, b) =>
    Math.sign(b.id - a.id),
  );

  for (let index = 0; index < rev_migration.length; index++) {
    const mig = rev_migration[index];

    const begin_run = await database.run("BEGIN");
    if (begin_run[0] === "err") {
      await database.run("ROLLBACK");
      throw begin_run[1];
    }

    const mig_run = await database.exec(mig.down);
    if (mig_run[0] === "err") {
      await database.run("ROLLBACK");
      throw mig_run[1];
    }

    const del_run = await database.run(
      `DELETE FROM "${migration_table}" WHERE id= ?`,
      [mig.id],
    );
    if (del_run[0] === "err") {
      await database.run("ROLLBACK");
      throw del_run[1];
    }

    const commit_run = await database.run("COMMIT");
    if (commit_run[0] === "err") {
      await database.run("ROLLBACK");
      throw del_run[1];
    }
  }

  const last_migration_id = db_migrations.at(-1)?.id || 0;

  for (const mig of migrations) {
    if (mig.id > last_migration_id) {
      const begin_run = await database.run("BEGIN");
      if (begin_run[0] === "err") {
        await database.run("ROLLBACK");
        throw begin_run[1];
      }

      const mig_run = await database.exec(mig.up);
      if (mig_run[0] === "err") {
        await database.run("ROLLBACK");
        throw mig_run[1];
      }

      const del_run = await database.run(
        `INSERT INTO "${migration_table}" (id, name, up, down) VALUES (?, ?, ?, ?);`,
        [mig.id, mig.name, mig.up, mig.down],
      );
      if (del_run[0] === "err") {
        await database.run("ROLLBACK");
        throw del_run[1];
      }

      const commit_run = await database.run("COMMIT");
      if (commit_run[0] === "err") {
        await database.run("ROLLBACK");
        throw del_run[1];
      }
    }
  }
}

export async function load_migrations(
  dir: string,
): Promise<Array<i_migration_data>> {
  const _migration_flies = await async_handler(() => {
    return readdir(dir, { encoding: "utf8", recursive: false });
  });

  if (_migration_flies[0] === "err") {
    throw _migration_flies[1];
  }

  console.log(_migration_flies);
  const migration_flies = _migration_flies[2].filter((filename) => {
    return FILE_NAME_REGEX.test(filename);
  });

  console.log(migration_flies);

  const migration_promises = migration_flies.map(
    async (mig_filename): Promise<Result<i_migration_data, Error>> => {
      const file_path = path.resolve(dir, mig_filename);
      const [s, err, migration_data] = await async_handler(() => {
        return readFile(file_path, { encoding: "utf8" });
      });

      if (s === "err") {
        return [s, err, null];
      }

      const [up, down] = migration_data.split("-- DOWN");
      const match = mig_filename.match(FILE_NAME_REGEX);

      assert(match !== null, "invalid migration file name");
      assert(match.length > 3, "invalid migration file name pattern");

      const [_file, index, name, ext] = match;

      assert(!isNaN(Number(index)), "file index must be a number");

      const data = {
        id: Number(index),
        name,
        filename: name + ext,
        up: normalize_sqlstr(up),
        down: normalize_sqlstr(down),
      };

      return ["ok", null, data];
    },
  );

  const migrations = await async_handler(() => Promise.all(migration_promises));
  if (migrations[0] === "err") {
    throw migrations[1];
  }

  const unwrapped_migration = migrations[2]
    .map((m) => {
      const unwapped_m = unwrap(m);
      if (unwapped_m === null) {
        throw m[1];
      }

      return unwapped_m;
    })
    .sort((a, b) => Math.sign(b.id - a.id));

  const unique_migrations = Array.from(
    new Set(unwrapped_migration.map((e) => e.id)),
  );

  assert.equal(unique_migrations.length, unwrapped_migration.length);

  return unwrapped_migration;
}

function normalize_sqlstr(str: string) {
  return str
    .replace(/-- \w+/, "")
    .split("\n")
    .filter(Boolean)
    .join("\n")
    .trim();
}
