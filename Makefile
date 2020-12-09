.PHONY: testdata node-apollo tools apollo synchs sink streamer relay sinker sink-release

tools:
	cargo build --package=genconfig

testdata:
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--target testdata/b400-n3

# ============= BUILD APOLLO ============
apollo: 
	cargo build --package=node-apollo --package=client-apollo --release

node-apollo:
	cargo build --package=node-apollo

client-apollo:
	cargo build --package=client-apollo

# ============== BUILD SINK ==============
sink: streamer relay sinker

streamer:
	cargo build --package=streamer

sinker:
	cargo build --package=sinker

relay:
	cargo build --package=relay

sink-release:
	cargo build --package=streamer --package=sinker --package=relay --release

# ============== SYNC HOTSTUFF ===========
synchs: node-synchs client-synchs

node-synchs:
	cargo build --package=node-synchs

client-synchs:
	cargo build --package=client-synchs