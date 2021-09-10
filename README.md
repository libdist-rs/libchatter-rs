Testing this codebase:
1. `make testdata`
2. `bash scripts/apollo-release-quick-test.sh` or any of the test scripts 

# Other notes
- Consensus module contains the reactors which react to the different protocol
  messages
- The config module holds the config information
- The crypto implements RSA, EDDSA and SECP256K1 PKI
- net module implements the optimized tokio module (gives two channels for the
  consensus reactors to send/receive messages to/from)
- scripts contain a myriad of scripts used for data/node/aws processing
- tools has a config generation tool
- types holds the definitions of blocks, transactions, rounds, heights, along
  with some common types
- util holds some utility libraries like writing out config files
