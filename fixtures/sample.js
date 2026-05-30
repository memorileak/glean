class EventEmitter {
  #listeners = new Map();

  on(event, callback) {
    if (!this.#listeners.has(event)) {
      this.#listeners.set(event, []);
    }
    this.#listeners.get(event).push(callback);
    return this;
  }

  emit(event, ...args) {
    this.#listeners.get(event)?.forEach((cb) => cb(...args));
  }
}

class Logger extends EventEmitter {
  constructor(name) {
    super();
    this.name = name;
  }

  log(level, message) {
    const entry = { level, message, ts: Date.now() };
    this.emit("log", entry);
    console.log(`[${this.name}] ${level}: ${message}`);
  }
}

function createPipeline(...steps) {
  return (input) => steps.reduce((acc, fn) => fn(acc), input);
}

function* range(start, end, step = 1) {
  for (let i = start; i < end; i += step) {
    yield i;
  }
}

const debounce = (fn, delay) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
};

const memoize = function (fn) {
  const cache = new Map();
  return function (...args) {
    const key = JSON.stringify(args);
    if (cache.has(key)) return cache.get(key);
    const result = fn.apply(this, args);
    cache.set(key, result);
    return result;
  };
};

// Prototype-style class
function Vector2D(x, y) {
  this.x = x;
  this.y = y;
}

Vector2D.prototype.add = function (other) {
  return new Vector2D(this.x + other.x, this.y + other.y);
};

Vector2D.prototype.length = function () {
  return Math.sqrt(this.x ** 2 + this.y ** 2);
};

describe("EventEmitter", () => {
  it("emits events to registered listeners", () => {
    const emitter = new EventEmitter();
    let received = null;
    emitter.on("data", (v) => (received = v));
    emitter.emit("data", 42);
    expect(received).toBe(42);
  });

  it("supports chaining .on() calls", () => {
    const emitter = new EventEmitter();
    expect(emitter.on("x", () => {})).toBe(emitter);
  });
});

describe("createPipeline", () => {
  test("applies steps in order", () => {
    const double = (x) => x * 2;
    const addOne = (x) => x + 1;
    const pipe = createPipeline(double, addOne);
    expect(pipe(3)).toBe(7);
  });
});

test.skip("range generator (not yet implemented)", () => {
  expect([...range(0, 5)]).toEqual([0, 1, 2, 3, 4]);
});
