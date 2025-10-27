import os
import textwrap
from pyphen import Pyphen


def hyphenate_wrap(text, width=80):
    wrapped_paragraphs = []

    for paragraph in text.split("\n"):
        paragraph = paragraph.strip()
        if paragraph == "":
            wrapped_paragraphs.append("")
            continue

        words = paragraph.split()
        processed_words = []

        for word in words:
            if len(word) > width:
                hyphenated = Pyphen(lang="en_US").inserted(word)
                pieces = hyphenated.replace("\u00ad", "-").split("-")
                processed_words.append("- ".join(pieces))
            else:
                processed_words.append(word)

        wrapped = textwrap.fill(" ".join(processed_words), width=width)
        wrapped_paragraphs.append(wrapped)

    return "\n".join(wrapped_paragraphs)


def process_markdown_files():
    for dirpath, _, filenames in os.walk("."):
        for filename in filenames:
            if filename.endswith(".md"):
                filepath = os.path.join(dirpath, filename)
                with open(filepath, "r", encoding="utf-8") as f:
                    text = f.read()
                formatted_text = hyphenate_wrap(text)
                with open(filepath, "w", encoding="utf-8") as f:
                    f.write(formatted_text)


if __name__ == "__main__":
    process_markdown_files()
