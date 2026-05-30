export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export type Nullable<T> = T | null | undefined;

export interface Repository<T, ID = string> {
  findById(id: ID): Promise<Nullable<T>>;
  findAll(): Promise<T[]>;
  save(entity: T): Promise<T>;
  delete(id: ID): Promise<void>;
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export interface UserService {
  getUser(id: string): Promise<Nullable<User>>;
  createUser(dto: CreateUserDto): Promise<User>;
  updateUser(id: string, dto: Partial<CreateUserDto>): Promise<User>;
  deleteUser(id: string): Promise<void>;
}

export class User {
  readonly id: string;
  name: string;
  email: string;
  createdAt: Date;

  constructor(id: string, name: string, email: string) {
    this.id = id;
    this.name = name;
    this.email = email;
    this.createdAt = new Date();
  }

  toJSON() {
    return { id: this.id, name: this.name, email: this.email };
  }
}

export class ApiError extends Error {
  constructor(
    public readonly statusCode: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export type CreateUserDto = Pick<User, "name" | "email">;

export class InMemoryUserRepository implements Repository<User> {
  private store = new Map<string, User>();

  async findById(id: string): Promise<Nullable<User>> {
    return this.store.get(id) ?? null;
  }

  async findAll(): Promise<User[]> {
    return Array.from(this.store.values());
  }

  async save(user: User): Promise<User> {
    this.store.set(user.id, user);
    return user;
  }

  async delete(id: string): Promise<void> {
    this.store.delete(id);
  }
}

export function paginate<T>(items: T[], page: number, pageSize: number): Paginated<T> {
  const start = (page - 1) * pageSize;
  return {
    items: items.slice(start, start + pageSize),
    total: items.length,
    page,
    pageSize,
  };
}

export function isOk<T, E>(result: Result<T, E>): result is { ok: true; value: T } {
  return result.ok;
}

function* idGenerator(prefix = "id"): Generator<string> {
  let n = 0;
  while (true) {
    yield `${prefix}-${++n}`;
  }
}

describe("InMemoryUserRepository", () => {
  it("saves and retrieves a user", async () => {
    const repo = new InMemoryUserRepository();
    const user = new User("1", "Alice", "alice@example.com");
    await repo.save(user);
    expect(await repo.findById("1")).toEqual(user);
  });

  describe("findAll", () => {
    it("returns empty array initially", async () => {
      const repo = new InMemoryUserRepository();
      expect(await repo.findAll()).toHaveLength(0);
    });
  });
});
