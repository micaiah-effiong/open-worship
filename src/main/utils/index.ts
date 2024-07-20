export type Result<T = unknown, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E | undefined };

export function unwrap<T>(val: Result<T>) {
  if (val.ok) {
    return val.value;
  }

  return null;
}

export async function async_handler<
  T extends (...arg: unknown[]) => any,
  U = ReturnType<T>,
>(fn: T): Promise<Result<Awaited<U>>> {
  try {
    const res = await fn();
    return { ok: true, value: res };
  } catch (error) {
    return { ok: false, error: error as Error };
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
