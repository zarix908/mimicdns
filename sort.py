#!/usr/bin/env python3
"""Read words from a file, sort them by length, and write back to the file."""

import sys


def sort_words_by_length(filepath: str) -> None:
    """Read words from file, sort by length, and write back."""
    with open(filepath, 'r', encoding='utf-8') as f:
        words = f.read().split()
    
    words.sort(key=len)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write('\n'.join(words))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <filepath>")
        sys.exit(1)
    
    sort_words_by_length(sys.argv[1])
