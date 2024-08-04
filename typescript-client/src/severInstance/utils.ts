export async function sleep(timeout_ms: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeout_ms);
  });
}

export async function retry<T>(fn: () => Promise<T>, retry: number): Promise<T> {
  let count = 0;
  while (true) {
    count += 1;
    try {
      return await fn();
    } catch (err) {
      if (count > retry) {
        throw err;
      }
    }
  }
}

export function omit<T extends object, K extends keyof T>(obj: T, keys: K[]): Omit<T, K> {
  return Object.keys(obj)
    .filter(k => !keys.includes(k as K))
    .reduce(
      (filteredObj: Omit<T, K>, key) => ({
        ...filteredObj,
        [key]: obj[key as keyof T],
      }),
      {} as Omit<T, K>
    );
}
