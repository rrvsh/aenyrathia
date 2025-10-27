nice: format lint

lint: lint-py

lint-py:
  ruff check --fix

format: format-py format-md

format-py:
  ruff format

format-md:
  python src/utils/format-md.py

roll *ARGS:
  python src/utils/roll.py {{ ARGS }}
