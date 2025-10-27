nice: format lint

lint: lint-py

lint-py:
  ruff check --fix

format: format-py format-md

format-py:
  ruff format

format-md:
  python utils/format-md.py

roll *ARGS:
  python utils/roll.py {{ ARGS }}
