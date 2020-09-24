TESTS = test_queue test_base

default: compiled_result

compiled_result.o : compiled_result.c base.h builtin.h
	$(CC) $(CFLAGS) -c compiled_result.c -o compiled_result.o

compiled_result : compiled_result.o base.o builtin.o gc.o bit_array.o
	$(CC) $(CFLAGS) compiled_result.o gc.o base.o builtin.o bit_array.o -o compiled_result

base.o : base.c base.h gc.h queue.h vec.h common.h
	$(CC) $(CFLAGS) -c base.c -o base.o

gc.o : gc.c gc.h base.h hash_table.h vec.h queue.h
	$(CC) $(CFLAGS) -c gc.c -o gc.o

builtin.o : builtin.c builtin.h base.h
	$(CC) $(CFLAGS) -c builtin.c -o builtin.o

bit_array.o : bit_array.c bit_array.h
	$(CC) $(CFLAGS) -c bit_array.c -o bit_array.o

test_queue : test_queue.c queue.h
	$(CC) $(CFLAGS) test_queue.c -o test_queue

test_base : test_base.c base.h base.o gc.o
	$(CC) $(CFLAGS) test_base.c base.o gc.o -o test_base

.PHONY: clean test

test : $(TESTS)
	@for t in $(TESTS); do \
		./$$t; \
	done

clean :
	-rm -f base.o
	-rm -f base
	-rm -f gc.o
	-rm -f builtin.o
	-rm -f compiled_result.o
	-rm -f bit_array.o
	-rm -f test_queue
	-rm -f test_base
	-rm -f compiled_result
