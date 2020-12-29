# Instructions for performing Experiments on AWS

## Setup instances

1. Get an AMI up and running. I use the Arch Linux AMIs found [here](https://www.uplinklabs.net/projects/arch-linux-on-ec2/) for convenience. Before starting the AMIs ensure that they are on the same subnet. Refer [here](https://www.simplilearn.com/tutorials/aws-tutorial/aws-vpc) for more instructions on setting up a private network.
    Note: I used the public key of my local machine, so I can directly run `ssh arch@XXXX` without providing a key file. If you have used another key file, change the scripts to use `ssh -i PATH_TO_KEY ...` in all the scripts.

2. List the public IPs of the instance(s) in `aws_ips.log` file. It must be in `scripts/aws/aws_ips.log`.

3. Run the following command to update compilers, install Rust, clone the code, and build the binaries.
   ```bash
   $ bash scripts/aws/do_setup.sh < scripts/aws/aws_ips.log 
   ```
   Note: Change `scripts/aws/setup.sh` if using other AMIs such as ubuntu.

4. At this point, we can setup multiple instances in two ways:
    - Clone more instances from the first instance after finishing the setup
    - Alternatively, `scripts/aws/aws_ips.log` can have the IPs of multiple instances, and the setup script will start parallel update sessions.
        Note: The output may be hard to read using method II. It is a good approach to use `t1micro` instance (which is free) to debug the scripts and ensure that they work to be sure that method II works for your setting.

5. Store their private IPs in `scripts/aws/pvt_ips.log`. We will use this in setting up the connections between nodes for later experiments.

## Ping Experiment

0. Ensure that the instances are setup as defined previously. We will use the scripts in `scripts/aws/ping` for this experiment.
1. The ping experiment only needs $2$ nodes: one for the echo server and the other for the ping client. Therefore, `scripts/aws/ping/aws_ips.log` must contain only two public IPs of the instances.
2. Store the private IP of the echo server in `scripts/aws/ping/pvt_ips.log`.
3. Run the following command to start the experiment:
    ```bash
    # Takes two arguments: AWS Public IP file and the private IP file
    $ bash scripts/aws/ping/do_ping-exp.sh \
    scripts/aws/ping/aws_ips.log scripts/aws/ping/pvt_ips.log
    ```
    Note: By default this will result in all numbers being reported on the terminal. You can redirect this to a raw file to debug any errors. Otherwise, a third optional argument can be provided that will redirect all the output to the file, for later processing as follows:
    ```bash
    # Use the third optional argument to get results: 
    $ bash scripts/aws/ping/do_ping-exp.sh \
    scripts/aws/ping/aws_ips.log scripts/aws/ping/pvt_ips.log \
    scripts/aws/ping/ping-raw.log # This is the default value if nothing is specified
    ```
4. To parse the raw log file into a csv file for plotting use the parse script `scripts/ping/parse-ping-exp.py` as follows:
    ```bash
    $ python scripts/ping/parse-ping-exp.py \
    scripts/aws/ping/ping-raw.log \ # Path to the RAW log file
    scripts/aws/ping/ping.csv \ # Where to output the CSV file
    --extract ping # Extract ping information
    ```
5. Finally, I use the ipython file `scripts/ping/plot-ping.ipynb` to manage the plots used in the paper. Modify this file as per taste.