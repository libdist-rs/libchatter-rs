.PHONY: testdata tools apollo apollo-release synchs synchs-release sink-exp sink-exp-release release debug

release:
	cargo build --all --release

debug:
	cargo build --all

tools:
	cargo build --package=genconfig --release

testdata:
	@mkdir -p testdata/b400-n3 testdata/b100-n3 testdata/b800-n3 \
	testdata/b800-n3-p128 testdata/b400-n3-p128 testdata/b100-n3-p128 \
	testdata/b800-n3-p1024 testdata/b400-n3-p1024 testdata/b100-n3-p1024 \
	testdata/test testdata/b400-p0-f{1,4,8,32}
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 1 \
	--base_port 4000 \
	--client_base_port 10000 \
	--target testdata/test
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--target testdata/b400-n3
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 100 \
	--base_port 4000 \
	--client_base_port 10000 \
	--target testdata/b100-n3
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 800 \
	--base_port 4000 \
	--client_base_port 10000 \
	--target testdata/b800-n3
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 800 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 128 \
	--target testdata/b800-n3-p128
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 128 \
	--target testdata/b400-n3-p128
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 100 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 128 \
	--target testdata/b100-n3-p128
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 800 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 1024 \
	--target testdata/b800-n3-p1024
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 1024 \
	--target testdata/b400-n3-p1024
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 100 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 1024 \
	--target testdata/b100-n3-p1024
	# testdata/b400-p0-f{1,4,8,32}
	@./target/debug/genconfig \
	-n 3 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 0 \
	--target testdata/b400-p0-f1
	@./target/debug/genconfig \
	-n 9 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 0 \
	--target testdata/b400-p0-f4
	@./target/debug/genconfig \
	-n 17 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 0 \
	--target testdata/b400-p0-f8
	@./target/debug/genconfig \
	-n 64 \
	-d 50 \
	--blocksize 400 \
	--base_port 4000 \
	--client_base_port 10000 \
	--payload 0 \
	--target testdata/b400-p0-f32

# ============= APOLLO =================================================
apollo-release: 
	cargo build --package=node-apollo --package=client-apollo --release

apollo:
	cargo build --package=node-apollo --package=client-apollo

# ============== PING-EXP ===============================================
ping-release:
	cargo build --package=ping --package=echo --release

ping:
	cargo build --package=ping --package=echo

# ============== SYNC HOTSTUFF ==========================================
synchs-release: 
	cargo build --package=node-synchs --package=client-synchs --release

synchs:
	cargo build --package=node-synchs --package=client-synchs
