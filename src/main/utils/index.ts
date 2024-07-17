export type Result<T = unknown, E = Error> = ["ok", null, T] | ["err", E, null];

export function unwrap<T>(val: Result<T>) {
  return val[2] || null;
}

export async function async_handler<
  T extends (...arg: unknown[]) => any,
  U = ReturnType<T>,
>(fn: T): Promise<Result<Awaited<U>>> {
  try {
    const res = await fn();
    return ["ok", null, res];
  } catch (error) {
    return ["err", error as Error, null];
  }
}

export async function callback_handler<T>(fn: (callback: () => void) => void) {
  const p = new Promise<T>((resolve, rejects) => {
    fn(function () {
      if (arguments[0]) {
        return rejects(arguments[0]);
      }
      resolve(arguments[1]);
    });
  });

  return await async_handler(() => p);
}
