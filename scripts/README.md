# Local Testing

Here, I document how to test the code locally, to ensure that everything works correctly.

## Build

1. Run the following command to ensure that everything is built:
    ```bash
    # Build all the targets with optimizations
    $ cargo build --all --release 
    # Or
    $ make release # Runs the above, but shorter
    ```
2. Prepare the test data by running:
    ```bash
    # Prepares default config files for tests
    $ make testdata 
    ```

## Ping Experiment

0. Ensure that at least Step 1 of the [Build](#Build) is performed.
1. Run the ping test script `scripts/ping/ping-exp.sh` as:
    ```bash
    # Do local ping experiment and store it
    $ bash scripts/ping/ping-exp.sh \
    &> scripts/data/ping/ping-raw.log # Can be omitted
    ```
2. Using the raw data from the previous, we can create a CSV file with percentile data points by running:
    ```bash
    # `scripts/data/ping/ping-raw.log` is the input file
    # `scripts/data/ping/ping.csv` specifies where to output the csv
    $ python scripts/ping/parse-ping-exp.py \
    scripts/data/ping/ping-raw.log \
    scripts/data/ping/ping.csv
    ```
    Note: Modify the `scripts/ping/parse-ping-exp.py` to change the data points.

3. I use the ipython notebook file `scripts/ping/plot-ping.ipynb` to produce the final graphs used in the paper.

## Throughput vs. Latency

0. Ensure that all of the steps in Build is performed.
1. The main driver scripts are:
    ```bash
    # Sync HotStuff driver script
    $ bash scripts/quick-test.sh
    Successfully decoded the config file
    Successfully decoded the config file
    Successfully decoded the config file
    Connected to 127.0.0.1:55062
    Connected to 127.0.0.1:60738
    Connected to peer: 0
    ...
    Successfully decoded the config file
    Successfully decoded the config file
    DP[Throughput]: 57954.94521678823
    DP[Latency]: 1616.5541975664194
    Closing tx producer channel: channel closed
    $
    # Apollo driver script
    $ bash scripts/synchs-test.sh
    Successfully decoded the config file
    Successfully decoded the config file
    Successfully decoded the config file
    Connected to 127.0.0.1:55104
    ...
    DP[Throughput]: 58295.12316892239
    DP[Latency]: 1609.0790820917907
    Closing tx producer channel: channel closed
    $
    ```
2. These scripts use environment variables `W`, `TESTDIR`, and `TYPE` that can be used to change the behaviour of the script.
    ```bash
    # Change window duration
    $ W=50000 bash scripts/quick-test.sh
    ...
    # Change test directory
    $ TESTDIR="testdata/b800-n3" bash scripts/quick-test.sh
    ...
    # Change the type (debug or release)
    $ TYPE="debug" bash scripts/quick-test.sh
    ```
3. Driver Scripts: In order to streamline a series of experiments, we use driver scripts that use these smaller scripts with different parameters. Examples are `scripts/apollo-debug-quick-test.sh`, `scripts/synchs-release-quick-test.sh`, `scripts/throughput-vs-latency/vary-b/exp.sh`.