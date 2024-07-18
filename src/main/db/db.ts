import path from "path";
import sqlite3 from "sqlite3";
import { callback_handler } from "../utils";

const base_db_path = path.resolve(process.cwd(), "src/main/db");
const db_path = path.resolve(base_db_path, "store", "open_worship.sqlite");
export const migration_path = path.resolve(base_db_path, "migrations");

const db = new sqlite3.Database(db_path);

async function run(sql: string, param: unknown[] = []) {
  return await callback_handler<void>((cb) => db.run(sql, param, cb));
}

async function exec(sql: string) {
  return await callback_handler<void>((cb) => db.exec(sql, cb));
}

async function get<T>(sql: string, param: unknown[] = []) {
  return await callback_handler<T>((cb) => db.get<T>(sql, param, cb));
}

async function all<T>(sql: string, param: unknown[] = []) {
  return await callback_handler<T[]>((cb) => db.all<T>(sql, param, cb));
}

async function close() {
  return await callback_handler((cb) => db.close(cb));
}

export const database = { db, run, exec, get, all, close };
export type i_database = typeof database;
