export default class Fifo<T> {
  private head?: Item<T>;
  private tail?: Item<T>;
  private itemCount: number;

  public constructor() {
    this.head = undefined;
    this.tail = undefined;
    this.itemCount = 0;
  }

  public push(item: T) {
    this.itemCount++;

    if (!this.tail) {
      this.head = {
        item: item,
      };
      this.tail = this.head;
      return;
    }

    this.tail.next = {
      item: item,
    };
    this.tail = this.tail.next;
  }

  public pop(): T | undefined {
    const item = this.head;
    if (!item) {
      return undefined;
    }

    this.itemCount--;
    this.head = item.next;
    if (Object.is(item, this.tail)) {
      this.tail = undefined;
    }

    return item.item;
  }

  public get length(): number {
    return this.itemCount;
  }
}

type Item<T> = {
  item: T;
  next?: Item<T>;
};
