.PHONY: testdata node-apollo client-sink tools

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

node-apollo:
	cargo build --package=node-apollo

client-sink:
	cargo build --package=client-sink