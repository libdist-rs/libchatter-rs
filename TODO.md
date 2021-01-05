# Things to improve in the library

- [x] Used a new improved channel implementation (crossfire is dangerous; It causes non-determinstic freezes/deadlocks in the program)
- [x] Write a new wake based stream 
- [x] Write a new efficient delay queue implementation
- [x] Write a new wake based sink
- [x] Look for parallelization opportunities (in consensus)
- [ ] Make testdata for 
    - [ ] Vary d experiment
    - [ ] Vary f experiment
- [ ] Fix exp.sh for vary f
- [ ] Fix exp.sh for vary d
- [x] Change apollo client pending command manager: send f blocks first, then start tracking pending commands.
- [ ] Util: Write one generic encoder
- [ ] Util: Write one generic decoder
- [x] Use Arcs to avoid cloning of protocol messages
- [x] Initialize protocol messages and blocks only once before arcing
- [x] Net: Generalize to be protocol agnostic
- [ ] Reorg library