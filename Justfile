release:
	cargo build --release

build:
	cargo build

run: build
	@echo "Running on Dr. Kim's test program..."
	./target/debug/parser samples/new_example.ssc

out:
	just run > output.out

debug:
	cargo build
	@echo "Opening lldb..."
	lldb ./target/debug/parser samples/new_example.ssc

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
	sudo cargo flamegraph --dev -- samples/new_example.ssc

cov:
	cargo llvm-cov
