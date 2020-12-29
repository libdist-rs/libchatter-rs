# Local Testing

Here, I document how to test the code locally, to ensure that everything works correctly.

## Build

1. Run the following command to ensure that everything is built:
    ```bash
    $ cargo build --all --release # Build all the targets with optimizations
    # Or
    $ make release # Runs the above, but shorter
    ```
2. Prepare the test data by running:
    ```bash
    # Prepares config files for tests
    $ make testdata 
    ```

## Ping Experiment

0. Ensure that at least Step 1 of the Build is performed.
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
1. 
