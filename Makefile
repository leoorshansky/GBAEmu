S_FILES = ${wildcard tests/*.s}
TEST_NAMES = ${subst .s,,${S_FILES}}
O_FILES = ${addsuffix .o,${TEST_NAMES}}
BIN_FILES = ${addsuffix .bin,${TEST_NAMES}}

${O_FILES} : %.o : Makefile ${S_FILES}
	arm-none-eabi-gcc -o $@ -c $*.s --specs=nosys.specs

${BIN_FILES} : %.bin : Makefile %.o
	arm-none-eabi-objcopy -O binary $*.o $@

tests: Makefile ${BIN_FILES}