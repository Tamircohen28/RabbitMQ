build:
	cargo build
	
run_tcp:
	.\target\debug\tcp.exe

run_client:
	.\target\debug\client.exe -i 127.0.0.1 -p 9999 -s amqp://guest:guest@localhost:5672 --rabbit_queue_src starta_test_queue --rabbit_queue_dst enda_test_queue

run_client_oppsite:
	.\target\debug\client.exe -i 127.0.0.1 -p 9999 -s amqp://guest:guest@localhost:5672 --rabbit_queue_src startb_test_queue --rabbit_queue_dst endb_test_queue 
