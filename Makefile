S_FILES = ${wildcard tests/*.s}
TEST_NAMES = ${subst .s,,${S_FILES}}
O_FILES = ${addsuffix .o,${TEST_NAMES}}
BIN_FILES = ${addsuffix .bin,${TEST_NAMES}}

${O_FILES} : %.o : %.s
	arm-none-eabi-gcc -march=armv4t -o $@ -c $*.s --specs=nosys.specs

${BIN_FILES} : %.bin : Makefile %.o
	arm-none-eabi-objcopy -O binary $*.o $@

THUMB_S_FILES = ${wildcard thumb_tests/*.s}
THUMB_TEST_NAMES = ${subst .s,,${THUMB_S_FILES}}
THUMB_O_FILES = ${addsuffix .o,${THUMB_TEST_NAMES}}
THUMB_BIN_FILES = ${addsuffix .bin,${THUMB_TEST_NAMES}}

${THUMB_O_FILES} : %.o : Makefile %.s
	arm-none-eabi-gcc -march=armv4t -o $@ -c $*.s --specs=nosys.specs

${THUMB_BIN_FILES} : %.bin : Makefile %.o
	arm-none-eabi-objcopy -O binary $*.o $@

C_FILES = ${wildcard c_tests/*.c}
C_TEST_NAMES = ${subst .c,,${C_FILES}}
C_BIN_FILES = ${addsuffix .bin,${C_TEST_NAMES}}

${C_BIN_FILES} : %.bin : Makefile %.c c_tests/startup.s
	arm-none-eabi-gcc -Tc_tests/script.ld -o $*.o $*.c c_tests/startup.s -nostdlib
	arm-none-eabi-objcopy -O binary $*.o $@

tests: Makefile ${BIN_FILES}

thumb_tests: Makefile ${THUMB_BIN_FILES}

c_tests: Makefile ${C_BIN_FILES}

clean:
	rm -f ${O_FILES}
	rm -f ${THUMB_O_FILES}
	rm -f ${addsuffix .o,${C_TEST_NAMES}}