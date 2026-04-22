#! /usr/bin/env python3

import json
import os
import re
import subprocess
from collections import defaultdict
from datetime import datetime
from pathlib import Path

root = os.getcwd()
inputs_dir = Path(root, "inputs")
expected_dir = Path(root, "expected")


# Utilities
# ======================================================================================
class bcolors:
    HEADER = "\033[95m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    OKGREEN = "\033[92m"
    WARNING = "\033[93m"
    FAIL = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"


test_results = []


def test_results_add(number, name, output, score, max_score=1):
    test_results.append(
        {
            "max_score": max_score,
            "name": name,
            "number": number,
            "output": output,
            "score": score,
        }
    )


def run_sim(args, inputfile):
    # Run the simulation.
    sim_process = subprocess.Popen(
        ["./cachesim", *args],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )

    # Pass in the input file on stdin.
    with open(inputfile, "rb") as i:
        stdout, _ = sim_process.communicate(i.read())

    # Return the lines that have OUTPUT at the beginning
    return list(filter(lambda l: l.startswith("OUTPUT"), stdout.decode().split("\n")))


def run_matrix_addition():
    # Run the simulation.
    cwd = os.getcwd()
    os.chdir('matrix_addition')

    make_run_process = subprocess.run(
        ["make", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )

    if make_run_process.returncode != 0:
        print(make_run_process.stdout)
        return None

    os.chdir(cwd)


# LRU and LRU_PREFER_CLEAN functionality
# ======================================================================================
print(f"{bcolors.BOLD}Checking ADJACENT and SEQUENTIAL functionality.{bcolors.ENDC}")

adjacent_i = 1
sequential_i = 1

# Iterate through all of the inputs, and for each of the corresponding "expected" files,
# run the simulation with the parameters specified by the expected fie.
for infile in sorted(inputs_dir.iterdir()):
    print(f"  Checking {infile}")
    for expected_file_path in sorted(expected_dir.glob(f"*-{infile.name}")):
        file_parts = re.match(
            rf"(\d+)-(\d+)-(\d+)-(adjacent|sequential)-(\d+)-{infile.name}",
            expected_file_path.name,
        )
        (
            cache_size, cache_lines, associativity, prefetcher, prefetch_amount
        ) = file_parts.groups()
        print(
            f"    with parameters {' '.join(map(str.upper, file_parts.groups()))}...",
            end=" ",
        )

        # Calculate the test number and name
        if prefetcher == "adjacent":
            test_number = f"1.1.{adjacent_i}"
            adjacent_i += 1
            max_score = 3
        else:
            test_number = f"1.2.{sequential_i}"
            sequential_i += 1
            max_score = 1
        test_name = expected_file_path.name

        output_lines = run_sim(["LRU", *map(str.upper, file_parts.groups())], infile)

        # Get the expected output.
        with open(expected_file_path) as ef:
            expected_output_lines = [line.strip() for line in ef.readlines()]

        # Make sure everything matches up
        if len(output_lines) != len(expected_output_lines):
            print(f"{bcolors.BOLD}{bcolors.FAIL}FAIL{bcolors.ENDC}")
            error_text = "      {} OUTPUT lines found, expected {}".format(
                len(output_lines),
                len(expected_output_lines),
            )
            print(error_text)
            test_results_add(test_number, test_name, error_text, 0)
            continue

        # Check each of the output lines.
        fail = False
        for i, (found, expected) in enumerate(zip(output_lines, expected_output_lines)):
            if found != expected:
                print(f"{bcolors.BOLD}{bcolors.FAIL}FAIL{bcolors.ENDC}")
                error_text = "\n".join(
                    (
                        f"      On line {i} found:",
                        f"        {found}",
                        "      expected:",
                        f"        {expected}",
                    )
                )
                print(error_text)
                test_results_add(
                    test_number,
                    test_name,
                    error_text,
                    0,
                    max_score=max_score
                )
                fail = True
                break

        if fail:
            continue

        test_results_add(test_number, test_name, "PASS", max_score, max_score=max_score)
        print(f"{bcolors.BOLD}{bcolors.OKGREEN}PASS{bcolors.ENDC}")


# Print out the test results and store to the test results JSON file.
# ======================================================================================
test_results_dir = Path(root, "test_results")
test_results_dir.mkdir(exist_ok=True, parents=True)
test_results_filename = test_results_dir.joinpath(
    datetime.now().strftime("%Y-%m-%d-%H-%M-%S.json")
)

# Sort by the number (convert to tuples of integers for sorting)
test_results.sort(key=lambda x: tuple(int(n) for n in x["number"].split(".")))
aggregated_scores = defaultdict(int)
aggregated_max_scores = defaultdict(int)
for tr in test_results:
    rubric_item_key = tuple(tr["number"].split(".")[:2])
    aggregated_scores[rubric_item_key] += tr["score"]
    aggregated_max_scores[rubric_item_key] += tr["max_score"]

# Print out the results
print(f"\n{bcolors.BOLD}Results Summary{bcolors.ENDC}")
for k, v in aggregated_max_scores.items():
    print(f"  Rubric Item {'.'.join(k)}: {aggregated_scores[k]}/{v}")
print(bcolors.BOLD)
total_score = sum(aggregated_scores.values())
print(f"  Total Autograded Score: {total_score}/{sum(aggregated_max_scores.values())}")

print(bcolors.ENDC + bcolors.WARNING)
print("  NOTE: the starter code does not contain the complete set of test cases, so")
print("  this score may not fully represent your final autograded score.")
print(bcolors.ENDC)

# Write results to a file
print(f"{bcolors.BOLD}Writing results to {test_results_filename}{bcolors.ENDC}")
with open(test_results_filename, "w+") as f:
    json.dump(test_results, f)
