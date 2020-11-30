build_tcp:
	cargo build --manifest-path tcp/Cargo.toml

run_tcp:
	.\tcp\target\debug\tcp.exe

build_client:
	cargo build --manifest-path client/Cargo.toml

run_client:
	.\client\target\debug\client.exe -i 127.0.0.1 -p 9999 -s amqp://guest:guest@localhost:5672
