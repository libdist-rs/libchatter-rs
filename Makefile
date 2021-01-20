.PHONY: testdata tools apollo apollo-release synchs synchs-release sink-exp sink-exp-release release debug

release:
	cargo build --all --release

debug:
	cargo build --all

tools:
	cargo build --package=genconfig --release

testdata:
	@mkdir -p testdata/b{100,400,800,2000,4000}-n3 \
	testdata/b{100,400,800}-n3-p{128,1024} \
	testdata/b400-p0-f{1,4,8,16,32} \
	testdata/test
	for b in 100 400 800 2000 4000 ; do \
		./target/release/genconfig -n 3 -d 50 --blocksize $$b --base_port 4000 --client_base_port 10000 --target testdata/b$$b-n3 ; \
	done
	for b in 100 400 800 ; do \
		for p in 128 1024 ; do \
			./target/release/genconfig -n 3 -d 50 --blocksize $$b --base_port 4000 --client_base_port 10000 --payload $$p --target testdata/b$$b-n3-p$$p ; \
		done \
	done
	for f in 1 4 8 16 32 ; do \
		N=$$(( 2*$$f + 1 )) ; \
		./target/release/genconfig -n $$N -d 50 --blocksize 400 --base_port 4000 --client_base_port 10000 --payload 0 --target testdata/b400-p0-f$$f ;\
	done
	@./target/release/genconfig -n 7 -d 50 --blocksize 100 --base_port 4000 --client_base_port 10000 --target testdata/test

# ============= APOLLO =================================================
apollo-release: 
	cargo build --package=node-apollo --package=client-apollo --package=normal-client-apollo --release

apollo:
	cargo build --package=node-apollo --package=client-apollo --package=normal-client-apollo

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
