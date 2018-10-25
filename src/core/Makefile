TESTS = test_queue test_tree test_base

default: compiled_result

compiled_result.o : compiled_result.c base.h builtin.h
	$(CC) $(CFLAGS) -c compiled_result.c -o compiled_result.o

compiled_result : compiled_result.o base.o builtin.o gc.o tree.o
	$(CC) $(CFLAGS) compiled_result.o gc.o base.o builtin.o tree.o -o compiled_result

base.o : base.c base.h gc.h queue.h tree.h vec.h common.h
	$(CC) $(CFLAGS) -c base.c -o base.o

gc.o : gc.c gc.h base.h vec.h tree.h queue.h
	$(CC) $(CFLAGS) -c gc.c -o gc.o

tree.o : tree.c tree.h vec.h
	$(CC) $(CFLAGS) -c tree.c -o tree.o

builtin.o : builtin.c builtin.h base.h
	$(CC) $(CFLAGS) -c builtin.c -o builtin.o

test_queue : test_queue.c queue.h
	$(CC) $(CFLAGS) test_queue.c -o test_queue

test_tree : test_tree.c tree.h tree.o
	$(CC) $(CFLAGS) test_tree.c tree.o -o test_tree

test_base : test_base.c base.h base.o gc.o tree.o
	$(CC) $(CFLAGS) test_base.c base.o gc.o tree.o -o test_base

.PHONY: clean test

test : $(TESTS)
	@for t in $(TESTS); do \
		./$$t; \
	done

clean :
	-rm -f base.o
	-rm -f base
	-rm -f gc.o
	-rm -f tree.o
	-rm -f builtin.o
	-rm -f compiled_result.o
	-rm -f test_tree
	-rm -f test_queue
	-rm -f test_base
	-rm -f compiled_result
