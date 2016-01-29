# Build C++ client example
# Process with GNU make

all: test

check: all
	./test

HEADER := ../include/mp4parse.h
CXXFLAGS = -g -Wall -std=c++11 -I$(dir $(HEADER))

libmp4parse.a libmp4parse.a.out : ../src/lib.rs
	rustc -g --crate-type staticlib --crate-name mp4parse \
	  --emit dep-info,link=$@ \
	  -L ../target/debug/deps/ $< 2> libmp4parse.a.out

-include mp4parse.d

test: RUST_LIBS = $(shell awk '/^note: library: / {print "-l"$$3}' libmp4parse.a.out)
test: test.cc libmp4parse.a $(HEADER)
	$(CXX) $(CXXFLAGS) -c $(filter %.cc,$^)
	$(CXX) $(CXXFLAGS) -o $@ *.o libmp4parse.a $(RUST_LIBS)

clean:
	$(RM) test
	$(RM) *.a.out
	$(RM) *.o *.a
	$(RM) *.d
