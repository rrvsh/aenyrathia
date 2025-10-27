# /usr/bin/env python
from collections import Counter
from random import randint
from re import fullmatch
from argparse import ArgumentParser
from typing import Mapping, Tuple


class DicePool:
    """
    _rolls is a map of index (which die in the pool) to result (what number on the die)
    """

    size: int
    sides: int

    @classmethod
    def parse_notation(cls, encoded: str) -> Tuple[int, int]:
        match = fullmatch(r"(\d*)d(\d+)", encoded)
        if match is None:
            raise ValueError(f"Invalid dice format: {encoded}")
        size, sides = match.groups()
        if size is not None:
            size = int(size)
        else:
            size = 1
        return (size, int(sides))

    @classmethod
    def from_notation(cls, encoded: str):
        size, sides = cls.parse_notation(encoded)
        return cls(size, sides)

    def __init__(self, size: int, sides: int):
        self.size = size
        self.sides = sides

    def get_rolls(self) -> Mapping[int, int]:
        return {index: randint(1, self.sides) for index in range(self.size)}

    def get_total(self) -> int:
        return sum(self.get_rolls().values())

    def get_highest_number(self) -> int:
        return max(self.get_rolls().values())


parser = ArgumentParser()
parser.add_argument("notation", type=str)
parser.add_argument("rolls", type=int)
args = parser.parse_args()
max_size, sides = DicePool.parse_notation(args.notation)

for size in range(1, max_size + 1):
    counter = Counter()
    notation = f"{size}d{sides}"
    print("===")
    print(f"Stats for {notation} over {args.rolls} rolls:")

    for roll_number in range(args.rolls):
        counter[DicePool(size, sides).get_highest_number()] += 1

    for side_number in range(1, sides + 1):
        probability = counter[side_number] / args.rolls
        bar = "█" * int(probability * 100)
        print(f"{side_number}: {bar} {int(probability * 100)}%")
