release:
	cargo build --release

run: release
	@echo "Running on Dr. Kim's test program..."
	./target/release/parser samples/kim_example.ssc

debug:
	cargo build
	@echo "Opening lldb..."
	lldb ./target/debug/parser samples/kim_example.ssc

watch:
	cargo watch -x fmt -x build -x test

test:
	cargo test

clean:
	cargo clean
	rm *.svg ; \
	rm -rf *.out*

loc: clean
	cloc .

flamegraph:
	sudo cargo flamegraph --dev -- samples/kim_example.ssc

cov:
	cargo llvm-cov
