#!/bin/bash

# Fail immediately if any command has a non-zero exit status, or if a variable hasn't been defined.
set -eu

# Used in code to customize behaviour for testing.
# Don't change this, as that will lead to an infinite main (repl) loop!
# This is required in this script to be `true`.
export TEST=true

# Build project with the above `TEST` value.
# Inside the `run.sh` script, `DEBUG` can be changed freely.
echo "Building the shell for testing..."
./run.sh &
sleep 1s # Give test shell some time to build.

# A shorthand for running the shell under test.
# TODO: Probably remove 2>/dev/null
shell="exec /tmp/build-shell/release/posix-shell 2>/dev/null &"

# Start the shell for testing.
echo "Starting the test shell..."
server_pid=$! # Get the PID of the shell background process.
sleep 2s # Give test shell some time to run.
printf "Test shell running...\n\n";

# Just testing - TODO: remove this
echo pwd | $shell
echo whoami | $shell
echo "cat a" | $shell
echo "cat n" | $shell
echo pwd | $shell
echo "cat a n" | $shell

# Test number counter
test_nr=0

# Runs a single test. It expects two arguments:
# 1. Shell command
# 2. Expected response
run_test()
{
  test_nr=$((test_nr+1))
  printf 'Running test #%d...' "$test_nr"
  response=$(echo -ne "$1" | nc localhost "$PORT"; echo .)
  response=${response%.}
  if [ "$response" = "$2" ]; then
    printf ' PASSED'
  else
    printf ' FAILED\nGot:\n%s' "$response"
    printf 'Expected:\n%s' "$2"
  fi
  echo
  # sleep 0.1
}

# Run all tests in succession now.



# Stop the test server. First try by its PID, and if that fails, kill all running shell processes.
set +e # Don't fail immediately in case of non-zero exit status anymore, as we'll depend on this now.
sleep 0.1
echo
kill $server_pid
kill_status=$?
if [ $kill_status -eq 0 ]; then
  echo "Test shell stopped."
else
  echo "kill $server_pid failed; killing all running shell processes now..."
  pkill posix-shell
  echo "All shells stopped."
fi
