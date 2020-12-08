.PHONY: testdata node-apollo client-sink tools sink apollo synchs

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
sink: client-sink replica-sink

client-sink:
	cargo build --package=client-sink

replica-sink:
	cargo build --package=replica-sink

# ============== SYNC HOTSTUFF ===========
synchs: node-synchs client-synchs

node-synchs:
	cargo build --package=node-synchs

client-synchs:
	cargo build --package=client-synchs