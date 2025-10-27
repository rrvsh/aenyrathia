# /usr/bin/env python
from random import randint
from re import fullmatch
from argparse import ArgumentParser
from typing import Iterable, Mapping, Tuple


class Dice:
    """
    _rolls is a map of index (which die in the pool) to result (what number on the die)
    """

    number: int
    sides: int

    @classmethod
    def parse_notation(cls, encoded: str) -> Tuple[int, int]:
        match = fullmatch(r"(\d*)d(\d+)", encoded)
        if match is None:
            raise ValueError(f"Invalid dice format: {encoded}")
        number_string, sides_string = match.groups()
        if number_string is not None:
            number = int(number_string)
        else:
            number = 1
        return (number, int(sides_string))

    @classmethod
    def from_notation(cls, encoded: str):
        number, sides = cls.parse_notation(encoded)
        return cls(number, sides)

    def __init__(self, number: int, sides: int):
        self.number = number
        self.sides = sides

    def get_rolls(self) -> Mapping[int, int]:
        return {index: randint(1, self.sides) for index in range(self.number)}

    def get_total(self) -> int:
        rolls: Iterable[int] = self.get_rolls().values()
        return sum(rolls)


parser = ArgumentParser()
parser.add_argument("notation", type=str)
parser.add_argument("rolls", type=int)
args = parser.parse_args()
number, sides = Dice.parse_notation(args.notation)

for i in range(1, number + 1):
    total = 0
    for _ in range(args.rolls):
        total += Dice(i, sides).get_total()
    print(
        f"average result of {i}d{sides} over {args.rolls} rolls: {round(total / args.rolls, 2)}"
    )
