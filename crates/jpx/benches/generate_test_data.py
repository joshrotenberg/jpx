#!/usr/bin/env python3
"""Generate test JSON files of various sizes for benchmarking jpx."""

import json
import random
import string
import os

def random_string(length=10):
    return ''.join(random.choices(string.ascii_letters, k=length))

def generate_user():
    return {
        "id": random.randint(1, 1000000),
        "name": random_string(15),
        "email": f"{random_string(8)}@example.com",
        "age": random.randint(18, 80),
        "active": random.choice([True, False]),
        "score": round(random.uniform(0, 100), 2),
        "tags": [random_string(5) for _ in range(random.randint(1, 5))],
        "metadata": {
            "created": "2024-01-15T10:30:00Z",
            "updated": "2024-06-20T14:45:00Z",
            "version": random.randint(1, 10)
        }
    }

def generate_dataset(num_users):
    return {"users": [generate_user() for _ in range(num_users)]}

def main():
    sizes = [
        ("small", 100),
        ("medium", 1000),
        ("large", 10000),
        ("xlarge", 50000),
    ]

    output_dir = os.path.dirname(os.path.abspath(__file__))

    for name, count in sizes:
        data = generate_dataset(count)
        filename = os.path.join(output_dir, f"test_{name}.json")
        with open(filename, 'w') as f:
            json.dump(data, f)
        size_kb = os.path.getsize(filename) / 1024
        print(f"Generated {filename}: {count} users, {size_kb:.1f} KB")

    # Also generate a file with multiple JSON objects for slurp testing
    slurp_file = os.path.join(output_dir, "test_slurp.json")
    with open(slurp_file, 'w') as f:
        for _ in range(1000):
            json.dump(generate_user(), f)
            f.write('\n')
    size_kb = os.path.getsize(slurp_file) / 1024
    print(f"Generated {slurp_file}: 1000 objects, {size_kb:.1f} KB")

if __name__ == "__main__":
    main()
