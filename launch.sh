#!/bin/bash
/usr/bin/time -f "%E real, %F fault, %I/%O fileIO, %r/%s socketIO, %K mem, %Mko maxmem (%t)" ./target/debug/good-morning --data-path ../good-morning.db run
#valgrind ./target/debug/good-morning --data-path ../good-morning.db run