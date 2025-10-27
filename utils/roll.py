#/usr/bin/env python
from random import randint

class Die():
    sides: int

    def __init__(self, *, sides: int):
        self.sides = sides

    def roll(self) -> int:
        return randint(1, self.sides)

d6 = Die(sides=6)

print(d6.roll())
