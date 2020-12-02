build:
	cargo build

run_tcp:
	cargo run --bin tcp_server -- --tcp_addr 127.0.0.1:9999

run_client:
	cargo run --bin rabbit_to_tcp -- --tcp_addr 127.0.0.1:9999 --rabbit_addr amqp://guest:guest@localhost:5672 --src_queue starta_test_queue --dst_queue enda_test_queue

run_client_oppsite:
	cargo run --bin rabbit_to_tcp -- --tcp_addr 127.0.0.1:9999 --rabbit_addr amqp://guest:guest@localhost:5672 --src_queue startb_test_queue --dst_queue endb_test_queue

run_platform:
	cargo run --bin platform

gen_msg:
	cargo run --bin msg_generator