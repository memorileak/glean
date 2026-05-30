from __future__ import annotations

import math
from typing import Generic, Iterator, Protocol, TypeVar

T = TypeVar("T")
K = TypeVar("K")
V = TypeVar("V")


class Comparable(Protocol):
    def __lt__(self, other: object) -> bool: ...
    def __eq__(self, other: object) -> bool: ...


class Stack(Generic[T]):
    def __init__(self) -> None:
        self._items: list[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> T:
        if not self._items:
            raise IndexError("pop from empty stack")
        return self._items.pop()

    def peek(self) -> T:
        if not self._items:
            raise IndexError("peek at empty stack")
        return self._items[-1]

    def __len__(self) -> int:
        return len(self._items)

    def __iter__(self) -> Iterator[T]:
        return iter(reversed(self._items))


class BinarySearchTree(Generic[T]):
    class _Node:
        def __init__(self, value: T) -> None:
            self.value = value
            self.left: BinarySearchTree._Node | None = None
            self.right: BinarySearchTree._Node | None = None

    def __init__(self) -> None:
        self._root: BinarySearchTree._Node | None = None
        self._size = 0

    def insert(self, value: T) -> None:
        self._root = self._insert(self._root, value)
        self._size += 1

    def _insert(self, node: _Node | None, value: T) -> _Node:
        if node is None:
            return BinarySearchTree._Node(value)
        if value < node.value:
            node.left = self._insert(node.left, value)
        elif value > node.value:
            node.right = self._insert(node.right, value)
        return node

    def contains(self, value: T) -> bool:
        node = self._root
        while node is not None:
            if value == node.value:
                return True
            node = node.left if value < node.value else node.right
        return False

    def inorder(self) -> Iterator[T]:
        def _walk(node: _Node | None) -> Iterator[T]:
            if node:
                yield from _walk(node.left)
                yield node.value
                yield from _walk(node.right)
        return _walk(self._root)


def fibonacci(n: int) -> int:
    if n < 0:
        raise ValueError("n must be non-negative")
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a


def sieve_of_eratosthenes(limit: int) -> list[int]:
    if limit < 2:
        return []
    is_prime = [True] * (limit + 1)
    is_prime[0] = is_prime[1] = False
    for i in range(2, math.isqrt(limit) + 1):
        if is_prime[i]:
            for j in range(i * i, limit + 1, i):
                is_prime[j] = False
    return [i for i, p in enumerate(is_prime) if p]


def flatten(nested: list) -> list:
    result = []
    for item in nested:
        if isinstance(item, list):
            result.extend(flatten(item))
        else:
            result.append(item)
    return result


MAX_RETRIES = 3
DEFAULT_TIMEOUT = 30.0
VERSION = "0.1.0"
