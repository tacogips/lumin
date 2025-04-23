#!/usr/bin/env python3

"""
Setup script for testing lumin
"""

import os
import sys

def main():
    print("Setting up test environment...")
    
    # Create test files
    with open("test_file.txt", "w") as f:
        f.write("This is a test file for searching\n")
        f.write("It contains some patterns to find\n")
        f.write("Pattern: TEST_PATTERN_123\n")
    
    print("Setup complete!")

if __name__ == "__main__":
    main()
